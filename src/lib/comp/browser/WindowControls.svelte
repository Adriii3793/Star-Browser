<script lang="ts">
	import { onMount, onDestroy, untrack } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { platform as detectPlatform } from '@tauri-apps/plugin-os';
	import { windowChrome } from '$lib/stores/windowChrome.svelte';

	type Platform = 'macos' | 'windows' | 'linux';

	interface Props {
		platform?: Platform;
		background?: string;
		maximizable?: boolean;
		onminimize?: () => void;
		onmaximize?: (maximized: boolean) => void;
		onclose?: () => void;
	}

	let { platform, background, maximizable = true, onminimize, onmaximize, onclose }: Props = $props();

	let os = $state<Platform>(untrack(() => platform) ?? 'windows');
	let maximized = $derived(windowChrome.maximized);

	let root = $state<HTMLElement | undefined>(undefined);
	let mounted = $state(false);
	let tone = $state<'light' | 'dark'>('light');
	let observer: MutationObserver | undefined;

	type Rgba = { r: number; g: number; b: number; a: number };

	function parseColor(value: string): Rgba | null {
		if (!value) return null;
		const direct = value.match(/^rgba?\(([^)]+)\)$/i);
		const parts = direct ? direct[1].split(/[\s,/]+/).filter(Boolean) : normalize(value);
		if (!parts || parts.length < 3) return null;
		const [r, g, b, a] = parts;
		return {
			r: parseFloat(r),
			g: parseFloat(g),
			b: parseFloat(b),
			a: a === undefined ? 1 : parseFloat(a)
		};
	}

	function normalize(value: string): string[] | null {
		try {
			const ctx = document.createElement('canvas').getContext('2d');
			if (!ctx) return null;
			ctx.fillStyle = '#000';
			ctx.fillStyle = value;
			const resolved = ctx.fillStyle as string;
			if (resolved.startsWith('#')) {
				const hex = resolved.slice(1);
				const full =
					hex.length === 3
						? hex
								.split('')
								.map((c) => c + c)
								.join('')
						: hex;
				return [
					String(parseInt(full.slice(0, 2), 16)),
					String(parseInt(full.slice(2, 4), 16)),
					String(parseInt(full.slice(4, 6), 16))
				];
			}
			const m = resolved.match(/^rgba?\(([^)]+)\)$/i);
			return m ? m[1].split(/[\s,/]+/).filter(Boolean) : null;
		} catch {
			return null;
		}
	}

	function effectiveBackground(el: HTMLElement | null): Rgba {
		const layers: Rgba[] = [];
		let node: HTMLElement | null = el;

		while (node) {
			const color = parseColor(getComputedStyle(node).backgroundColor);
			if (color && color.a > 0) {
				layers.push(color);
				if (color.a === 1) break;
			}
			node = node.parentElement;
		}

		let base: Rgba = { r: 255, g: 255, b: 255, a: 1 };
		for (let i = layers.length - 1; i >= 0; i--) {
			const top = layers[i];
			base = {
				r: top.r * top.a + base.r * (1 - top.a),
				g: top.g * top.a + base.g * (1 - top.a),
				b: top.b * top.a + base.b * (1 - top.a),
				a: 1
			};
		}
		return base;
	}

	function luminance({ r, g, b }: Rgba): number {
		const [lr, lg, lb] = [r, g, b].map((channel) => {
			const c = channel / 255;
			return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
		});
		return 0.2126 * lr + 0.7152 * lg + 0.0722 * lb;
	}

	function updateTone() {
		if (!root) return;
		const bg = (background && parseColor(background)) || effectiveBackground(root);
		const l = luminance(bg);
		const againstWhite = 1.05 / (l + 0.05);
		const againstBlack = (l + 0.05) / 0.05;
		tone = againstWhite > againstBlack ? 'dark' : 'light';
	}

	$effect(() => {
		if (mounted) {
			background;
			updateTone();
		}
	});

	$effect(() => {
		if (platform) {
			os = platform;
		}
	});

	function resolvePlatform(): Platform {
		if (platform) return platform;
		try {
			const p = detectPlatform();
			return p === 'macos' ? 'macos' : p === 'linux' ? 'linux' : 'windows';
		} catch {
			const ua = navigator.userAgent;
			if (/Macintosh|Mac OS X/.test(ua)) return 'macos';
			if (/Linux/.test(ua) && !/Android/.test(ua)) return 'linux';
			return 'windows';
		}
	}

	function appWindow() {
		try {
			return getCurrentWindow();
		} catch {
			return null;
		}
	}

	async function minimize() {
		onminimize?.();
		await appWindow()?.minimize().catch(() => {});
	}

	async function toggleMaximize() {
		await windowChrome.toggle();
		onmaximize?.(windowChrome.maximized);
	}

	async function close() {
		onclose?.();
		const win = appWindow();
		if (!win) return;
		const force = setTimeout(() => {
			void win.destroy().catch(() => {});
		}, 1500);
		try {
			await win.close();
		} catch {
			clearTimeout(force);
			await win.destroy().catch(() => {});
		}
	}

	onMount(() => {
		os = resolvePlatform();
		mounted = true;
		updateTone();

		observer = new MutationObserver(() => updateTone());
		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['style', 'class']
		});
		if (root?.parentElement) {
			observer.observe(root.parentElement, {
				attributes: true,
				attributeFilter: ['style', 'class']
			});
		}

		windowChrome.init();
	});

	onDestroy(() => {
		observer?.disconnect();
	});
</script>

{#if os === 'macos'}
	<div bind:this={root} class="controls macos" role="group" aria-label="Window controls">
		<button type="button" class="traffic close" aria-label="Close" onclick={close}>
			<svg viewBox="0 0 12 12" aria-hidden="true">
				<path d="M3.35 3.35 L8.65 8.65 M8.65 3.35 L3.35 8.65" />
			</svg>
		</button>

		<button type="button" class="traffic minimize" aria-label="Minimize" onclick={minimize}>
			<svg viewBox="0 0 12 12" aria-hidden="true">
				<path d="M3 6 H9" />
			</svg>
		</button>

		{#if maximizable}
			<button type="button" class="traffic maximize" aria-label="Zoom" onclick={() => void toggleMaximize()}>
				<svg viewBox="0 0 12 12" aria-hidden="true" class="filled">
					<path d="M3.1 3.1 H6.9 L3.1 6.9 Z" />
					<path d="M8.9 8.9 H5.1 L8.9 5.1 Z" />
				</svg>
			</button>
		{/if}
	</div>
{:else}
	<div
		bind:this={root}
		class="controls win"
		class:on-light={tone === 'light'}
		class:on-dark={tone === 'dark'}
		role="group"
		aria-label="Window controls"
	>
		<button type="button" class="cell" aria-label="Minimize" onclick={minimize}>
			<svg viewBox="0 0 10 10" aria-hidden="true">
				<path d="M0 5 H10" />
			</svg>
		</button>

		{#if maximizable}
			<button
				type="button"
				class="cell"
				aria-label={maximized ? 'Restore' : 'Maximize'}
				onclick={() => void toggleMaximize()}
			>
				{#if maximized}
					<svg viewBox="0 0 10 10" aria-hidden="true">
						<rect x="0.5" y="2.5" width="7" height="7" />
						<path d="M2.5 2.5 V0.5 H9.5 V7.5 H7.5" />
					</svg>
				{:else}
					<svg viewBox="0 0 10 10" aria-hidden="true">
						<rect x="0.5" y="0.5" width="9" height="9" />
					</svg>
				{/if}
			</button>
		{/if}

		<button type="button" class="cell close" aria-label="Close" onclick={close}>
			<svg viewBox="0 0 10 10" aria-hidden="true">
				<path d="M0.5 0.5 L9.5 9.5 M9.5 0.5 L0.5 9.5" />
			</svg>
		</button>
	</div>
{/if}

<style>
	.controls {
		display: flex;
		align-items: stretch;
		align-self: stretch;
		min-height: 32px;
		flex-shrink: 0;
	}

	button {
		-webkit-app-region: no-drag;
		cursor: default;
	}

	.macos {
		align-items: center;
		gap: 8px;
		padding: 0 8px 0 12px;
	}

	.traffic {
		width: 12px;
		height: 12px;
		padding: 0;
		border-radius: 50%;
		border: 0.5px solid rgba(0, 0, 0, 0.12);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.traffic.close {
		background: #ff544d;
	}
	.traffic.minimize {
		background: #ffbd2e;
	}
	.traffic.maximize {
		background: #28c93f;
	}

	.traffic svg {
		width: 100%;
		height: 100%;
		opacity: 0;
		transition: opacity 0.12s ease;
		pointer-events: none;
	}
	.traffic svg path {
		fill: none;
		stroke: rgba(0, 0, 0, 0.55);
		stroke-width: 1.1;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.traffic svg.filled path {
		fill: rgba(0, 0, 0, 0.55);
		stroke: none;
	}

	.macos:hover .traffic svg {
		opacity: 1;
	}

	.traffic.close:active {
		background: #bf403a;
	}
	.traffic.minimize:active {
		background: #bf9122;
	}
	.traffic.maximize:active {
		background: #1e9930;
	}

	.win {
		--fg: var(--controls-fg, var(--auto-fg));
	}

	.win.on-light {
		--auto-fg: #1a1a1a;
		--hover: rgba(0, 0, 0, 0.07);
		--active: rgba(0, 0, 0, 0.14);
	}

	.win.on-dark {
		--auto-fg: #ffffff;
		--hover: rgba(255, 255, 255, 0.09);
		--active: rgba(255, 255, 255, 0.18);
	}

	.cell {
		width: 46px;
		height: auto;
		align-self: stretch;
		padding: 0;
		border: 0;
		background: transparent;
		color: var(--fg);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		transition:
			background-color 0.12s ease,
			color 0.12s ease;
	}

	.cell svg {
		width: 10px;
		height: 10px;
	}
	.cell svg path,
	.cell svg rect {
		fill: none;
		stroke: currentColor;
		stroke-width: 1;
	}

	.cell:hover {
		background: var(--hover);
	}
	.cell:active {
		background: var(--active);
	}

	.cell.close:hover {
		background: #c42b1c;
		color: #ffffff;
	}
	.cell.close:active {
		background: rgba(196, 43, 28, 0.85);
	}

	@media (prefers-reduced-motion: reduce) {
		.cell,
		.traffic svg {
			transition: none;
		}
	}
</style>
