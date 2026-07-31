<script lang="ts">
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
		oncreateGroup,
		onungroup,
		ongroupdrop,
		onaddtogroup,
		onaddexistingtogroup,
		onrenamegroup,
		onmute,
		onduplicate,
		oncloseothers,
		onmenutoggle
	}: {
		tabs: TabData[];
		activeId: string;
		onselect: (id: string) => void;
		onclose: (id: string) => void;
		onnew: () => void;
		onreorder?: (from: number, to: number) => void;
		groups?: TabGroup[];
		oncreateGroup?: (tabId?: string | null) => void;
		onungroup?: (tabId: string) => void;
		ongroupdrop?: (sourceId: string, targetId: string) => void;
		onaddtogroup?: (groupId: string) => void;
		onaddexistingtogroup?: (tabId: string, groupId: string) => void;
		onrenamegroup?: (groupId: string, name: string) => void;
		onmute?: (tabId: string) => void;
		onduplicate?: (tabId: string) => void;
		oncloseothers?: (tabId: string) => void;
		onmenutoggle?: (open: boolean) => void;
	} = $props();

	let strip = $state<HTMLElement>();
	let dragIndex = $state<number | null>(null);
	let pendingIndex = 0;
	let startX = 0;
	let groupTargetIndex = $state<number | null>(null);
	let contextMenu = $state<{ x: number; y: number; tabId: string | null; grouped: boolean } | null>(null);
	let renameTarget = $state<{ groupId: string; value: string; x: number; y: number } | null>(null);

	const THRESHOLD = 4;
	
	type Run = {group: TabGroup | null; items: {tab:TabData; index: number}[]};
	let runs = $derived.by(() => {
		const out: Run[] = [];
		tabs.forEach((tab, index) => {
			const group = groups.find((g) => g.id === tab.groupId) ?? null;
			const last = out[out.length - 1];
			if (last && last.group?.id === group?.id) last.items.push({tab, index});
			else out.push({group, items: [{tab, index}]});
		});
		return out;
	});

	function grab(index: number, e: PointerEvent) {
		if (e.button !== 0) return;
		pendingIndex = index;
		startX = e.clientX;
		window.addEventListener('pointermove', drag);
		window.addEventListener('pointerup', release, { once: true });
		window.addEventListener('pointercancel', release, { once: true });
	}

	function tabEls(): HTMLElement[] {
		return [...(strip?.querySelectorAll<HTMLElement>('[data-tab-index]') ?? [])];
	}

	function drag(e: PointerEvent) {
		if (dragIndex === null) {
			if (Math.abs(e.clientX - startX) < THRESHOLD) return;
			dragIndex = pendingIndex;
		}

		const hit = tabEls().find((el) => {
			const box = el.getBoundingClientRect();
			return e.clientX >= box.left && e.clientX <= box.right;
		});
		const target = hit ? Number(hit.dataset.tabIndex) : -1;

		if (target === -1 || target === dragIndex) {
			groupTargetIndex = null;
			return;
		}

		const targetBox = hit!.getBoundingClientRect();
		const overCenter = e.clientX > targetBox.left + targetBox.width * 0.25 && e.clientX < targetBox.right - targetBox.width * 0.25;
		if (overCenter) {
			groupTargetIndex = target;
			return;
		}

		groupTargetIndex = null;
		onreorder?.(dragIndex, target);
		dragIndex = target;
	}

	function release() {
		window.removeEventListener('pointermove', drag);
		if (dragIndex !== null && groupTargetIndex !== null && dragIndex !== groupTargetIndex) {
			const source = tabs[dragIndex];
			const target = tabs[groupTargetIndex];
			if (source && target) ongroupdrop?.(source.id, target.id);
		}
		dragIndex = null;
		groupTargetIndex = null;
	}

	function openContextMenu(e: MouseEvent) {
		e.preventDefault();
		const el = e.target instanceof Element ? e.target.closest<HTMLElement>('[data-tab-index]') : null;
		const tab = el ? tabs[Number(el.dataset.tabIndex)] : null;
		contextMenu = { x: e.clientX, y: e.clientY, tabId: tab?.id ?? null, grouped: Boolean(tab?.groupId) };
	}

	function createGroup() {
		const menu = contextMenu;
		contextMenu = null;
		if (!menu?.tabId) return;
		oncreateGroup?.(menu.tabId);
	}

	function addToGroup(groupId: string) {
		const menu = contextMenu;
		contextMenu = null;
		if (!menu?.tabId) return;
		onaddexistingtogroup?.(menu.tabId, groupId);
	}

	function removeFromGroup() {
		const menu = contextMenu;
		contextMenu = null;
		if (!menu?.tabId) return;
		onungroup?.(menu.tabId);
	}

	function renameStart() {
		const menu = contextMenu;
		contextMenu = null;
		if (!menu?.tabId) return;
		const tab = tabs.find((t) => t.id === menu.tabId);
		const group = tab?.groupId ? groups.find((g) => g.id === tab.groupId) : null;
		if (!group) return;
		renameTarget = { groupId: group.id, value: group.name, x: menu.x, y: menu.y };
	}

	function confirmRename() {
		if (!renameTarget) return;
		onrenamegroup?.(renameTarget.groupId, renameTarget.value);
		renameTarget = null;
	}

	$effect(() => {
		if (!contextMenu && !renameTarget) return;
		const dismiss = (e: Event) => {
			if (e.target instanceof Element && (e.target.closest('.group-menu') || e.target.closest('.rename-pop'))) return;
			contextMenu = null;
			renameTarget = null;
		};
		const key = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				contextMenu = null;
				renameTarget = null;
			}
		};
		window.addEventListener('pointerdown', dismiss, true);
		window.addEventListener('blur', dismiss);
		window.addEventListener('keydown', key);
		return () => {
			window.removeEventListener('pointerdown', dismiss, true);
			window.removeEventListener('blur', dismiss);
			window.removeEventListener('keydown', key);
		};
	});

	function groupFor(tab: TabData) {
		return groups.find((group) => group.id === tab.groupId);
	}

	$effect(() => {
		const idx = tabs.findIndex((t) => t.id === activeId);
		if (idx === -1 || !strip) return;
		const el = strip.querySelector<HTMLElement>(`[data-tab-index="${idx}"]`);
		el?.scrollIntoView({ inline: 'nearest', block: 'nearest' });
	});

	$effect(() => {
		onmenutoggle?.(Boolean(contextMenu || renameTarget));
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
		{inGroup}
		groupColor={groupFor(tab)?.color}
		groupTarget={groupTargetIndex === i}
		onselect={() => onselect(tab.id)}
		onclose={() => onclose(tab.id)}
		onmutetoggle={() => onmute?.(tab.id)}
		onpointerdown={(e) => grab(i, e)}
	/>
{/snippet}

<div class="tabbar">
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
					title={run.group.name}
				>
					{#each run.items as item (item.tab.id)}
						{@render tabItem(item.tab, item.index, true)}
					{/each}
					<button class="group-add" type="button" aria-label="Add tab to {run.group.name}" onclick={() => onaddtogroup?.(run.group?.id ?? '')}>
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

	<button class="new" type="button" onclick={onnew} aria-label="Nuova scheda">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M12 5v14M5 12h14" />
		</svg>
	</button>
</div>

{#if contextMenu}
	{@const menuTabId = contextMenu.tabId}
	{@const targetTab = menuTabId ? tabs.find((t) => t.id === menuTabId) : null}
	<div class="group-menu" role="menu" style:left="{contextMenu.x}px" style:top="{contextMenu.y}px">
		{#if menuTabId}
			<button class="group-menu-item" type="button" role="menuitem"
				onclick={() => { const id = menuTabId; contextMenu = null; onmute?.(id); }}>
				{targetTab?.muted ? 'Unmute tab' : 'Mute tab'}
			</button>
			<button class="group-menu-item" type="button" role="menuitem"
				onclick={() => { const id = menuTabId; contextMenu = null; onduplicate?.(id); }}>
				Duplicate tab
			</button>
			<button class="group-menu-item" type="button" role="menuitem"
				onclick={() => { const id = menuTabId; contextMenu = null; oncloseothers?.(id); }}>
				Close other tabs
			</button>
			<div class="menu-sep" role="separator"></div>
		{/if}
		{#if !contextMenu.grouped}
			<button class="group-menu-item" type="button" role="menuitem" onclick={createGroup}>
				Create a Group
			</button>
			{#each groups as group (group.id)}
				<button class="group-menu-item" type="button" role="menuitem" onclick={() => addToGroup(group.id)}>
					<span class="group-dot" style:background={group.color}></span>
					Add to "{group.name}"
				</button>
			{/each}
		{:else}
			<button class="group-menu-item" type="button" role="menuitem" onclick={renameStart}>
				Rename Group
			</button>
			{#each groups.filter((g) => g.id !== targetTab?.groupId) as group (group.id)}
				<button class="group-menu-item" type="button" role="menuitem" onclick={() => addToGroup(group.id)}>
					<span class="group-dot" style:background={group.color}></span>
					Move to "{group.name}"
				</button>
			{/each}
			<button class="group-menu-item" type="button" role="menuitem" onclick={removeFromGroup}>
				<span class="group-dot muted"></span>
				Remove from Group
			</button>
		{/if}
	</div>
{/if}

{#if renameTarget}
	<div class="rename-pop" style:left="{renameTarget.x}px" style:top="{renameTarget.y}px">
		<input
			type="text"
			bind:value={renameTarget.value}
			maxlength="30"
			aria-label="Group name"
			onkeydown={(e) => {
				if (e.key === 'Enter') confirmRename();
				else if (e.key === 'Escape') renameTarget = null;
			}}
		/>
		<button type="button" aria-label="Save group name" onclick={confirmRename}>
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 7" /></svg>
		</button>
	</div>
{/if}

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

	.strip {
		display: flex;
		align-items: center;
		flex: 0 1 auto;
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
		color: var(--text,#444);
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

	.group-menu {
		position: fixed;
		z-index: 20;
		display: flex;
		flex-direction: column;
		min-width: 190px;
		padding: 6px;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-page);
		box-shadow: 0 8px 24px var(--shadow);
	}
	.group-menu-item {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 32px;
		padding: 0 10px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--text);
		font: inherit;
		font-size: 12px;
		font-weight: 600;
		text-align: left;
		white-space: nowrap;
		cursor: pointer;
	}
	.group-menu-item:hover { background: var(--tab-hover); }
	.menu-sep { height: 1px; margin: 5px 4px; background: var(--border); }
	.group-dot { flex: 0 0 auto; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
	.group-dot.muted { background: var(--text-muted); }

	.rename-pop {
		position: fixed;
		z-index: 21;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--bg-page);
		box-shadow: 0 8px 24px var(--shadow);
	}
	.rename-pop input {
		width: 150px;
		height: 28px;
		padding: 0 8px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: transparent;
		color: var(--text);
		font: inherit;
		font-size: 12px;
	}
	.rename-pop input:focus {
		outline: none;
		border-color: var(--accent);
	}
	.rename-pop button {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 28px;
		height: 28px;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: var(--accent);
		color: var(--accent-contrast, #ffffff);
		cursor: pointer;
	}
	.rename-pop button svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2.4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

    .new:focus-visible {
        outline: 2px solid var(--accent, #1a73e8);
        outline-offset: 2px;
    }

	@media (prefers-reduced-motion: reduce) {
		.new {
			transition: none;
		}
        .strip {
            scroll-behavior: auto;
        }
	}
</style>
