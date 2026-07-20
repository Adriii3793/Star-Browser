<script lang="ts">
	import Tab from './Tab.svelte';

	interface TabData {
		id: string;
		title: string;
		favicon?: string;
	}

	let {
		tabs,
		activeId,
		onselect,
		onclose,
		onnew,
		onreorder
	}: {
		tabs: TabData[];
		activeId: string;
		onselect: (id: string) => void;
		onclose: (id: string) => void;
		onnew: () => void;
		onreorder?: (from: number, to: number) => void;
	} = $props();

	let strip = $state<HTMLElement>();
	let dragIndex = $state<number | null>(null);
	let pendingIndex = 0;
	let startX = 0;

	const THRESHOLD = 4;

	$effect(() => {
		const index = tabs.findIndex((t) => t.id === activeId);
		if (dragIndex !== null) return;
		strip?.children[index]?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
	});

	function grab(index: number, e: PointerEvent) {
		if (e.button !== 0) return;
		pendingIndex = index;
		startX = e.clientX;
		window.addEventListener('pointermove', drag);
		window.addEventListener('pointerup', release, { once: true });
		window.addEventListener('pointercancel', release, { once: true });
	}

	function drag(e: PointerEvent) {
		if (!strip) return;

		if (dragIndex === null) {
			if (Math.abs(e.clientX - startX) < THRESHOLD) return;
			dragIndex = pendingIndex;
		}

		const target = [...strip.children].findIndex((el) => {
			const box = el.getBoundingClientRect();
			return e.clientX >= box.left && e.clientX <= box.right;
		});

		if (target === -1 || target === dragIndex) return;
		onreorder?.(dragIndex, target);
		dragIndex = target;
	}

	function release() {
		window.removeEventListener('pointermove', drag);
		dragIndex = null;
	}

	function move(e: KeyboardEvent) {
		const last = tabs.length - 1;
		const current = tabs.findIndex((t) => t.id === activeId);
		let next: number;

		if (e.key === 'ArrowRight') next = current === last ? 0 : current + 1;
		else if (e.key === 'ArrowLeft') next = current === 0 ? last : current -1;
		else if (e.key === 'Home') next = 0;
		else if (e.key === 'End') next = last;
		else return;

		e.preventDefault();
		onselect(tabs[next].id);
		(strip?.children[next] as HTMLElement | undefined)?.focus();
	}
</script>

<div class="tabbar">
	<div class="strip" role="tablist" aria-label="Tabs" bind:this={strip}>
		{#each tabs as tab, i (tab.id)}
			<Tab
				title={tab.title}
				favicon={tab.favicon ?? ''}
				active={tab.id === activeId}
				dragging={dragIndex === i}
				onselect={() => onselect(tab.id)}
				onclose={() => onclose(tab.id)}
				onpointerdown={(e) => grab(i, e)}
			/>
		{/each}
	</div>

	<button class="new" type="button" onclick={onnew} aria-label="New Tab">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M12 5v14M5 12h14" />
		</svg>
	</button>
</div>

<style>
	.tabbar {
		display: flex;
		align-items: flex-end;
		gap: 8px;
		min-width: 0;
		height: 100%;
		padding: 0 8px;
		box-sizing: border-box;
	}

	.strip {
		display: flex;
		align-items: flex-end;
		
	}
	.strip::-webkit-scrollbar {
		display: none;
	}

	.new {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 26px;
		height: 26px;
		margin: 0 0 3px 5px;
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: #666;
		cursor: default;
		transition: background-color 0.18s ease;
	}
	.new svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
	}
	.new:hover {
		background: #e3e3e3;
	}

	@media (prefers-reduced-motion: reduce) {
		.new {
			transition: none;
		}
	}
</style>