import { getCurrentWindow, currentMonitor } from '@tauri-apps/api/window';
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';

type Platform = 'macos' | 'windows' | 'linux';
type Bounds = { position: PhysicalPosition; size: PhysicalSize };

class WindowChromeStore {
	maximized = $state(false);

	#os: Platform = 'windows';
	#restoreBounds: Bounds | null = null;
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
					if (this.#restoreBounds) {
						await win.setPosition(this.#restoreBounds.position);
						await win.setSize(this.#restoreBounds.size);
						this.#restoreBounds = null;
					} else {
						await win.unmaximize();
					}
					this.maximized = false;
				} else {
					await this.#fillWorkArea(true);
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
				this.maximized = (await win.isMaximized()) || this.#restoreBounds !== null;
			} else {
				this.maximized = (await win.isMaximized()) || (await win.isFullscreen());
			}
		} catch {
			this.maximized = false;
		}
	}

	async #fillWorkArea(keepFlag: boolean) {
		const win = getCurrentWindow();
		const [position, size, monitor] = await Promise.all([
			win.outerPosition(),
			win.outerSize(),
			currentMonitor()
		]);
		if (!monitor) return;
		this.#restoreBounds = { position, size };
		await win.setSize(monitor.workArea.size);
		await win.setPosition(monitor.workArea.position);
		if (keepFlag) this.maximized = true;
	}
}

export const windowChrome = new WindowChromeStore();
