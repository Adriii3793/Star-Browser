import { getCurrentWindow, currentMonitor } from '@tauri-apps/api/window';
import type { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/dpi';

type Platform = 'macos' | 'windows' | 'linux';
type Bounds = { position: PhysicalPosition; size: PhysicalSize };

/**
 * Single source of truth for "is the window maximized/fullscreen".
 *
 * Windows undecorated windows must never use the native maximize() call:
 * WebView2/tao draws it past the work area and covers the taskbar. Instead we
 * emulate maximize by resizing the window to the current monitor's work area
 * and remember the prior bounds to restore later. macOS/Linux use the native
 * maximize/fullscreen APIs directly.
 *
 * WindowControls (the titlebar button) and the outer app shell (rounded
 * corners + resize handles) both need to agree on this flag, so it lives here
 * instead of being recomputed independently in each component.
 */
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
			if (os === 'windows' && (await win.isMaximized())) {
				// Never trust a native maximize on startup (e.g. leftover OS
				// state): unmaximize to recover the pre-maximize bounds, then
				// re-apply our own work-area emulation on top of them.
				await win.unmaximize();
				await this.#fillWorkArea(true);
			} else {
				await this.#refresh();
			}
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
			// Window may have closed mid-toggle; nothing to recover.
		}
	}

	async #refresh() {
		const win = getCurrentWindow();
		try {
			if (this.#os === 'windows') {
				// While faked-maximized the OS reports isMaximized() === false,
				// so fall back to whether we're still tracking restore bounds.
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
		await win.setPosition(monitor.workArea.position);
		await win.setSize(monitor.workArea.size);
		if (keepFlag) this.maximized = true;
	}
}

export const windowChrome = new WindowChromeStore();
