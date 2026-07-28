import { call } from './ipc';

export type ContentPart =
    | { type: 'text'; text: string }
    | { type: 'image_url'; image_url: { url: string } };

export interface ChatMessage {
    // 'system' added: this is where memory + page context get injected.
    role: 'user' | 'assistant' | 'system';
    content: string | ContentPart[];
}

export interface UsageStatus {
    used: number;
    limit: number;
}

export interface PageContext {
    url: string;
    title: string;
    text: string;
    images: string[];
    videos: string[];
    truncated: boolean;
}

export function usageStatus(): Promise<UsageStatus> {
    return call('usage_status');
}

export function aiChat(messages: ChatMessage[]): Promise<string> {
    return call('ai_chat', { messages });
}

export function fetchPageContext(url: string): Promise<PageContext> {
    return call('fetch_page_context', { url });
}
