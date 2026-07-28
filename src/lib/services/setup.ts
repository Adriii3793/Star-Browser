import { call } from './ipc';
import type { SetupData } from '$lib/stores/setup.svelte';

/** True once onboarding has been completed and persisted at least once. */
export function isSetupComplete(): Promise<boolean> {
    return call('is_setup_complete');
}

export function saveSetup(data: SetupData): Promise<void> {
    return call('save_setup', { data });
}

export function loadSetup(): Promise<SetupData | null> {
    return call('load_setup');
}

/** Wipes the saved profile so onboarding runs again on next launch. */
export function resetSetup(): Promise<void> {
    return call('reset_setup');
}

/** Absolute path of the folder holding the database and window state. */
export function dataDir(): Promise<string> {
    return call('data_dir');
}

/** Reveals the data folder in the OS file manager. */
export function openDataDir(): Promise<void> {
    return call('open_data_dir');
}

/**
 * Opens the OS "default apps" panel.
 *
 * Windows 10+ and modern macOS deliberately block apps from silently making
 * themselves the default browser, so the honest behaviour is to take the user
 * to the right settings screen. Linux can be set directly via xdg-settings.
 * Resolves to true only when the change was actually applied programmatically.
 */
export function setDefaultBrowser(): Promise<boolean> {
    return call('set_default_browser');
}
