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

	function grab(index: number, e: PointerEvent) {
		if (e.button !== 0) return;
		pendingIndex = index;
		startX = e.clientX;
		window.addEventListener('pointermove', drag);
		window.addEventListener('pointerup', release, { once: true });
		window.addEventListener('pointercancel', release, { once: true });
	}

	function drag(e: PointerEvent) {
		if (dragIndex === null) {
			if (Math.abs(e.clientX - startX) < THRESHOLD) return;
			dragIndex = pendingIndex;
		}

		const target = [...(strip?.children ?? [])].findIndex((el) => {
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
</script>

<div class="tabbar">
	<div class="strip" role="tablist" aria-label="Schede" bind:this={strip}>
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

	<button class="new" type="button" onclick={onnew} aria-label="Nuova scheda">
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M12 5v14M5 12h14" />
		</svg>
	</button>
</div>

<style>
	.tabbar {
		display: flex;
		align-items: flex-end;
		min-width: 0;
		height: 38px;
		padding: 0 4px;
		background: transparent;
		box-sizing: border-box;

	}

	.strip {
		display: flex;
		align-items: flex-end;
		gap: 5px;
		min-width: 0;
		overflow-x: auto;
		overflow-y: hidden;
		scrollbar-width: none;
		-ms-overflow-style: none;
	}
	.strip::-webkit-scrollbar {
		display: none;
	}

	.new {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 32px;
		height: 32px;
		margin: 0 0 3px 5px;
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