import {call} from './ipc';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

function bounds(rect: DOMRect) {
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height};
}

export function openTabWebview(tabId: string, url: string, rect: DOMRect): Promise<void> {
    return call('open_tab_webview', {tabId, url, ...bounds(rect)});

}

export function navigateTabWebview(tabId: string, url: string): Promise<void> {
    return call('navigate_tab_webview', {tabId, url});
}

export function setTabBounds(tabId: string, rect: DOMRect): Promise<void> {
    return call('set_tab_bounds', {tabId, ...bounds(rect)});
}

export function showTabWebview(tabId: string): Promise<void> {
    return call('show_tab_webview', {tabId});
}

export function hideTabWebview(tabId: string): Promise<void> {
    return call('hide_tab_webview', {tabId});
}

export function closeTabWebview(tabId: string): Promise<void> {
    return call('close_tab_webview', {tabId});
}

export interface TabUrlChanged {
    tabId: string;
    url: string;
}

export function onTabUrlChanged(
    handler: (change: TabUrlChanged) => void
): Promise<UnlistenFn> {
    return listen<TabUrlChanged>('tab-url-changed', (event) => handler(event.payload));
}