import {
    recentHistory,
    recordVisit,
    searchHistory,
    clearHistory

} from '$lib/services/history';
import type { HistoryEntry } from '$lib/types';

class HistoryStore {
    entries = $state<HistoryEntry[]>([]);
    loading = $state(false);

    #limit = 20;

    async load(limit = this.#limit) {
        this.#limit = limit;
        this.loading = true;
        try {
            this.entries = await recentHistory(limit);
        } finally {
            this.loading = false;
        }
    }

    async record(url: string, title: string, query: string | null) {
        await recordVisit(url, title, query);
        await this.load();
    }

    async search(term: string) {
        this.entries = term.trim()
            ? await searchHistory(term, this.#limit)
            : await recentHistory(this.#limit);
    }

    async clear() {
        await clearHistory();
        this.entries = [];
    }
}

export const history = new HistoryStore();