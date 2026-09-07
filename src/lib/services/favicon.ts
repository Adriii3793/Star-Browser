export function faviconSources(rawUrl: string): string[] {
    const origin = originOf(rawUrl);
    if (!origin) return [];
    return [
        `${origin}/favicon.ico`,
        `${origin}/favicon.png`,
        `${origin}/apple-touch-icon.png`
    ];
}

export function faviconCandidates(rawUrl: string, declared?: string | null): string[] {
    const guesses = faviconSources(rawUrl);
    return declared ? [declared, ...guesses] : guesses;
}

export function faviconKey(candidates: string[]): string {
    return candidates.join('\n');
}

export interface FaviconProgress {
    key: string;
    step: number;
}

export function faviconStep(candidates: string[], progress: FaviconProgress | null): number {
    return progress && progress.key === faviconKey(candidates) ? progress.step : 0;
}

export function originOf(rawUrl: string): string {
    const parsed = parse(rawUrl);
    return parsed ? parsed.origin : '';
}

function hostOf(rawUrl: string): string {
    const parsed = parse(rawUrl);
    return parsed ? parsed.hostname : '';
}

function parse(rawUrl: string): URL | null {
    const trimmed = rawUrl.trim();
    if (!trimmed) return null;
    try {
        const u = new URL(/^[a-z][a-z0-9+.-]*:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`);
        return u.protocol === 'http:' || u.protocol === 'https:' ? u : null;
    } catch {
        return null;
    }
}

export function domainOf(rawUrl: string): string {
    return hostOf(rawUrl).replace(/^www\./, '') || rawUrl;
}
