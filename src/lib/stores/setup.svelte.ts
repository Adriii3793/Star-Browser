import { loadSetup, saveSetup } from '$lib/services/setup';

export interface SetupData {
    name: string;
    avatar: string | null;
    searchEngine: string;
    theme: string;
    background: string | null;
    /** Custom theme colors, only set when the user imported a theme file. */
    customBg: string | null;
    customSurface: string | null;
    customAccent: string | null;
}

export const DEFAULT_SETUP: SetupData = {
    name: '',
    avatar: null,
    searchEngine: 'google',
    theme: 'light',
    background: null,
    customBg: null,
    customSurface: null,
    customAccent: null
};

export interface SearchEngine {
    id: string;
    name: string;
    initial: string;
    color: string;
    url: string;
}

export const SEARCH_ENGINES: SearchEngine[] = [
    { id: 'google', name: 'Google', initial: 'G', color: '#4285F4', url: 'https://www.google.com/search?q=' },
    { id: 'duckduckgo', name: 'DuckDuckGo', initial: 'D', color: '#DE5833', url: 'https://duckduckgo.com/?q=' },
    { id: 'brave', name: 'Brave', initial: 'B', color: '#FB542B', url: 'https://search.brave.com/search?q=' },
    { id: 'bing', name: 'Bing', initial: 'B', color: '#008373', url: 'https://www.bing.com/search?q=' }
];

// 'loading' and 'welcome' sit before the dotted progress indicator.
export const STEPS = ['loading', 'welcome', 'profile', 'search', 'style', 'browser', 'review'] as const;
export type Step = (typeof STEPS)[number];

const FIRST_DOT_STEP = 2;
export const DOT_COUNT = STEPS.length - FIRST_DOT_STEP;

class SetupStore {
    step = $state<Step>('loading');
    data = $state<SetupData>({ ...DEFAULT_SETUP });
    /** True once persisted settings have been read back from the database. */
    loaded = $state(false);

    get dotIndex() {
        return STEPS.indexOf(this.step) - FIRST_DOT_STEP;
    }

    /** Mirrors the guard inside back() so the button can never disagree with it. */
    get canGoBack() {
        return STEPS.indexOf(this.step) > 1;
    }

    get engine(): SearchEngine {
        return SEARCH_ENGINES.find((e) => e.id === this.data.searchEngine) ?? SEARCH_ENGINES[0];
    }

    /** Builds a search URL for the engine chosen during setup. */
    searchUrl(query: string): string {
        return this.engine.url + encodeURIComponent(query);
    }

    next() {
        const i = STEPS.indexOf(this.step);
        if (i < STEPS.length - 1) this.step = STEPS[i + 1];
    }

    back() {
        const i = STEPS.indexOf(this.step);
        if (i > 1) this.step = STEPS[i - 1];
    }

    goto(s: Step) {
        this.step = s;
    }

    /** Reads persisted settings. Safe to call before the backend exists. */
    async load() {
        try {
            const saved = await loadSetup();
            if (saved) this.data = { ...DEFAULT_SETUP, ...saved };
        } catch {
            // Backend unavailable: keep defaults rather than blocking startup.
        } finally {
            this.loaded = true;
        }
    }

    async save() {
        await saveSetup($state.snapshot(this.data)).catch(() => {});
    }
}

export const setup = new SetupStore();
