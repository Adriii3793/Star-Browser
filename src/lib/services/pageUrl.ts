/**
 * Whether the live document is the page the tab was sent to. Sites often append their own
 * parameters on arrival (Google adds `sei=`, trackers add `utm_*`), so every parameter of the
 * requested address must be present with the same value, but extra ones are fine. A different
 * search (`q=dogs` vs `q=cats`) or the previous page still fails.
 */
export function showsPage(live: string, requested: string): boolean {
    try {
        const x = new URL(live);
        const y = new URL(requested);
        if (x.origin !== y.origin || trimPath(x) !== trimPath(y)) return false;
        for (const [key, value] of y.searchParams) {
            if (!x.searchParams.getAll(key).includes(value)) return false;
        }
        return true;
    } catch {
        return live === requested;
    }
}

/** Two reads of one document are "settled" once its text stops changing much. */
export function settled(previous: string | null, current: string): boolean {
    if (previous === null) return false;
    const a = previous.length;
    const b = current.length;
    return Math.abs(a - b) <= Math.max(40, Math.max(a, b) * 0.05);
}

function trimPath(u: URL): string {
    return u.pathname.replace(/\/+$/, '') || '/';
}
