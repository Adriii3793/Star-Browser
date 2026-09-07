import { setSurfaceClip } from './webview';
import { paintsAnything } from './paint';

export function clippedSurface(): boolean {
	return (
		typeof window !== 'undefined' &&
		(window as Window & { __starClippedSurface?: boolean }).__starClippedSurface === true
	);
}


export interface SurfaceClipPart {
	x: number;
	y: number;
	width: number;
	height: number;
	radius: number;
}

const CLIPPED_CARD_ATTR = 'data-star-clipped-card';

const CARD_SEARCH_DEPTH = 2;

function coversViewport(box: DOMRect): boolean {
	return (
		box.left <= 1 &&
		box.top <= 1 &&
		box.right >= window.innerWidth - 1 &&
		box.bottom >= window.innerHeight - 1
	);
}

function paints(element: Element): boolean {
	return paintsAnything(getComputedStyle(element));
}

function paintedParts(root: Element, depth = CARD_SEARCH_DEPTH): Element[] {
	const parts: Element[] = [];
	for (const child of Array.from(root.children)) {
		const box = child.getBoundingClientRect();
		if (box.width <= 0 || box.height <= 0) continue;
		if (coversViewport(box) || !paints(child)) {
			if (depth > 0 && child.children.length > 0) parts.push(...paintedParts(child, depth - 1));
			continue;
		}
		parts.push(child);
	}
	return parts;
}

export function measureSurfaceClip(root: Element | null | undefined): SurfaceClipPart[] {
	if (!root || typeof window === 'undefined') return [];
	return paintedParts(root)
		.map((part) => {
			const box = part.getBoundingClientRect();
			return {
				x: box.left,
				y: box.top,
				width: box.width,
				height: box.height,
				radius: cardRadius(part)
			};
		})
		.filter((part) => part.width > 0 && part.height > 0);
}

function cardRadius(card: Element): number {
	return Number.parseFloat(getComputedStyle(card).borderTopLeftRadius) || 0;
}

const SETTLE_DELAYS_MS = [80, 200, 420];

export function watchSurfaceClip(
	surface: 'menu' | 'overlay',
	getRoot: () => Element | null | undefined
): { sync: () => void; invalidate: () => void; destroy: () => void } {
	if (!clippedSurface() || typeof window === 'undefined') {
		return { sync: () => {}, invalidate: () => {}, destroy: () => {} };
	}

	let last = '';
	let frame = 0;
	let observed: Element[] = [];
	let settleTimers: ReturnType<typeof setTimeout>[] = [];

	const push = () => {
		frame = 0;
		const root = getRoot();
		const elements = root ? paintedParts(root) : [];
		const parts = measureSurfaceClip(root);
		const key = parts
			.map((part) =>
				[part.x, part.y, part.width, part.height, part.radius]
					.map((value) => Math.round(value))
					.join(',')
			)
			.join('|');
		if (key !== last) {
			last = key;
			void setSurfaceClip(surface, parts).catch(() => {});
		}

		if (elements.length !== observed.length || elements.some((el, i) => el !== observed[i])) {
			sizes.disconnect();
			for (const element of observed) element.removeAttribute(CLIPPED_CARD_ATTR);
			for (const element of elements) {
				sizes.observe(element);
				element.setAttribute(CLIPPED_CARD_ATTR, '');
			}
			observed = elements;
		}
	};

	const sync = () => {
		if (frame) return;
		frame = requestAnimationFrame(push);
	};

	const sizes = new ResizeObserver(sync);
	const contents = new MutationObserver(sync);
	window.addEventListener('resize', sync, { passive: true });

	const clearSettle = () => {
		for (const timer of settleTimers) clearTimeout(timer);
		settleTimers = [];
	};

	return {
		sync,
		invalidate() {
			last = '';
			const root = getRoot();
			if (root) {
				contents.disconnect();
				contents.observe(root, { childList: true, subtree: true, attributes: true });
			}
			sync();
			clearSettle();
			settleTimers = SETTLE_DELAYS_MS.map((delay) => setTimeout(sync, delay));
		},
		destroy() {
			if (frame) cancelAnimationFrame(frame);
			clearSettle();
			sizes.disconnect();
			contents.disconnect();
			for (const element of observed) element.removeAttribute(CLIPPED_CARD_ATTR);
			observed = [];
			window.removeEventListener('resize', sync);
		}
	};
}
