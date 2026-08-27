import {
    recentHistory,
    recordVisit,
    searchHistory,
    clearHistory

} from '$lib/services/history';
import type { HistoryEntry } from '$lib/types';

const RELOAD_DEBOUNCE_MS = 400;

class HistoryStore {
    entries = $state<HistoryEntry[]>([]);
    loading = $state(false);

    #limit = 20;
    #reloadTimer: ReturnType<typeof setTimeout> | undefined;

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
        clearTimeout(this.#reloadTimer);
        this.#reloadTimer = setTimeout(() => void this.load(), RELOAD_DEBOUNCE_MS);
    }

    async search(term: string) {
        clearTimeout(this.#reloadTimer);
        this.entries = term.trim()
            ? await searchHistory(term, this.#limit)
            : await recentHistory(this.#limit);
    }

    async clear() {
        clearTimeout(this.#reloadTimer);
        await clearHistory();
        this.entries = [];
    }
}

export const history = new HistoryStore();