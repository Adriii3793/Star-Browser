import {call} from './ipc';

export interface ChatMessage {
    role: 'user' | 'assistant';
    content: string;
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