export interface MemoryItem {id: string; text: string; createdAt: number;}
const KEY = 'ai_memory';

export const MAX_ITEMS = 200;
export const MAX_ITEM_CHARS = 500;
export const MAX_IMPORT_BYTES = 256 * 1024;


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

function load(): MemoryItem[] {
    try {
        if (typeof localStorage === 'undefined') return [];
        const raw = localStorage.getItem(KEY);
        const p = raw ? JSON.parse(raw) : [];
        return Array.isArray(p) ? p.filter((x) => typeof x?.text === 'string') : [];
    } catch { return [];}
}

class MemoryStore {
    items = $state<MemoryItem[]>(load());
    lastError = $state<string | null>(null);
    add(text: string): boolean {
        const t = sanitize(text).slice(0, MAX_ITEM_CHARS);
        if (!t) return false;
        if (this.items.length >= MAX_ITEMS) {
            this.lastError = `Memory is full (${MAX_ITEMS} items). Remove some first.`;
            return false;
        }
        if (this.items.some((m) => m.text.toLowerCase() === t.toLowerCase())) return false;
        this.items = [...this.items, {id: crypto.randomUUID(), text: t, createdAt: Date.now()}];
        return this.#persist();
    }

    remove(id: string) {
        this.items = this.items.filter((m) => m.id !== id);
        this.#persist();
    }
    clear() {this.items = []; this.#persist();}

    matches(phrase: string): MemoryItem[] {
        const q = sanitize(phrase).toLowerCase();
        if(!q) return [];
        return this.items.filter((m) => m.text.toLowerCase().includes(q));
    }

    forget(phrase: string): number {
        const doomed = new Set(this.matches(phrase).map((m) => m.id));
        if (doomed.size === 0) return 0;
        this.items = this.items.filter((m) => !doomed.has(m.id));
        this.#persist();
        return doomed.size;
    }
    toPromptBlock(): string {
        if (this.items.length === 0) return '';
        const lines = this.items.map((m) => `- ${m.text}`).join('\n');
        return `Facts the user asked you to remember. Use them when relevant:\n${lines}`;
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

            if (seen.has(t.toLowerCase())) {report.duplicates++; continue;}
            seen.add(t.toLowerCase());
            staged.push({id: crypto.randomUUID(), text: t,createdAt: Date.now()});
        }

        this.items = mode === 'replace' ? staged : [...this.items, ...staged];
        this.#persist();
        report.added = staged.length;
        return report;
    }

    #persist(): boolean {
        try {
            if (typeof localStorage === 'undefined') return false;
            localStorage.setItem(KEY, JSON.stringify(this.items));
            this.lastError = null;
            return true;
        } catch {
            this.lastError = 'Could not save memory: storage is full';
            return false;
        }
    }

}

export const memory = new MemoryStore();