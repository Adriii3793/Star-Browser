import { getCurrentWindow, currentMonitor, type Window } from '@tauri-apps/api/window';

const EDGE_TOLERANCE = 2;

class WindowChromeStore {
	maximized = $state(false);
	fullscreen = $state(false);
	snapped = $state(false);

	#win: Window | null = null;
	#unlistenResized: (() => void) | undefined;
	#unlistenMoved: (() => void) | undefined;
	#unlistenFullScreen: (() => void) | undefined;
	#unlistenViewport: (() => void) | undefined;
	#refreshToken = 0;
	#refs = 0;
	#refreshing = false;
	#refreshQueued = false;

	get squared(): boolean {
		return this.maximized || this.fullscreen || this.snapped;
	}

	async init() {
		this.#refs += 1;
		if (this.#refs > 1) {
			await this.refresh().catch(() => {});
			return;
		}

		try {
			this.#win = getCurrentWindow();
		} catch {
			this.#win = null;
		}

		await this.refresh().catch(() => {});

		if (this.#win) {
			try {
				this.#unlistenResized = await this.#win.onResized(() => void this.refresh());
			} catch {
				this.#unlistenResized = undefined;
			}

			if (typeof window !== 'undefined') {
				let raf = 0;
				const onViewportResize = () => {
					cancelAnimationFrame(raf);
					raf = requestAnimationFrame(() => void this.refresh());
				};
				window.addEventListener('resize', onViewportResize, { passive: true });
				this.#unlistenViewport = () => {
					cancelAnimationFrame(raf);
					window.removeEventListener('resize', onViewportResize);
				}
			}
			try {
				this.#unlistenMoved = await this.#win.onMoved(() => void this.refresh());
			} catch {
				this.#unlistenMoved = undefined;
			}
		}

		if (typeof document !== 'undefined') {
			const onFsChange = () => {
				const docEl = document.documentElement as HTMLElement & {
					webkitIsFullScreen?: boolean;
					msFullscreenElement?: Element;
				};
				const nativeFullscreen = Boolean(
					document.fullscreenElement ||
					docEl.webkitIsFullScreen ||
					docEl.msFullscreenElement
				);
				if (nativeFullscreen !== this.fullscreen) {
					this.fullscreen = nativeFullscreen;
				}
			};
			const opts: AddEventListenerOptions = { passive: true };
			document.addEventListener('fullscreenchange', onFsChange, opts);
			document.addEventListener('webkitfullscreenchange', onFsChange, opts);
			document.addEventListener('msfullscreenchange', onFsChange, opts);
			this.#unlistenFullScreen = () => {
				document.removeEventListener('fullscreenchange', onFsChange);
				document.removeEventListener('webkitfullscreenchange', onFsChange);
				document.removeEventListener('msfullscreenchange', onFsChange);
			};
		}
	}

	destroy() {
		if (this.#refs === 0) return;
		this.#refs -= 1;
		if (this.#refs > 0) return;

		this.#unlistenResized?.();
		this.#unlistenResized = undefined;
		this.#unlistenMoved?.();

		this.#unlistenMoved = undefined;
		this.#unlistenFullScreen?.();
		this.#unlistenFullScreen = undefined;
		this.#unlistenViewport?.();
		this.#unlistenViewport = undefined;
	}

	async toggle() {
		try {
			if (this.fullscreen) {
				await this.setFullscreen(false);
			} else if (this.#win && (await this.#win.isMaximized())) {
				await this.#win.unmaximize();
			} else if (this.#win) {
				await this.#win.maximize();
			}
		} catch {}
		await this.refresh();
	}

	async setFullscreen(next: boolean) {
		const nativeSet = async (want: boolean) => {
			if (typeof document === 'undefined') return;
			const docEl = document.documentElement as HTMLElement & {
				webkitRequestFullscreen?: () => Promise<void>;
				msRequestFullscreen?: () => Promise<void>;
			};
			const anyDoc = document as Document & {
				webkitExitFullscreen?: () => Promise<void>;
				msExitFullscreen?: () => Promise<void>;
			};
			try {
				if (want) {
					if (docEl.requestFullscreen) await docEl.requestFullscreen();
					else if (docEl.webkitRequestFullscreen) await docEl.webkitRequestFullscreen();
					else if (docEl.msRequestFullscreen) await docEl.msRequestFullscreen();
				} else {
					if (anyDoc.exitFullscreen) await anyDoc.exitFullscreen();
					else if (anyDoc.webkitExitFullscreen) await anyDoc.webkitExitFullscreen();
					else if (anyDoc.msExitFullscreen) await anyDoc.msExitFullscreen();
				}
			} catch {}
		};

		let tauriOk = false;
		if (this.#win) {
			try {
				await this.#win.setFullscreen(next);
				tauriOk = true;
			} catch {
				tauriOk = false;
			}
		}
		if (!tauriOk) {
			await nativeSet(next);
		}
		await this.refresh();
	}

	async refresh() {
		if (this.#refreshing) {
			this.#refreshQueued = true;
			return;
		}
		this.#refreshing = true;
		try {
			do {
				this.#refreshQueued = false;
				await this.#read();
			} while (this.#refreshQueued);
		} finally {
			this.#refreshing = false;
			this.#refreshQueued = false;
		}
	}

	async #read() {
		const token = ++this.#refreshToken;
		let winFs = false;
		let winMax = false;
		if (this.#win) {
			try {
				winFs = await this.#win.isFullscreen();
				winMax = await this.#win.isMaximized();
			} catch {
				winFs = false;
				winMax = false;
			}
		}

		if (token !== this.#refreshToken) return;

		let nativeFs = false;
		if (typeof document !== 'undefined') {
			const docEl = document.documentElement as HTMLElement & {
				webkitIsFullScreen?: boolean;
				msFullscreenElement?: Element;
			};
			nativeFs = Boolean(
				document.fullscreenElement ||
					docEl.webkitIsFullScreen ||
					docEl.msFullscreenElement
			);
		}

		const fullscreen = winFs || nativeFs;
		this.fullscreen = fullscreen;
		this.maximized = winMax;
		this.snapped = winMax || fullscreen ? false : await this.#detectSnapped();
	}

	async #detectSnapped(): Promise<boolean> {
		if (!this.#win) return false;
		try {
			const [pos, size, monitor] = await Promise.all([
				this.#win.outerPosition(),
				this.#win.outerSize(),
				currentMonitor()
			]);
			if (!monitor) return false;

			const area = monitor.workArea;
			const flushLeft = pos.x <= area.position.x + EDGE_TOLERANCE;
			const flushTop = pos.y <= area.position.y + EDGE_TOLERANCE;
			const flushRight =
				pos.x + size.width >= area.position.x + area.size.width - EDGE_TOLERANCE;
			const flushBottom =
				pos.y + size.height >= area.position.y + area.size.height - EDGE_TOLERANCE;

			return (flushLeft || flushRight) && (flushTop || flushBottom);
		} catch {
			return false;
		}
	}
}

export const windowChrome = new WindowChromeStore();
