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
