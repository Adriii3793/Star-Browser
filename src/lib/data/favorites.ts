export interface Favorite {
    id: string;
    title: string;
    url: string;
}

export function favoriteKey(raw: string): string {
    const trimmed = raw.trim();
    if (!trimmed) return '';
    try {
        const u = new URL(/^[a-z][a-z0-9+.-]*:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`);
        const host = u.hostname.toLowerCase().replace(/^www\./, '');
        const path = u.pathname === '/' ? '' : u.pathname.replace(/\/+$/, '');
        return `${u.protocol.toLowerCase()}//${host}${u.port ? `:${u.port}` : ''}${path}${u.search}`;
    } catch {
        return trimmed.toLowerCase();
    }
}

export function sameFavorite(a: string, b: string): boolean {
    const key = favoriteKey(a);
    return key !== '' && key === favoriteKey(b);
}

export const defaultFavorites: Favorite[] = [
    { id: 'apple', title: 'Apple', url: 'https://www.apple.com' },
    { id: 'google', title: 'Google', url: 'https://www.google.com' },
    { id: 'youtube', title: 'YouTube', url: 'https://www.youtube.com' },
    { id: 'github', title: 'GitHub', url: 'https://www.github.com' },
    { id: 'wikipedia', title: 'Wikipedia', url: 'https://www.wikipedia.org' },
    { id: 'reddit', title: 'Reddit', url: 'https://www.reddit.com' },
    { id: 'amazon', title: 'Amazon', url: 'https://www.amazon.com' },
    { id: 'x', title: 'X', url: 'https://x.com' }
];
