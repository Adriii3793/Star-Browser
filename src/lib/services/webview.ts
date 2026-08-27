import {call} from './ipc';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

function bounds(rect: DOMRect) {
    return { x: rect.x, y: rect.y, width: rect.width, height: rect.height};
}

export interface TabCorners {
    radius?: number;
    roundBottomLeft?: boolean;
    roundBottomRight?: boolean;
}

export function openTabWebview(
    tabId: string,
    url: string,
    rect: DOMRect,
    corners?: TabCorners
): Promise<void> {
    return call('open_tab_webview', {tabId, url, ...bounds(rect), ...corners});
}

export function navigateTabWebview(tabId: string, url: string): Promise<void> {
    return call('navigate_tab_webview', {tabId, url});
}

export function setTabBounds(tabId: string, rect: DOMRect, corners?: TabCorners): Promise<void> {
    return call('set_tab_bounds', {tabId, ...bounds(rect), ...corners});
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

export interface TabTitleChanged {
    tabId: string;
    title: string;
}

export function onTabTitleChanged(
    handler: (change: TabTitleChanged) => void
): Promise<UnlistenFn> {
    return listen<TabTitleChanged>('tab-title-changed', (event) => handler(event.payload));
}

export interface TabShortcut {
    tabId: string;
    action: string;
}

export function onTabShortcut(
    handler: (shortcut: TabShortcut) => void
): Promise<UnlistenFn> {
    return listen<TabShortcut>('tab-shortcut', (event) => handler(event.payload));
}

export interface DownloadStarted {
    tabId: string;
    fileName: string;
}

export interface DownloadFinished {
    tabId: string;
    fileName: string;
    success: boolean;
}

export function onDownloadStarted(
    handler: (download: DownloadStarted) => void
): Promise<UnlistenFn> {
    return listen<DownloadStarted>('download-started', (event) => handler(event.payload));
}

export function onDownloadFinished(
    handler: (download: DownloadFinished) => void
): Promise<UnlistenFn> {
    return listen<DownloadFinished>('download-finished', (event) => handler(event.payload));
}

export function openMenuWebview(rect: DOMRect): Promise<void> {
    return call('open_menu_webview', {...bounds(rect)});
}

export function closeMenuWebview(): Promise<void> {
    return call('close_menu_webview', {});
}

export function openOverlayWebview(rect: DOMRect): Promise<void> {
    return call('open_overlay_webview', {...bounds(rect)});
}

export function warmOverlayWebview(rect: DOMRect): Promise<void> {
    return call('warm_overlay_webview', {...bounds(rect)});
}

export function closeOverlayWebview(): Promise<void> {
    return call('close_overlay_webview', {});
}

export function tabBack(tabId: string): Promise<void> {
    return call('tab_back', {tabId});
}
export function tabForward(tabId: string): Promise<void> {
    return call('tab_forward', {tabId});
}
export function tabReload(tabId: string): Promise<void> {
    return call('tab_reload', {tabId});
}
export function tabPrint(tabId: string): Promise<void> {
    return call('tab_print', {tabId});
}

export function setTabZoomWebview(tabId: string, factor: number): Promise<void> {
    return call('set_tab_zoom', {tabId, factor});
}

export function setTabMutedWebview(tabId: string, muted: boolean): Promise<void> {
    return call('set_tab_muted', {tabId, muted});
}

export function tabMediaToggle(tabId: string, playing: boolean): Promise<void> {
    return call('tab_media_toggle', {tabId, playing});
}

export function tabStopMedia(tabId: string): Promise<void> {
    return call('tab_stop_media', {tabId});
}

export function setAdblockEnabled(enabled: boolean): Promise<void> {
    return call('set_adblock', {enabled});
}

export interface TabAudioChanged {
    tabId: string;
    audible: boolean;
    muted: boolean;
}

export function onTabAudioChanged(
    handler: (change: TabAudioChanged) => void
): Promise<UnlistenFn> {
    return listen<TabAudioChanged>('tab-audio-changed', (event) => handler(event.payload));
}
