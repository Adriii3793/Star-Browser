import { defaultFavorites, favoriteKey, sameFavorite, type Favorite } from '$lib/data/favorites';

const STORAGE_KEY = 'star.favorites';

class FavoritesStore {
    items = $state<Favorite[]>([]);

    #loaded = false;

    init() {
        if (this.#loaded || typeof localStorage === 'undefined') return;
        this.#loaded = true;
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            const parsed = raw ? (JSON.parse(raw) as unknown) : null;
            this.items = Array.isArray(parsed)
                ? this.#dedupe(parsed as Favorite[])
                : defaultFavorites.map((f) => ({ ...f }));
        } catch {
            this.items = defaultFavorites.map((f) => ({ ...f }));
        }
    }

    #dedupe(list: Favorite[]): Favorite[] {
        const seen = new Set<string>();
        return list.filter((item) => {
            if (typeof item?.url !== 'string' || typeof item?.id !== 'string') return false;
            const key = favoriteKey(item.url);
            if (!key || seen.has(key)) return false;
            seen.add(key);
            return true;
        });
    }

    #persist() {
        if (typeof localStorage === 'undefined') return;
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(this.items));
        } catch {
        }
    }

    #find(url: string): Favorite | undefined {
        return this.items.find((item) => sameFavorite(item.url, url));
    }

    add(title: string, url: string) {
        const normalized = url.trim();
        if (!normalized) return;
        const existing = this.#find(normalized);
        if (existing) {
            this.update(existing.id, title.trim() || existing.title, normalized);
            return;
        }
        this.items = [
            ...this.items,
            { id: crypto.randomUUID(), title: title.trim() || 'Favorite', url: normalized }
        ];
        this.#persist();
    }

    hasUrl(url: string): boolean {
        return this.#find(url) !== undefined;
    }

    upsertFromUrl(title: string, url: string) {
        const normalized = url.trim();
        if (!normalized) return;
        const existing = this.#find(normalized);
        if (existing) {
            this.items = this.items.map((item) =>
                item.id === existing.id ? { ...item, title: title.trim() || item.title } : item
            );
        } else {
            this.items = [
                ...this.items,
                { id: crypto.randomUUID(), title: title.trim() || 'Favorite', url: normalized }
            ];
        }
        this.#persist();
    }

    removeByUrl(url: string) {
        const existing = this.#find(url);
        if (existing) this.remove(existing.id);
    }

    update(id: string, title: string, url: string) {
        const clash = this.items.find((f) => f.id !== id && sameFavorite(f.url, url));
        this.items = this.items
            .filter((f) => f.id !== clash?.id)
            .map((f) => (f.id === id ? { ...f, title, url } : f));
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
