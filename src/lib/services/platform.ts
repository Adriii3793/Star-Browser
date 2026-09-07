import { platform as tauriPlatform } from '@tauri-apps/plugin-os';

export type OS = 'macos' | 'windows' | 'linux';

export function detectOs(): OS {
	try {
		const p = tauriPlatform();
		return p === 'macos' ? 'macos' : p === 'linux' ? 'linux' : 'windows';
	} catch {
		if (typeof navigator === 'undefined') return 'windows';
		const ua = navigator.userAgent;
		if (/Macintosh|Mac OS X/.test(ua)) return 'macos';
		if (/Linux|X11/.test(ua) && !/Android/.test(ua)) return 'linux';
		return 'windows';
	}
}
