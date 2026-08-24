import { getCurrentWindow, type Window } from '@tauri-apps/api/window';

class WindowChromeStore {
	maximized = $state(false);
	fullscreen = $state(false);

	#win: Window | null = null;
	#unlistenResized: (() => void) | undefined;
	#unlistenFullscreen: (() => void) | undefined;

	get squared(): boolean {
		return this.maximized || this.fullscreen;
	}

	async init() {
		this.destroy();
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
			this.#unlistenFullscreen = () => {
				document.removeEventListener('fullscreenchange', onFsChange);
				document.removeEventListener('webkitfullscreenchange', onFsChange);
				document.removeEventListener('msfullscreenchange', onFsChange);
			};
		}
	}

	destroy() {
		this.#unlistenResized?.();
		this.#unlistenResized = undefined;
		this.#unlistenFullscreen?.();
		this.#unlistenFullscreen = undefined;
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

		this.fullscreen = winFs || nativeFs;
		this.maximized = winMax;
	}
}

export const windowChrome = new WindowChromeStore();
