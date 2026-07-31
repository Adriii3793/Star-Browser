import { call } from './ipc';

export interface TabSessionEntry {
    url: string;
    title: string;
    searchText?: string | null;
    hasNavigated: boolean;
    hist: string[];
    cursor: number;
    zoom: number;
    groupId?: string | null;
}

export interface TabSessionGroup {
    id: string;
    name: string;
    color: string;
}

export interface TabSession {
    tabs: TabSessionEntry[];
    groups: TabSessionGroup[];
    activeIndex: number;
}

export function saveTabSession(session: TabSession): Promise<void> {
    return call('save_tab_session', { session });
}

export function loadTabSession(): Promise<TabSession | null> {
    return call('load_tab_session');
}
