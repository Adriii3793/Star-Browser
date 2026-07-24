export interface Favorite {
    id: string;
    title: string;
    url: string;
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
