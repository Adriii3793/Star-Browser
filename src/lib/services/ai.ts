import { call } from './ipc';
import { detectOs } from './platform';
import { settled, showsPage } from './pageUrl';

export type ContentPart =
    | { type: 'text'; text: string }
    | { type: 'image_url'; image_url: { url: string } };

export interface ChatMessage {
    role: 'user' | 'assistant' | 'system';
    content: string | ContentPart[];
}

export interface PageContext {
    url: string;
    title: string;
    text: string;
    images: string[];
    videos: string[];
    truncated: boolean;
    /** Live reads only: the document had not finished loading. */
    loading?: boolean;
}

export interface AiKeyStatus {
    source: 'user' | 'environment' | 'embedded' | 'proxy' | 'none';
    hint: string | null;
}

export function aiKeyStatus(): Promise<AiKeyStatus> {
    return call('ai_key_status');
}

export function setAiKey(key: string): Promise<AiKeyStatus> {
    return call('set_ai_key', { key });
}

export function aiChat(messages: ChatMessage[], model?: string | null): Promise<string> {
    return call('ai_chat', { messages, model: model ?? null });
}

export function fetchPageContext(url: string): Promise<PageContext> {
    return call('fetch_page_context', { url });
}

export function readTabPage(tabId: string): Promise<PageContext | null> {
    return call('read_tab_page', { tabId });
}

const LIVE_ATTEMPTS = 10;
const LIVE_RETRY_MS = 350;
const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Reads the page the tab is showing right now. The tab's URL changes as soon as a navigation
 * starts, so an immediate read can still see the previous (or a blank, loading) document, and
 * script-rendered pages (search results, feeds) keep filling in after load. Retry briefly until
 * the live document is the requested page and its text has stopped changing. Falls back to
 * fetching the URL only when no live read is possible (non-Windows builds, closed webview):
 * many sites, search engines included, serve a server-side fetch an error or "unsupported
 * browser" page instead of what the user sees.
 */
export async function readCurrentPage(tabId: string | null, url: string): Promise<PageContext | null> {
    // Only the WebView2 build can read a tab's live document; elsewhere read_tab_page is always null.
    if (tabId && detectOs() === 'windows') {
        let matched: PageContext | null = null;
        let other: PageContext | null = null;
        let previousText: string | null = null;
        for (let attempt = 0; attempt < LIVE_ATTEMPTS; attempt++) {
            if (attempt > 0) await wait(LIVE_RETRY_MS);
            const live = await readTabPage(tabId).catch(() => null);
            if (!live?.text?.trim()) continue;
            if (!showsPage(live.url, url)) {
                // Maybe the previous page; maybe a redirect the tab URL hasn't caught up with.
                other = live;
                continue;
            }
            matched = live;
            if (!live.loading && settled(previousText, live.text)) return live;
            previousText = live.text;
        }
        // Out of time: what is on screen now is still the best answer.
        if (matched ?? other) return matched ?? other;
    }
    return fetchPageContext(url).catch(() => null);
}

export function saveTextFile(fileName: string, contents: string): Promise<string> {
    return call('save_text_file', { fileName, contents });
}
