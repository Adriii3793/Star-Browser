import { getCurrentWindow, currentMonitor, type Window } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';

function rememberFullscreenRestore(maximized: boolean): Promise<void> {
	return invoke<void>('set_fullscreen_restore', { maximized }).catch(() => {});
}

function takeFullscreenRestore(): Promise<boolean> {
	return invoke<boolean>('take_fullscreen_restore').catch(() => false);
}

const EDGE_TOLERANCE = 2;
const FULLSCREEN_STATE_RESTORE_DELAY_MS = 120;

class WindowChromeStore {
	maximized = $state(false);
	fullscreen = $state(false);
	snapped = $state(false);

	#win: Window | null = null;
	#unlistenResized: (() => void) | undefined;
	#unlistenScaleFactor: (() => void) | undefined;
	#unlistenMoved: (() => void) | undefined;
	#unlistenFullScreen: (() => void) | undefined;
	#unlistenViewport: (() => void) | undefined;
	#refreshToken = 0;
	#refs = 0;
	#refreshing = false;
	#refreshQueued = false;
	#fsTransition: Promise<void> | null = null;

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
			try {
				this.#unlistenScaleFactor = await this.#win.onScaleChanged(() => void this.refresh());
			} catch {
				this.#unlistenScaleFactor = undefined;
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

		if (this.#refs === 0) this.#teardown();
	}

	#teardown() {
		this.#unlistenResized?.();
		this.#unlistenResized = undefined;
		this.#unlistenScaleFactor?.();
		this.#unlistenScaleFactor = undefined;
		this.#unlistenMoved?.();
		this.#unlistenMoved = undefined;
		this.#unlistenFullScreen?.();
		this.#unlistenFullScreen = undefined;
		this.#unlistenViewport?.();
		this.#unlistenViewport = undefined;
	}

	destroy() {
		if (this.#refs === 0) return;
		this.#refs -= 1;
		if (this.#refs > 0) return;
		this.#teardown();
	}

	async toggle() {
		try {
			if (this.fullscreen) {
				await this.setFullscreen(false);
			} else if (this.#win && (await this.#win.isMaximized())) {
				await this.#win.unmaximize();
				await this.refresh();
			} else if (this.#win) {
				await this.#win.maximize();
				await this.refresh();
			}
		} catch (e) {
			console.error('[windowChrome] toggle failed:', e);
			await this.refresh();
		}
	}

	async setFullscreen(next: boolean): Promise<void> {
		if (this.#fsTransition) return this.#fsTransition;

		this.#fsTransition = this.#doSetFullscreen(next)
			.catch((e) => {
				console.error('[windowChrome] setFullscreen transition failed:', e);
			})
			.finally(() => {
				this.#fsTransition = null;
			});

		return this.#fsTransition;
	}

	async #doSetFullscreen(next: boolean): Promise<void> {
		const nativeSet = async (want: boolean): Promise<boolean> => {
			if (typeof document === 'undefined') return false;
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
					if (docEl.requestFullscreen) {
						await docEl.requestFullscreen();
						return true;
					} else if (docEl.webkitRequestFullscreen) {
						await docEl.webkitRequestFullscreen();
						return true;
					} else if (docEl.msRequestFullscreen) {
						await docEl.msRequestFullscreen();
						return true;
					}
				} else {
					if (anyDoc.exitFullscreen) {
						await anyDoc.exitFullscreen();
						return true;
					} else if (anyDoc.webkitExitFullscreen) {
						await anyDoc.webkitExitFullscreen();
						return true;
					} else if (anyDoc.msExitFullscreen) {
						await anyDoc.msExitFullscreen();
						return true;
					}
				}
			} catch (e) {
				console.warn('[windowChrome] native Fullscreen API fallback failed:', e);
			}
			return false;
		};

		const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

		if (!this.#win) {
			console.warn('[windowChrome] no Tauri window available, using native fallback only');
			await nativeSet(next);
			await this.refresh();
			return;
		}

		try {
			if (next) {
				const wasMaximized = await this.#win.isMaximized().catch(() => false);
				await rememberFullscreenRestore(wasMaximized);

				if (wasMaximized) {
					try {
						await this.#win.unmaximize();
						await sleep(FULLSCREEN_STATE_RESTORE_DELAY_MS);
					} catch (e) {
						console.warn('[windowChrome] unmaximize before fullscreen failed, proceeding anyway:', e);
					}
				}

				let tauriOk = false;
				try {
					await this.#win.setFullscreen(true);
					tauriOk = true;
				} catch (e) {
					console.error('[windowChrome] Tauri setFullscreen(true) failed:', e);
					tauriOk = false;
				}

				if (!tauriOk) {
					console.warn('[windowChrome] falling back to browser Fullscreen API');
					const nativeOk = await nativeSet(true);
					if (!nativeOk) {
						console.error('[windowChrome] both Tauri and native fullscreen APIs failed to activate');
					}
				}
			} else {
				const shouldRestoreMaximized = await takeFullscreenRestore();
				let tauriExitOk = false;

				try {
					await this.#win.setFullscreen(false);
					tauriExitOk = true;
				} catch (e) {
					console.error('[windowChrome] Tauri setFullscreen(false) failed:', e);
				}

				if (!tauriExitOk) {
					await nativeSet(false);
				}

				if (shouldRestoreMaximized) {
					await sleep(FULLSCREEN_STATE_RESTORE_DELAY_MS);
					try {
						const stillNotMaximized = !(await this.#win.isMaximized().catch(() => false));
						if (stillNotMaximized) {
							await this.#win.maximize();
						}
					} catch (e) {
						console.warn('[windowChrome] restore maximized after fullscreen exit failed:', e);
					}
				}
			}
		} catch (e) {
			console.error('[windowChrome] fullscreen state change encountered unexpected error:', e);
			await nativeSet(next);
		} finally {
			await this.refresh();
		}
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
				[winFs, winMax] = await Promise.all([
					this.#win.isFullscreen(),
					this.#win.isMaximized()
				]);
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
		const snappedCandidate =
			winMax || fullscreen ? Promise.resolve(false) : this.#detectSnapped();
		const snapped = await snappedCandidate;

		if (token !== this.#refreshToken) return;

		this.fullscreen = fullscreen;
		this.maximized = winMax;
		this.snapped = snapped;
	}

	async #detectSnapped(): Promise<boolean> {
		const native = await invoke<boolean | null>('window_tiled').catch(() => null);
		if (typeof native === 'boolean') return native;
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
