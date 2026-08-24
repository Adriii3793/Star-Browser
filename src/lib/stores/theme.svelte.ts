export interface Theme {
    id: string;
    name: string;
    bg: string;
    surface: string;
    accent: string;
    image?: string | null;
}

export const PRESET_THEMES: Theme[] = [
    { id: 'light', name: 'Light', bg: '#faf7f7', surface: '#ffffff', accent: '#80A4D4' },
    { id: 'dark', name: 'Dark', bg: '#1c1917', surface: '#292524', accent: '#80A4D4' },
    { id: 'sand', name: 'Sand', bg: '#f2e8dc', surface: '#fdf9f4', accent: '#b8763f' }
];

export const SYSTEM_THEME: Theme = {
    id: 'system',
    name: 'System',
    bg: '#6b7280',
    surface: '#f8fafc',
    accent: '#80A4D4'
};

export function luminance(hex: string): number {
    const h = hex.replace('#', '').trim();
    const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
    const [r, g, b] = [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16) / 255);
    if ([r, g, b].some(Number.isNaN)) return 0.5;
    const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function readableText(bgHex: string): string {
    const l = luminance(bgHex);
    const withWhite = 1.05 / (l + 0.05);
    const withBlack = (l + 0.05) / 0.05;
    return withBlack >= withWhite ? '#1c1917' : '#ffffff';
}

export function imageLuminance(dataUrl: string): Promise<number> {
    return new Promise((resolve) => {
        const img = new Image();
        img.onload = () => {
            const c = document.createElement('canvas');
            c.width = 32;
            c.height = 32;
            const ctx = c.getContext('2d');
            if (!ctx) return resolve(0.5);
            try {
                ctx.drawImage(img, 0, 0, 32, 32);
                const { data } = ctx.getImageData(0, 0, 32, 32);
                let sum = 0;
                for (let i = 0; i < data.length; i += 4) {
                    sum += (0.2126 * data[i] + 0.7152 * data[i + 1] + 0.0722 * data[i + 2]) / 255;
                }
                resolve(sum / (data.length / 4));
            } catch {
                resolve(0.5);
            }
        };
        img.onerror = () => resolve(0.5);
        img.src = dataUrl;
    });
}

const STORAGE_KEY = 'star.theme';

function toRgb(hex: string): [number, number, number] {
    const h = hex.replace('#', '').trim();
    const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
    return [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16) || 0) as [number, number, number];
}

function toHex(rgb: [number, number, number]): string {
    return '#' + rgb.map((c) => Math.round(Math.max(0, Math.min(255, c))).toString(16).padStart(2, '0')).join('');
}

export function mix(a: string, b: string, amount: number): string {
    const [r1, g1, b1] = toRgb(a);
    const [r2, g2, b2] = toRgb(b);
    const t = Math.max(0, Math.min(1, amount));
    return toHex([r1 + (r2 - r1) * t, g1 + (g2 - g1) * t, b1 + (b2 - b1) * t]);
}

export function isDark(t: Theme): boolean {
    return luminance(t.surface) < 0.5;
}

export function themeVars(t: Theme): Record<string, string> {
    const dark = isDark(t);
    const text = readableText(t.surface);
    const onBg = readableText(t.bg);

    return {
        '--bg-chrome': t.bg,
        '--bg-page': t.surface,
        '--tab-active': t.surface,
        '--tab-hover': mix(t.bg, onBg, 0.07),
        '--field': mix(t.surface, text, 0.06),
        '--field-strong': mix(t.surface, text, 0.11),
        '--text': text,
        '--text-soft': mix(text, t.surface, 0.3),
        '--text-muted': mix(text, t.surface, 0.48),
        '--accent': t.accent,
        '--accent-hover': mix(t.accent, dark ? '#ffffff' : '#000000', 0.16),
        '--accent-contrast': readableText(t.accent),
        '--border': mix(t.surface, text, 0.14),
        '--border-strong': mix(t.surface, text, 0.24),
        '--hover': dark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.06)',
        '--success': dark ? '#5bd18d' : '#27875a',
        '--danger': dark ? '#ff7d70' : '#c0392b',
        '--overlay': dark ? 'rgba(0, 0, 0, 0.6)' : 'rgba(74, 58, 46, 0.28)',
        '--shadow': dark ? 'rgba(0, 0, 0, 0.55)' : 'rgba(74, 58, 46, 0.16)',
        'color-scheme': dark ? 'dark' : 'light'
    };
}

export function applyThemeVars(t: Theme, target?: HTMLElement) {
    if (typeof document === 'undefined') return;
    const root = target ?? document.documentElement;
    for (const [key, value] of Object.entries(themeVars(t))) {
        if (key === 'color-scheme') root.style.setProperty('color-scheme', value);
        else root.style.setProperty(key, value);
    }
    root.dataset.theme = isDark(t) ? 'dark' : 'light';
}

class ThemeStore {
    current = $state<Theme>(PRESET_THEMES[0]);
    preference = $state<string>('light');

    #loaded = false;
    #media: MediaQueryList | null = null;

    init() {
        if (this.#loaded) return;
        this.#loaded = true;
        this.#media = typeof window === 'undefined' ? null : window.matchMedia('(prefers-color-scheme: dark)');
        this.#media?.addEventListener('change', () => {
            if (this.preference !== 'system') return;
            this.current = this.#resolve('system');
            applyThemeVars(this.current);
        });

        const stored = this.#load();
        if (stored) this.#apply(stored, false);
        else applyThemeVars(this.current);
    }

    set(next: Theme | string) {
        this.#apply(next, true);
    }

    #apply(next: Theme | string, save: boolean) {
        this.preference = typeof next === 'string' ? next : next.id;
        this.current = this.#resolve(next);
        applyThemeVars(this.current);
        if (!save) return;
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify({ preference: this.preference, theme: next }));
        } catch {}
    }

    get dark(): boolean {
        return isDark(this.current);
    }

    #resolve(next: Theme | string): Theme {
        if (next === 'system') {
            const useDark = this.#media?.matches ?? false;
            return { ...(useDark ? PRESET_THEMES[1] : PRESET_THEMES[0]), id: 'system', name: 'System' };
        }
        if (typeof next === 'string') {
            return PRESET_THEMES.find((theme) => theme.id === next) ?? this.current;
        }
        return next;
    }

    #load(): Theme | string | null {
        try {
            const raw = localStorage.getItem(STORAGE_KEY);
            if (!raw) return null;
            const t = JSON.parse(raw);
            if (typeof t === 'string') return t;
            if (typeof t?.preference === 'string') {
                return t.preference === 'custom' && t.theme && typeof t.theme === 'object'
                    ? t.theme as Theme
                    : t.preference;
            }
            if (typeof t?.bg !== 'string' || typeof t?.surface !== 'string') return null;
            return t as Theme;
        } catch {
            return null;
        }
    }
}

export const theme = new ThemeStore();

export function parseTheme(raw: string): Theme | null {
    try {
        const t = JSON.parse(raw);
        const hex = /^#[0-9a-f]{3,8}$/i;
        if (typeof t?.name !== 'string') return null;
        if (![t.bg, t.surface, t.accent].every((c) => typeof c === 'string' && hex.test(c))) return null;
        if (t.image && !String(t.image).startsWith('data:image/')) return null;
        return {
            id: 'custom',
            name: String(t.name).slice(0, 40),
            bg: t.bg,
            surface: t.surface,
            accent: t.accent,
            image: t.image ?? null
        };
    } catch {
        return null;
    }
}
