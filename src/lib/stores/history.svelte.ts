import {
    recentHistory,
    recordVisit,
    retitleVisit,
    searchHistory,
    clearHistory,
    historyWriteError
} from '$lib/services/history';
import type { HistoryEntry } from '$lib/types';

const RELOAD_DEBOUNCE_MS = 400;

function reason(cause: unknown): string {
    if (typeof cause === 'string') return cause;
    if (cause instanceof Error) return cause.message;
    return 'History is unavailable.';
}

class HistoryStore {
    entries = $state<HistoryEntry[]>([]);
    loading = $state(false);
    error = $state<string | null>(null);
    loaded = $state(false);

    #limit = 20;
    #reloadTimer: ReturnType<typeof setTimeout> | undefined;

    async load(limit = this.#limit) {
        this.#limit = limit;
        this.loading = true;
        try {
            this.entries = await recentHistory(limit);
            this.loaded = true;
            this.error = null;
            if (this.entries.length === 0) {
                this.error = await historyWriteError().catch(() => null);
            }
        } catch (cause) {
            this.error = reason(cause);
        } finally {
            this.loading = false;
        }
    }

    async record(url: string, title: string, query: string | null) {
        try {
            await recordVisit(url, title, query);
            this.error = null;
        } catch (cause) {
            this.error = reason(cause);
            return;
        }
        clearTimeout(this.#reloadTimer);
        this.#reloadTimer = setTimeout(() => void this.load(), RELOAD_DEBOUNCE_MS);
    }

    async retitle(url: string, title: string) {
        try {
            await retitleVisit(url, title);
        } catch {
            return;
        }
        clearTimeout(this.#reloadTimer);
        this.#reloadTimer = setTimeout(() => void this.load(), RELOAD_DEBOUNCE_MS);
    }

    async search(term: string) {
        clearTimeout(this.#reloadTimer);
        try {
            this.entries = term.trim()
                ? await searchHistory(term, this.#limit)
                : await recentHistory(this.#limit);
            this.loaded = true;
            this.error = null;
        } catch (cause) {
            this.error = reason(cause);
        }
    }

    async clear() {
        clearTimeout(this.#reloadTimer);
        try {
            await clearHistory();
            this.entries = [];
            this.error = null;
        } catch (cause) {
            this.error = reason(cause);
        }
    }

    retry() {
        void this.load();
    }
}

export const history = new HistoryStore();
