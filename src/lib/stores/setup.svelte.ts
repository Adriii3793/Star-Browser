import { loadSetup, saveSetup } from '$lib/services/setup';

export interface SetupData {
    name: string;
    avatar: string | null;
    searchEngine: string;
    theme: string;
    background: string | null;
    customBg: string | null;
    customSurface: string | null;
    customAccent: string | null;
}

export const DEFAULT_SETUP: SetupData = {
    name: '',
    avatar: null,
    searchEngine: 'google',
    theme: 'system',
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

export const STEPS = ['loading', 'welcome', 'profile', 'search', 'style', 'review'] as const;
export type Step = (typeof STEPS)[number];

const FIRST_DOT_STEP = 2;
export const DOT_COUNT = STEPS.length - FIRST_DOT_STEP;

class SetupStore {
    step = $state<Step>('loading');
    data = $state<SetupData>({ ...DEFAULT_SETUP });
    loaded = $state(false);

    get dotIndex() {
        return STEPS.indexOf(this.step) - FIRST_DOT_STEP;
    }

    get canGoBack() {
        return STEPS.indexOf(this.step) > 1;
    }

    get engine(): SearchEngine {
        return SEARCH_ENGINES.find((e) => e.id === this.data.searchEngine) ?? SEARCH_ENGINES[0];
    }

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

    async load() {
        try {
            const saved = await loadSetup();
            if (saved) this.data = { ...DEFAULT_SETUP, ...saved };
        } catch {
        } finally {
            this.loaded = true;
        }
    }

    async save() {
        await saveSetup($state.snapshot(this.data)).catch(() => {});
    }
}

export const setup = new SetupStore();
