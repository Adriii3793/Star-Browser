export interface HeldVisit {
	url: string;
}

export function heldVisitAddress(
	held: HeldVisit | undefined,
	reported: string,
	query: string | null
): string {
	return query !== null && held ? held.url : reported;
}

export function continuesNavigation(
	navigating: boolean,
	sinceActionMs: number,
	redirectWindowMs: number
): boolean {
	return navigating || sinceActionMs < redirectWindowMs;
}
