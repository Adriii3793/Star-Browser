import { call } from './ipc';
import type { SetupData } from '$lib/stores/setup.svelte';

export function isSetupComplete(): Promise<boolean> {
    return call('is_setup_complete');
}

export function saveSetup(data: SetupData): Promise<void> {
    return call('save_setup', { data });
}

export function loadSetup(): Promise<SetupData | null> {
    return call('load_setup');
}

export function resetSetup(): Promise<void> {
    return call('reset_setup');
}

export function dataDir(): Promise<string> {
    return call('data_dir');
}

export function openDataDir(): Promise<void> {
    return call('open_data_dir');
}

export function setDefaultBrowser(): Promise<boolean> {
    return call('set_default_browser');
}
