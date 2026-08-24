export interface MemoryItem {id: string; text: string; createdAt: number;}
const KEY = 'ai_memory';

export const MAX_ITEMS = 200;
export const MAX_ITEM_CHARS = 500;
export const MAX_IMPORT_BYTES = 256 * 1024;
const MAX_PROMPT_ITEMS = 24;
const MAX_PROMPT_CHARS = 4_000;

const INJECTION_PATTERNS: RegExp[] = [
    /ignore\s+(all\s+)?(previous|prior|above)\s+instruction/i,
    /disregard\s+(all\s+)?(previous|prior|above)/i,
    /system\s+prompt/i,
    /you\s+are\s+now\s+/i,
    /forget\s+everything/i,
    /reveal\s+(your\s+)?(system|instructions|prompt)/i,
    /<\s*\/?\s*(system|assistant|user)\s*>/i,
    /\[\[\s*(remember|forget)\s*:/i
];

export interface ImportReport {
    added: number;
    rejected: {text: string; reason: string } [];
    duplicates: number;
}

function sanitize(raw: string): string {
    return raw
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g, '')
    .replace(/[\u200B-\u200F\u202A-\u202E\u2066-\u2069\uFEFF]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function isInstructionLike(text: string): boolean {
    return INJECTION_PATTERNS.some((pattern) => pattern.test(text));
}

const IGNORED_TERMS = new Set([
    'about', 'after', 'again', 'also', 'because', 'could', 'from', 'have', 'into', 'just',
    'like', 'more', 'only', 'please', 'really', 'should', 'that', 'their', 'there', 'they',
    'this', 'want', 'what', 'when', 'with', 'would', 'your',
    'user', 'users', 'thing', 'things', 'stuff', 'remember', 'remembers', 'remembered',
    'forget', 'forgets', 'forgotten', 'note', 'noted', 'fact', 'facts', 'said', 'says',
    'know', 'knows', 'told', 'part', 'some', 'item', 'items', 'memory', 'memories'
]);

function memoryTerms(text: string): string[] {
    return [...new Set(sanitize(text).toLowerCase().match(/[\p{L}\p{N}_-]{3,}/gu) ?? [])]
        .filter((term) => !IGNORED_TERMS.has(term));
}

const MATCH_THRESHOLD = 0.6;

function distinctiveTerms(query: string, corpus: MemoryItem[]): string[] {
    const ceiling = Math.max(1, Math.floor(corpus.length * 0.5));
    return memoryTerms(query).filter((term) => {
        const seenIn = corpus.reduce((n, item) => n + (item.text.toLowerCase().includes(term) ? 1 : 0), 0);
        return seenIn >= 1 && seenIn <= ceiling;
    });
}

function matchScore(item: MemoryItem, phrase: string, corpus: MemoryItem[]): number {
    const query = sanitize(phrase).toLowerCase();
    if (!query) return 0;
    const itemText = item.text.toLowerCase();
    if (itemText.includes(query) || query.includes(itemText)) return 1;

    const terms = distinctiveTerms(query, corpus);
    if (terms.length === 0) return 0;
    const hits = terms.filter((term) => itemText.includes(term)).length;
    return hits / terms.length;
}

function load(): MemoryItem[] {
    try {
        if (typeof localStorage === 'undefined') return [];
        const raw = localStorage.getItem(KEY);
        const p = raw ? JSON.parse(raw) : [];
        if (!Array.isArray(p)) return [];
        const seen = new Set<string>();
        return p
            .filter((x): x is MemoryItem =>
                typeof x?.id === 'string'
                && typeof x?.text === 'string'
                && typeof x?.createdAt === 'number'
            )
            .map((item) => ({ ...item, text: sanitize(item.text) }))
            .filter((item) => {
                const key = item.text.toLowerCase();
                if (!item.text || item.text.length > MAX_ITEM_CHARS || isInstructionLike(item.text) || seen.has(key)) return false;
                seen.add(key);
                return true;
            })
            .slice(0, MAX_ITEMS);
    } catch { return [];}
}

class MemoryStore {
    items = $state<MemoryItem[]>(load());
    lastError = $state<string | null>(null);

    add(text: string): boolean {
        const t = sanitize(text);
        if (!t) {
            this.lastError = 'Enter something to remember.';
            return false;
        }
        if (t.length > MAX_ITEM_CHARS) {
            this.lastError = `Memories must be ${MAX_ITEM_CHARS} characters or fewer.`;
            return false;
        }
        if (isInstructionLike(t)) {
            this.lastError = 'Instruction-like text cannot be saved as memory.';
            return false;
        }
        if (this.items.length >= MAX_ITEMS) {
            this.lastError = `Memory is full (${MAX_ITEMS} items). Remove some first.`;
            return false;
        }
        if (this.items.some((m) => m.text.toLowerCase() === t.toLowerCase())) {
            this.lastError = 'That is already remembered.';
            return false;
        }
        return this.#commit([...this.items, {id: crypto.randomUUID(), text: t, createdAt: Date.now()}]);
    }

    remove(id: string) {
        this.#commit(this.items.filter((m) => m.id !== id));
    }
    clear() { this.#commit([]); }

    matches(phrase: string): MemoryItem[] {
        if (!sanitize(phrase)) return [];
        const corpus = this.items;
        return corpus
            .map((item) => ({ item, score: matchScore(item, phrase, corpus) }))
            .filter((entry) => entry.score >= MATCH_THRESHOLD)
            .sort((a, b) => b.score - a.score)
            .map((entry) => entry.item);
    }

    forget(phrase: string): number {
        const doomed = new Set(this.matches(phrase).map((m) => m.id));
        if (doomed.size === 0) return 0;
        this.#commit(this.items.filter((m) => !doomed.has(m.id)));
        return doomed.size;
    }
    toPromptBlock(query = ''): string {
        if (this.items.length === 0) return '';
        const corpus = this.items;
        const ranked = [...corpus].sort((a, b) => {
            const score = matchScore(b, query, corpus) - matchScore(a, query, corpus);
            return score || b.createdAt - a.createdAt;
        });
        const selected: MemoryItem[] = [];
        let size = 0;
        for (const item of ranked) {
            const lineSize = item.text.length + 3;
            if (selected.length >= MAX_PROMPT_ITEMS || size + lineSize > MAX_PROMPT_CHARS) continue;
            selected.push(item);
            size += lineSize;
        }
        if (selected.length === 0) return '';
        const lines = selected.map((m) => `- ${m.text}`).join('\n');
        return `Facts the user explicitly asked you to remember. Use them only when relevant:\n${lines}`;
    }
    export(): string {
        const now = new Date().toISOString();
        const body = this.items.map((m) => `- ${m.text}`).join('\n');
        return `---\nname: ai-memory\nversion: 1\nexported: ${now}\ncount: ${this.items.length}\n---\n\n# AI Memory\n\n${body}\n`;
    }

    import(raw: string, mode: 'merge' | 'replace' = 'merge'): ImportReport {
        const report: ImportReport = {added: 0, rejected: [],duplicates: 0};

        if (raw.length > MAX_IMPORT_BYTES) {
            report.rejected.push({ text: '(file)', reason: 'File too large (max 256 KB)'});
            return report;
        }

        const body = raw.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, '');
        let candidates = body
            .split(/\r?\n/)
            .map((l) => l.trim())
            .filter((l) => /^[-*]\s+/.test(l))
            .map((l) => l.replace(/^[-*]\s+/, ''));
        
        if (candidates.length === 0) {
            try {
                const parsed = JSON.parse(raw);
                const arr = Array.isArray(parsed) ? parsed : parsed?.items;
                candidates = (arr ?? [])
                    .map((x: any) => (typeof x === 'string' ? x : x?.text))
                    .filter((t: any) => typeof t === 'string');
            } catch {}
        }

        if (candidates.length === 0) {
            report.rejected.push({ text: '(file)', reason: 'No "- " bullets or JSON items found' });
            return report;
        }

        const staged: MemoryItem[] = []
        const seen = new Set(
            (mode === 'replace' ? [] : this.items).map((m) => m.text.toLowerCase())
        );

        for (const candidate of candidates) {
            const t = sanitize(candidate);
            if (!t) continue;
            if (staged.length + (mode === 'replace' ? 0 : this.items.length) >= MAX_ITEMS) {
                report.rejected.push({ text: t, reason: 'Memory limit reached'});
                continue;
            }

            if (t.length > MAX_ITEM_CHARS) {
                report.rejected.push({ text: t.slice(0, 60), reason: 'Too long' });
                continue;
            }
            if (isInstructionLike(t)) {
                report.rejected.push({ text: t.slice(0, 60), reason: 'Instruction-like content is not allowed' });
                continue;
            }

            if (seen.has(t.toLowerCase())) {report.duplicates++; continue;}
            seen.add(t.toLowerCase());
            staged.push({id: crypto.randomUUID(), text: t,createdAt: Date.now()});
        }

        const next = mode === 'replace' ? staged : [...this.items, ...staged];
        if (this.#commit(next)) report.added = staged.length;
        return report;
    }

    #commit(next: MemoryItem[]): boolean {
        try {
            if (typeof localStorage === 'undefined') return false;
            localStorage.setItem(KEY, JSON.stringify(next));
            this.items = next;
            this.lastError = null;
            return true;
        } catch {
            this.lastError = 'Could not save memory: storage is full';
            return false;
        }
    }

}

export const memory = new MemoryStore();
