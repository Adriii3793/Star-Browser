<script lang="ts">
	import { tick } from 'svelte';
	import Tab from './Tab.svelte';

	interface TabData {
		id: string;
		title: string;
		favicon?: string;
		groupId?: string;
		muted?: boolean;
		audible?: boolean;
	}

	export interface TabGroup {
		id: string;
		name: string;
		color: string;
	}

	let {
		tabs,
		activeId,
		onselect,
		onclose,
		onnew,
		onreorder,
		groups = [],
		ongroupdrop,
		onaddtogroup,
		oneditgroup,
		onmute,
		onmenustate,
		menuResetToken
	}: {
		tabs: TabData[];
		activeId: string;
		onselect: (id: string) => void;
		onclose: (id: string) => void;
		onnew: () => void;
		onreorder?: (from: number, to: number) => void;
		groups?: TabGroup[];
		ongroupdrop?: (sourceId: string, targetId: string) => void;
		onaddtogroup?: (groupId: string) => void;
		oneditgroup?: (groupId: string, x: number, y: number) => void;
		onmute?: (tabId: string) => void;
		onmenustate?: (state: { x: number; y: number; tabId: string | null; grouped: boolean } | null) => void;
		menuResetToken?: number;
	} = $props();

	const DRAG_THRESHOLD = 5;
	const NUDGE = 0.12;
	const GROUP_DWELL_MS = 420;
	const SLIDE_MS = 190;
	const SLIDE_EASE = 'cubic-bezier(.2, .8, .2, 1)';
	const SLIDE_ID = 'star-tab-slide';

	let strip = $state<HTMLElement>();
	let dragIndex = $state<number | null>(null);
	let dragX = $state(0);
	let groupTargetIndex = $state<number | null>(null);
	let contextMenu = $state<{ x: number; y: number; tabId: string | null; grouped: boolean } | null>(null);

	let pendingIndex = 0;
	let pointerStartX = 0;
	let pointerX = 0;
	let dragOrigin = 0;
	let pinned = 0;
	let reordering = false;
	let dwellTimer: ReturnType<typeof setTimeout> | undefined;
	let dwellId: string | null = null;
	let layout = new Map<string, number>();

	type Run = { group: TabGroup | null; items: { tab: TabData; index: number }[] };
	let runs = $derived.by(() => {
		const out: Run[] = [];
		tabs.forEach((tab, index) => {
			const group = groups.find((g) => g.id === tab.groupId) ?? null;
			const last = out[out.length - 1];
			if (last && last.group?.id === group?.id) last.items.push({ tab, index });
			else out.push({ group, items: [{ tab, index }] });
		});
		return out;
	});

	function reducedMotion(): boolean {
		return typeof matchMedia !== 'undefined' && matchMedia('(prefers-reduced-motion: reduce)').matches;
	}

	function tabEls(): HTMLElement[] {
		return [...(strip?.querySelectorAll<HTMLElement>('[data-tab-index]') ?? [])];
	}

	function draggedEl(): HTMLElement | undefined {
		if (dragIndex === null) return undefined;
		return strip?.querySelector<HTMLElement>(`[data-tab-index="${dragIndex}"]`) ?? undefined;
	}

	function cancelSlide(el: HTMLElement | undefined) {
		if (!el?.getAnimations) return;
		for (const animation of el.getAnimations()) {
			if (animation.id === SLIDE_ID) animation.cancel();
		}
	}

	function slide(el: HTMLElement, delta: number) {
		cancelSlide(el);
		const animation = el.animate(
			[{ transform: `translateX(${delta}px)` }, { transform: 'translateX(0)' }],
			{ duration: SLIDE_MS, easing: SLIDE_EASE }
		);
		animation.id = SLIDE_ID;
	}

	$effect(() => {
		void tabs.map((t) => `${t.id}:${t.groupId ?? ''}`).join('|');
		const held = dragIndex;
		const animate = !reducedMotion();
		const next = new Map<string, number>();

		for (const el of tabEls()) {
			const index = Number(el.dataset.tabIndex);
			const tab = tabs[index];
			if (!tab) continue;
			const left = el.offsetLeft;
			next.set(tab.id, left);
			if (index === held) {
				cancelSlide(el);
				continue;
			}
			if (!animate) continue;
			const prev = layout.get(tab.id);
			if (prev === undefined) continue;
			const delta = prev - left;
			if (Math.abs(delta) < 1) continue;
			slide(el, delta);
		}

		layout = next;
	});

	function grab(index: number, e: PointerEvent) {
		if (e.button !== 0) return;
		pendingIndex = index;
		pointerStartX = e.clientX;
		pointerX = e.clientX;
		window.addEventListener('pointermove', drag);
		window.addEventListener('pointerup', release, { once: true });
		window.addEventListener('pointercancel', release, { once: true });
	}

	function clampDrag() {
		const el = draggedEl();
		if (!el || !strip) return;
		const base = el.offsetLeft - strip.scrollLeft;
		const min = strip.offsetLeft - base;
		const max = strip.offsetLeft + strip.clientWidth - el.offsetWidth - base;
		dragX = Math.min(Math.max(dragX, min), Math.max(min, max));
	}

	function drag(e: PointerEvent) {
		pointerX = e.clientX;

		if (dragIndex === null) {
			if (Math.abs(pointerX - pointerStartX) < DRAG_THRESHOLD) return;
			dragIndex = pendingIndex;
			dragOrigin = pointerStartX;
			cancelSlide(draggedEl());
		}

		const desired = pointerX - dragOrigin;
		dragX = desired;
		clampDrag();
		pinned = desired > dragX + 1 ? 1 : desired < dragX - 1 ? -1 : 0;
		if (!reordering) updateTargets();
	}

	function updateTargets() {
		const el = draggedEl();
		if (!el || dragIndex === null) return;

		if (pinned > 0 && dragIndex < tabs.length - 1) {
			void reorderTo(tabs.length - 1);
			return;
		}
		if (pinned < 0 && dragIndex > 0) {
			void reorderTo(0);
			return;
		}

		const box = el.getBoundingClientRect();
		const center = box.left + box.width / 2;

		let ahead = -1;
		let behind = -1;
		let hovered: { index: number; center: number; width: number } | null = null;

		for (const item of tabEls()) {
			const index = Number(item.dataset.tabIndex);
			if (index === dragIndex) continue;
			const b = item.getBoundingClientRect();
			const targetCenter = b.left + b.width / 2;
			const margin = b.width * NUDGE;
			if (index > dragIndex && center > targetCenter + margin) ahead = Math.max(ahead, index);
			if (index < dragIndex && center < targetCenter - margin) behind = behind === -1 ? index : Math.min(behind, index);
			if (center >= b.left && center <= b.right) hovered = { index, center: targetCenter, width: b.width };
		}

		if (ahead !== -1) {
			void reorderTo(ahead);
			return;
		}
		if (behind !== -1) {
			void reorderTo(behind);
			return;
		}

		if (hovered && groupable(hovered.index) && Math.abs(center - hovered.center) <= hovered.width * NUDGE) {
			armDwell(hovered.index);
		} else {
			cancelDwell();
		}
	}

	function groupable(index: number): boolean {
		if (dragIndex === null) return false;
		const source = tabs[dragIndex];
		const target = tabs[index];
		if (!source || !target) return false;
		return !source.groupId || source.groupId !== target.groupId;
	}

	function armDwell(index: number) {
		const id = tabs[index]?.id ?? null;
		if (!id || dwellId === id) return;
		cancelDwell();
		dwellId = id;
		dwellTimer = setTimeout(() => {
			const at = tabs.findIndex((tab) => tab.id === id);
			if (at !== -1 && at !== dragIndex) groupTargetIndex = at;
		}, GROUP_DWELL_MS);
	}

	function cancelDwell() {
		clearTimeout(dwellTimer);
		dwellTimer = undefined;
		dwellId = null;
		groupTargetIndex = null;
	}

	async function reorderTo(target: number) {
		if (dragIndex === null || target === dragIndex || reordering) return;
		reordering = true;
		cancelDwell();
		const before = draggedEl()?.offsetLeft ?? 0;
		const from = dragIndex;
		dragIndex = target;
		onreorder?.(from, target);
		await tick();
		const after = draggedEl()?.offsetLeft;
		if (after !== undefined) {
			dragOrigin += after - before;
			dragX = pointerX - dragOrigin;
			clampDrag();
		}
		reordering = false;
	}

	async function release() {
		window.removeEventListener('pointermove', drag);
		clearTimeout(dwellTimer);
		dwellTimer = undefined;
		dwellId = null;

		if (dragIndex === null) {
			pinned = 0;
			groupTargetIndex = null;
			return;
		}

		const index = dragIndex;
		const grouping = groupTargetIndex;
		const offset = dragX;

		pinned = 0;
		dragIndex = null;
		dragX = 0;
		groupTargetIndex = null;

		if (grouping !== null && grouping !== index) {
			const source = tabs[index];
			const target = tabs[grouping];
			if (source && target) ongroupdrop?.(source.id, target.id);
			return;
		}

		if (Math.abs(offset) < 1 || reducedMotion()) return;
		await tick();
		const el = strip?.querySelector<HTMLElement>(`[data-tab-index="${index}"]`);
		if (el) slide(el, offset);
	}

	function openContextMenu(e: MouseEvent) {
		e.preventDefault();
		const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-tab-index]') : null;
		const tab = el ? tabs[Number(el.dataset.tabIndex)] : null;
		const MENU_W = 200;
		const MENU_H = 280;
		contextMenu = {
			x: Math.max(8, Math.min(e.clientX, window.innerWidth - MENU_W - 8)),
			y: Math.max(8, Math.min(e.clientY, window.innerHeight - MENU_H - 8)),
			tabId: tab?.id ?? null,
			grouped: Boolean(tab?.groupId)
		};
	}

	$effect(() => {
		if (!contextMenu) return;
		const dismiss = (e: PointerEvent) => {
			if (e.button !== 0) return;
			contextMenu = null;
		};
		const key = (e: KeyboardEvent) => {
			if (e.key === 'Escape') contextMenu = null;
		};
		window.addEventListener('pointerdown', dismiss, true);
		window.addEventListener('blur', dismiss as EventListener);
		window.addEventListener('keydown', key);
		return () => {
			window.removeEventListener('pointerdown', dismiss, true);
			window.removeEventListener('blur', dismiss as EventListener);
			window.removeEventListener('keydown', key);
		};
	});

	function groupFor(tab: TabData) {
		return groups.find((group) => group.id === tab.groupId);
	}

	$effect(() => {
		const idx = tabs.findIndex((t) => t.id === activeId);
		if (idx === -1 || !strip || dragIndex !== null) return;
		strip.querySelector<HTMLElement>(`[data-tab-index="${idx}"]`)
			?.scrollIntoView({ inline: 'nearest', block: 'nearest' });
	});

	$effect(() => {
		onmenustate?.(contextMenu ? { ...contextMenu } : null);
	});

	let lastReset = -1;
	$effect(() => {
		const token = menuResetToken ?? 0;
		if (lastReset === -1) {
			lastReset = token;
			return;
		}
		if (token === lastReset) return;
		lastReset = token;
		if (contextMenu) {
			contextMenu = null;
			onmenustate?.(null);
		}
	});
</script>

{#snippet tabItem(tab: TabData, i: number, inGroup: boolean)}
	<Tab
		index={i}
		title={tab.title}
		favicon={tab.favicon ?? ''}
		active={tab.id === activeId}
		muted={tab.muted ?? false}
		audible={tab.audible ?? false}
		dragging={dragIndex === i}
		dragOffset={dragIndex === i ? dragX : 0}
		{inGroup}
		groupColor={groupFor(tab)?.color}
		groupTarget={groupTargetIndex === i}
		onselect={() => onselect(tab.id)}
		onclose={() => onclose(tab.id)}
		onmutetoggle={() => onmute?.(tab.id)}
		onpointerdown={(e) => grab(i, e)}
	/>
{/snippet}

<div class="tabbar" class:dragging={dragIndex !== null}>
	<div class="strip"
		role="tablist"
		aria-label="Tabs"
		tabindex="0"
		bind:this={strip}
		oncontextmenu={openContextMenu}
	>
		{#each runs as run (`${run.group?.id ?? 'solo'}-${run.items[0].index}`)}
			{#if run.group}
				<div
					class="run grouped"
					style:--group-color={run.group.color}
					role="group"
					aria-label={run.group.name}
				>
					<button
						class="group-chip"
						type="button"
						aria-label="Edit group {run.group.name}"
						title="Rename or recolor “{run.group.name}”"
						onclick={(e) => {
							e.stopPropagation();
							const box = (e.currentTarget as HTMLElement).getBoundingClientRect();
							oneditgroup?.(run.group!.id, box.left, box.bottom + 8);
						}}
					>
						<span class="group-name">{run.group.name}</span>
						<svg class="group-pencil" viewBox="0 0 24 24" aria-hidden="true">
							<path d="M4 20h4L18.5 9.5a2.828 2.828 0 1 0-4-4L4 16v4" />
						</svg>
					</button>
					{#each run.items as item (item.tab.id)}
						{@render tabItem(item.tab, item.index, true)}
					{/each}
					<button class="group-add" type="button" aria-label="Add tab to {run.group.name}" onclick={(e) => { e.stopPropagation(); onaddtogroup?.(run.group?.id ?? ''); }}>
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
					</button>
				</div>
			{:else}
				{#each run.items as item (item.tab.id)}
					{@render tabItem(item.tab, item.index, false)}
				{/each}
			{/if}
		{/each}
	</div>

	<button class="new" type="button" onclick={onnew} aria-label="New tab" title="New tab">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M12 5v14M5 12h14" />
		</svg>
	</button>
</div>

<style>
	.tabbar {
		position: relative;
		display: flex;
		align-items: center;
		flex: 0 1 auto;
		min-width: 0;
		height: 40px;
		padding: 0;
		background: transparent;
		box-sizing: border-box;
	}

	.tabbar.dragging {
		cursor: grabbing;
	}

	.strip {
		display: flex;
		align-items: center;
		flex: 1 1 auto;
		gap: 2px;
		min-width: 0;
		padding: 0 2px;
		overflow-x: auto;
		overflow-y: visible;
		scrollbar-width: none;
		-ms-overflow-style: none;
	}
	.strip::-webkit-scrollbar {
		display: none;
	}

	.tabbar.dragging .strip {
		overflow-x: hidden;
	}

	.run {
		display: flex;
		align-items: center;
		gap: 2px;
		flex: 0 0 auto;
	}

	.run.grouped {
		gap: 1px;
		padding: 3px;
		border-radius: 12px;
		background: color-mix(in srgb, var(--group-color) 20%, transparent);
	}

	.group-add {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 22px;
		height: 22px;
		padding: 0;
		border: 0;
		border-radius: 50%;
		background: transparent;
		color: var(--group-color);
		opacity: .75;
		cursor: pointer;
		transition: background-color .14s ease, opacity .14s ease;
	}
	.group-add:hover {
		background: color-mix(in srgb, var(--group-color) 25%, transparent);
		opacity: 1;
	}
	.group-add svg {
		width: 12px;
		height: 12px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2.5;
		stroke-linecap: round;
	}

	.group-chip {
		display: grid;
		place-items: center;
		flex: 0 0 auto;
		max-width: 150px;
		height: 22px;
		padding: 0 8px;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: var(--text);
		font: inherit;
		font-size: 11.5px;
		font-weight: 600;
		cursor: pointer;
		transition: background-color .14s ease;
	}
	.group-chip:hover {
		background: color-mix(in srgb, var(--group-color) 30%, transparent);
	}
	.group-chip:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	.group-name,
	.group-pencil {
		grid-area: 1 / 1;
		transition: opacity .12s ease;
	}

	.group-name {
		max-width: 100%;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.group-pencil {
		width: 13px;
		height: 13px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
		opacity: 0;
	}

	.group-chip:hover .group-name,
	.group-chip:focus-visible .group-name {
		opacity: 0;
	}
	.group-chip:hover .group-pencil,
	.group-chip:focus-visible .group-pencil {
		opacity: 1;
	}

	.new {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 28px;
		height: 28px;
		margin: 0 2px 0 4px;
		align-self: center;
		padding: 0;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--text);
		cursor: default;
		transition: background-color 150ms ease-in-out;
	}
	.new svg {
		width: 16px;
		height: 16px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.75;
		stroke-linecap: round;
		opacity: 0.65;
	}
	.new:hover {
		background: var(--hover);
	}
	.new:hover svg {
		opacity: 1;
	}

	.new:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	@media (prefers-reduced-motion: reduce) {
		.new,
		.group-add,
		.group-chip,
		.group-pencil {
			transition: none;
		}
	}
</style>
