const STORAGE_KEY = 'star.home.prefs';

export type AiProviderId = 'nemotron' | 'gemma';

export interface AiProvider {
    id: AiProviderId;
    model: string;
    name: string;
    vendor: string;
    modalities: string;
    disclosure: string;
}

export const AI_PROVIDERS: AiProvider[] = [
    {
        id: 'nemotron',
        model: 'nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free',
        name: 'Nemotron 3 Nano Omni',
        vendor: 'NVIDIA',
        modalities: 'Text, image, video and audio in — text out',
        disclosure: 'NVIDIA uses your data to improve their products.'
    },
    {
        id: 'gemma',
        model: 'google/gemma-4-26b-a4b-it:free',
        name: 'Gemma 4',
        vendor: 'Google',
        modalities: 'Text, image and video in — text out',
        disclosure: 'Google does not retain your data.'
    }
];

export const DEFAULT_PROVIDER: AiProviderId = 'nemotron';

export function providerById(id: AiProviderId): AiProvider {
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
