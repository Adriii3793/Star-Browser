import { call } from './ipc';

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

export function saveTextFile(fileName: string, contents: string): Promise<string> {
    return call('save_text_file', { fileName, contents });
}
