export const DISMISS_GRACE_MS = 200;

export const REOPEN_GUARD_MS = 250;

export interface FocusedElement {
	tagName: string;
	type?: string;
}

export function opensNativeDialog(element: FocusedElement | null | undefined): boolean {
	if (!element || element.tagName.toUpperCase() !== 'INPUT') return false;
	const type = (element.type ?? '').toLowerCase();
	return type === 'file' || type === 'color';
}

export interface BlurDismissal {
	shown: boolean;
	visible: boolean;
	shownAt: number;
	now: number;
	activeElement?: FocusedElement | null;
	graceMs?: number;
}

export function shouldDismissOnBlur(input: BlurDismissal): boolean {
	if (!input.shown || !input.visible) return false;
	if (input.now - input.shownAt < (input.graceMs ?? DISMISS_GRACE_MS)) return false;
	return !opensNativeDialog(input.activeElement);
}

export interface PointerDismissal {
	shown: boolean;
	shownAt: number;
	now: number;
	graceMs?: number;
}

export function shouldDismissOnPointer(input: PointerDismissal): boolean {
	if (!input.shown) return false;
	return input.now - input.shownAt >= (input.graceMs ?? DISMISS_GRACE_MS);
}

export function isStaleDismissal(seq: number | undefined, currentSeq: number): boolean {
	return seq !== undefined && seq !== currentSeq;
}

export function suppressesReopen(
	lastFocusLossDismissAt: number,
	now: number,
	guardMs: number = REOPEN_GUARD_MS
): boolean {
	return lastFocusLossDismissAt > 0 && now - lastFocusLossDismissAt < guardMs;
}
