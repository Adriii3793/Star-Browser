import {aiChat, usageStatus, type ChatMessage, type ContentPart} from '$lib/services/ai';

class AiStore {
    messages = $state<ChatMessage[]>([]);
    sending = $state(false);
    error = $state<string | null>(null);
    used = $state(0);
    limit = $state(0);

    async init() {
        await this.refreshUsage();
    }

    async refreshUsage() {
        const status = await usageStatus();
        this.used = status.used;
        this.limit = status.limit;
    }

    async send(content: string | ContentPart[]) {
        const isEmpty = typeof content === 'string' ? !content.trim() : content.length === 0;
        if (isEmpty || this.sending) return;

        this.error = null;
        this.messages = [...this.messages, { role: 'user', content }];
        this.sending = true;

        try {
            const reply = await aiChat(this.messages);
            this.messages = [...this.messages, { role: 'assistant', content: reply }];
            await this.refreshUsage();
        } catch (e) {
            this.error = String(e);
        } finally {
            this.sending = false;
        }
    }

    reset() {
        this.messages = [];
        this.error = null;
    }

    setMessages(msgs: ChatMessage[]) {
        this.messages = msgs;
    }
}

export const ai = new AiStore();