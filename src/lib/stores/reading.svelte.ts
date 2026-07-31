import { fetchPageContext } from '$lib/services/ai';

export interface ReadPage {
    url: string;
    title: string;
    text: string;
    at: number;
}

const STORAGE_KEY = 'star.readingLog';
const MAX_PAGES = 12;
const MAX_TEXT = 3500;

class ReadingStore {
    pages = $state<ReadPage[]>([]);

    #loaded = false;
    #pending = new Set<string>();

    init() {
        if (this.#loaded) return;
        this.#loaded = true;
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (!raw) return;
            const parsed = JSON.parse(raw);
            if (Array.isArray(parsed)) {
                this.pages = parsed
                    .filter(
                        (p) =>
                            typeof p?.url === 'string' &&
                            typeof p?.title === 'string' &&
                            typeof p?.text === 'string'
                    )
                    .slice(-MAX_PAGES);
            }
        } catch {}
    }

    async capture(url: string) {
        if (!/^https?:/i.test(url)) return;
        if (this.#pending.has(url) || this.pages.some((p) => p.url === url)) return;
        this.#pending.add(url);
        try {
            const page = await fetchPageContext(url);
            if (!page?.text?.trim()) return;
            const entry: ReadPage = {
                url,
                title: page.title?.trim() || url,
                text: page.text.slice(0, MAX_TEXT),
                at: Date.now()
            };
            this.pages = [...this.pages.filter((p) => p.url !== url), entry].slice(-MAX_PAGES);
            this.#persist();
        } catch {
        } finally {
            this.#pending.delete(url);
        }
    }

    toPromptBlock(excludeUrl?: string | null): string | null {
        const items = this.pages.filter((p) => p.url !== excludeUrl);
        if (!items.length) return null;
        const body = items
            .map(
                (p) =>
                    `<page_content url="${p.url}" title="${p.title.replaceAll('"', "'")}">\n${p.text}\n</page_content>`
            )
            .join('\n');
        return (
            'Pages the user has read recently in this browser, oldest first. Use them as ' +
            'context when the user asks about "what I read", asks to compare or combine ' +
            'sources, or refers to earlier reading.\n' +
            body
        );
    }

    clear() {
        this.pages = [];
        this.#persist();
    }

    #persist() {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify($state.snapshot(this.pages)));
        } catch {}
    }
}

export const reading = new ReadingStore();
