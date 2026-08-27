export function faviconSources(rawUrl: string): string[] {
    const host = hostOf(rawUrl);
    if (!host) return [];
    return [
        `https://${host}/favicon.ico`,
        `https://icons.duckduckgo.com/ip3/${host}.ico`
    ];
}

export function hostOf(rawUrl: string): string {
    const trimmed = rawUrl.trim();
    if (!trimmed) return '';
    try {
        const u = new URL(/^[a-z][a-z0-9+.-]*:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`);
        return u.protocol === 'http:' || u.protocol === 'https:' ? u.hostname : '';
    } catch {
        return '';
    }
}

export function domainOf(rawUrl: string): string {
    return hostOf(rawUrl).replace(/^www\./, '') || rawUrl;
}
