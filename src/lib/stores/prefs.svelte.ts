const STORAGE_KEY = 'star.home.prefs';

interface HomePrefs {
    showFavorites: boolean;
    showRecent: boolean;
}

class PrefsStore {
    showFavorites = $state(true);
    showRecent = $state(true);

    #loaded = false;

    init() {
        if (this.#loaded || typeof localStorage === 'undefined') return;
        this.#loaded = true;
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (raw) {
                const p = JSON.parse(raw) as Partial<HomePrefs>;
                this.showFavorites = p.showFavorites ?? true;
                this.showRecent = p.showRecent ?? true;
            }
        } catch {
        
        }
    }

    #persist() {
        if (typeof localStorage === 'undefined') return;
        try {
            const data: HomePrefs = {
                showFavorites: this.showFavorites,
                showRecent: this.showRecent
            };
            localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
        } catch {
            
        }
    }

    setFavorites(value: boolean) {
        this.showFavorites = value;
        this.#persist();
    }

    setRecent(value: boolean) {
        this.showRecent = value;
        this.#persist();
    }
}

export const prefs = new PrefsStore();