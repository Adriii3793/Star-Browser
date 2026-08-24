import { defaultFavorites, type Favorite } from '$lib/data/favorites';

const STORAGE_KEY = 'star.favorites';

class FavoritesStore {
    items = $state<Favorite[]>([]);

    #loaded = false;

    init() {
        if (this.#loaded || typeof localStorage === 'undefined') return;
        this.#loaded = true;
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            this.items = raw ? (JSON.parse(raw) as Favorite[]) : defaultFavorites.map((f) => ({ ...f }));
        } catch {
            this.items = defaultFavorites.map((f) => ({ ...f }));
        }
    }

    #persist() {
        if (typeof localStorage === 'undefined') return;
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(this.items));
        } catch {

        }
    }

    add(title: string, url: string) {
        const normalized = url.trim();
        if (!normalized) return;
        this.items = [...this.items, { id: crypto.randomUUID(), title: title.trim() || 'Favorite', url: normalized }];
        this.#persist();
    }

    hasUrl(url: string): boolean {
        const normalized = url.trim();
        return normalized.length > 0 && this.items.some((item) => item.url.trim() === normalized);
    }

    upsertFromUrl(title: string, url: string) {
        const normalized = url.trim();
        if (!normalized) return;
        const existing = this.items.find((item) => item.url.trim() === normalized);
        if (existing) {
            this.items = this.items.map((item) => item.id === existing.id ? { ...item, title: title.trim() || item.title } : item);
        } else {
            this.items = [...this.items, { id: crypto.randomUUID(), title: title.trim() || 'Favorite', url: normalized }];
        }
        this.#persist();
    }

    update(id: string, title: string, url: string) {
        this.items = this.items.map((f) => (f.id === id ? { ...f, title, url } : f));
        this.#persist();
    }

    remove(id: string) {
        this.items = this.items.filter((f) => f.id !== id);
        this.#persist();
    }

    reorder(from: number, to: number) {
        const items = [...this.items];
        const [moved] = items.splice(from, 1);
        items.splice(to, 0, moved);
        this.items = items;
        this.#persist();
    }
}

export const favorites = new FavoritesStore();
