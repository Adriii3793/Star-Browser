const STORAGE_KEY = 'star.home.prefs';

export type AiProviderId = 'deepseek' | 'gemini' | 'llama';

export interface AiProvider {
    id: AiProviderId;
    model: string;
    name: string;
    vendor: string;
    modalities: string;
    disclosure: string;
    /** Whether the model accepts image_url parts. Text-only models get images stripped before sending. */
    vision: boolean;
}

export const AI_PROVIDERS: AiProvider[] = [
    {
        id: 'deepseek',
        model: 'deepseek/deepseek-v4-flash',
        name: 'DeepSeek V4 Flash',
        vendor: 'DeepSeek',
        modalities: 'Text in — text out · 1M context',
        disclosure: 'Requests are routed through OpenRouter to DeepSeek-hosted providers.',
        vision: false
    },
    {
        id: 'gemini',
        model: 'google/gemini-2.5-flash',
        name: 'Gemini 2.5 Flash',
        vendor: 'Google',
        modalities: 'Text, image, audio and video in — text out · 1M context',
        disclosure: 'Google does not retain your data.',
        vision: true
    },
    {
        id: 'llama',
        model: 'meta-llama/llama-3.3-70b-instruct',
        name: 'Llama 3.3 70B',
        vendor: 'Meta',
        modalities: 'Text in — text out · 128K context',
        disclosure: 'Requests are routed through OpenRouter to third-party Llama providers.',
        vision: false
    }
];

export const DEFAULT_PROVIDER: AiProviderId = 'deepseek';

function providerById(id: AiProviderId): AiProvider {
    return AI_PROVIDERS.find((p) => p.id === id) ?? AI_PROVIDERS[0];
}

interface HomePrefs {
    showFavorites: boolean;
    showRecent: boolean;
    skipUngroupedTabs: boolean;
    aiProvider: AiProviderId;
    seenDisclosures: AiProviderId[];
}

class PrefsStore {
    showFavorites = $state(true);
    showRecent = $state(true);
    skipUngroupedTabs = $state(false);
    aiProvider = $state<AiProviderId>(DEFAULT_PROVIDER);
    seenDisclosures = $state<AiProviderId[]>([]);

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
                this.skipUngroupedTabs = p.skipUngroupedTabs ?? false;
                this.aiProvider = AI_PROVIDERS.some((x) => x.id === p.aiProvider)
                    ? (p.aiProvider as AiProviderId)
                    : DEFAULT_PROVIDER;
                this.seenDisclosures = Array.isArray(p.seenDisclosures) ? p.seenDisclosures : [];
            }
        } catch {}
    }

    get provider(): AiProvider {
        return providerById(this.aiProvider);
    }

    get model(): string {
        return this.provider.model;
    }

    #persist() {
        if (typeof localStorage === 'undefined') return;
        try {
            const data: HomePrefs = {
                showFavorites: this.showFavorites,
                showRecent: this.showRecent,
                skipUngroupedTabs: this.skipUngroupedTabs,
                aiProvider: this.aiProvider,
                seenDisclosures: this.seenDisclosures
            };
            localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
        } catch {}
    }

    setFavorites(value: boolean) {
        this.showFavorites = value;
        this.#persist();
    }

    setRecent(value: boolean) {
        this.showRecent = value;
        this.#persist();
    }

    setSkipUngroupedTabs(value: boolean) {
        this.skipUngroupedTabs = value;
        this.#persist();
    }

    selectProvider(id: AiProviderId): boolean {
        this.aiProvider = id;
        const firstTime = !this.seenDisclosures.includes(id);
        if (firstTime) this.seenDisclosures = [...this.seenDisclosures, id];
        this.#persist();
        return firstTime;
    }
}

export const prefs = new PrefsStore();
