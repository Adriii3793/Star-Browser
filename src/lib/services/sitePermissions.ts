export type PermissionDecision = 'allow' | 'deny';

export interface SitePermissions {
    [origin: string]: { [kind: string]: PermissionDecision };
}

const STORAGE_KEY = 'star.sitePermissions';

export function loadSitePermissions(): SitePermissions {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        const parsed = raw ? JSON.parse(raw) : null;
        return parsed && typeof parsed === 'object' ? parsed : {};
    } catch {
        return {};
    }
}

function persist(all: SitePermissions) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
    } catch {}
}

export function originOf(uri: string): string | null {
    try {
        const url = new URL(uri);
        return url.protocol.startsWith('http') ? url.origin : null;
    } catch {
        return null;
    }
}

export function getDecision(origin: string, kind: string): PermissionDecision | null {
    const d = loadSitePermissions()[origin]?.[kind];
    return d === 'allow' || d === 'deny' ? d : null;
}

export function saveDecision(origin: string, kind: string, decision: PermissionDecision) {
    const all = loadSitePermissions();
    all[origin] = { ...all[origin], [kind]: decision };
    persist(all);
}

