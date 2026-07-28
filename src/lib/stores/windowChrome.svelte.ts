import { getCurrentWindow } from '@tauri-apps/api/window';

type Platform = 'macos' | 'windows' | 'linux';

class WindowChromeStore {
	maximized = $state(false);

	#os: Platform = 'windows';
	#unlisten: (() => void) | undefined;

	async init(os: Platform) {
		this.destroy();
		this.#os = os;
		const win = getCurrentWindow();

		try {
			await this.#refresh();
		} catch {
			this.maximized = false;
		}

		try {
			this.#unlisten = await win.onResized(() => this.#refresh());
		} catch {
			this.#unlisten = undefined;
		}
	}

	destroy() {
		this.#unlisten?.();
		this.#unlisten = undefined;
	}

	async toggle() {
		const win = getCurrentWindow();
		try {
			if (this.#os === 'windows') {
				if (this.maximized) {
					await win.unmaximize();
					this.maximized = false;
				} else {
					// Let Windows compute the fullscreen work-area bounds for a
					// transparent frameless window. Manual size/position math left a
					// visible empty strip on the left edge after maximizing.
					this.maximized = true;
					await win.maximize();
				}
			} else {
				if (await win.isMaximized()) {
					await win.unmaximize();
				} else {
					await win.maximize();
				}
				await this.#refresh();
			}
		} catch {
		}
	}

	async #refresh() {
		const win = getCurrentWindow();
		try {
			if (this.#os === 'windows') {
				this.maximized = await win.isMaximized();
			} else {
				this.maximized = (await win.isMaximized()) || (await win.isFullscreen());
			}
		} catch {
			this.maximized = false;
		}
	}
}

export const windowChrome = new WindowChromeStore();
