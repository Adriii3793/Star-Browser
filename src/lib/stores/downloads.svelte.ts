export interface DownloadEntry {
    id: string;
    fileName: string;
    state: 'downloading' | 'complete' | 'failed';
    at: number;
}

const STORAGE_KEY = 'star.downloads';
const MAX_ENTRIES = 100;

class DownloadsStore {
    entries = $state<DownloadEntry[]>([]);

    #loaded = false;

    init() {
        if (this.#loaded) return;
        this.#loaded = true;
        this.reload();
    }

    reload() {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (!raw) return;
            const parsed = JSON.parse(raw);
            if (Array.isArray(parsed)) {
                this.entries = parsed.filter(
                    (e) => typeof e?.id === 'string' && typeof e?.fileName === 'string'
                );
            }
        } catch {}
    }

    started(fileName: string): string {
        const id = crypto.randomUUID();
        this.entries = [
            { id, fileName, state: 'downloading' as const, at: Date.now() },
            ...this.entries
        ].slice(0, MAX_ENTRIES);
        this.#persist();
        return id;
    }

    finished(fileName: string, success: boolean) {
        const open = this.entries.find(
            (e) => e.fileName === fileName && e.state === 'downloading'
        );
        if (open) {
            this.entries = this.entries.map((e) =>
                e.id === open.id ? { ...e, state: success ? ('complete' as const) : ('failed' as const) } : e
            );
        } else {
            this.entries = [
                {
                    id: crypto.randomUUID(),
                    fileName,
                    state: success ? ('complete' as const) : ('failed' as const),
                    at: Date.now()
                },
                ...this.entries
            ].slice(0, MAX_ENTRIES);
        }
        this.#persist();
    }

    remove(id: string) {
        this.entries = this.entries.filter((e) => e.id !== id);
        this.#persist();
    }

    clear() {
        this.entries = [];
        this.#persist();
    }

    #persist() {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify($state.snapshot(this.entries)));
        } catch {}
    }
}

export const downloads = new DownloadsStore();
