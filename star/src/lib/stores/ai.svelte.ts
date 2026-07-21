import {aiChat, usageStatus, type ChatMessage} from '$lib/services/ai';

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

    async send(text: string) {
        const content = text.trim();
        if (!content || this.sending) return;

        this.error = null;
        this.messages.push({ role: 'user', content});
        this.sending = true;

        try {
            const reply = await aiChat(this.messages);
            this.messages.push({role: 'assistant', content: reply});
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
}

export const ai = new AiStore();