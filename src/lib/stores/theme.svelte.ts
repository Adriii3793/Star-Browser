export interface Theme {
    id: string;
    name: string;
    bg: string;
    surface: string;
    accent: string;
    image?: string | null;
}

export const PRESET_THEMES: Theme[] = [
    { id: 'light', name: 'Light', bg: '#faf7f7', surface: '#ffffff', accent: '#e8734a' },
    { id: 'dark', name: 'Dark', bg: '#1c1917', surface: '#292524', accent: '#e8734a' },
    { id: 'sand', name: 'Sand', bg: '#f2e8dc', surface: '#fdf9f4', accent: '#b8763f' }
];

export function luminance(hex: string): number {
    const h = hex.replace('#', '').trim();
    const full = h.length === 3 ? h.split('').map((c) => c + c).join('') : h;
    const [r, g, b] = [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16) / 255);
    if ([r, g, b].some(Number.isNaN)) return 0.5;
    const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

export function readableText(bgHex: string): string {
    return luminance(bgHex) > 0.5 ? '#1c1917' : '#ffffff';
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
