import {call} from './ipc';

export type ContentPart =
    | { type: 'text'; text: string }
    | { type: 'image_url'; image_url: { url: string } };

export interface ChatMessage {
    role: 'user' | 'assistant';
    content: string | ContentPart[];
}

export interface UsageStatus {
    used: number;
    limit: number;
}

export function usageStatus(): Promise<UsageStatus> {
    return call('usage_status');
}
export function aiChat(messages: ChatMessage[]): Promise<string> {
    return call('ai_chat', {messages});
}
