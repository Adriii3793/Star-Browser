<script lang="ts">
	import type { SearchEngine } from '$lib/stores/setup.svelte';

	let {
		engine,
		size = 28,
		radius = 8,
		pad = Math.round(size * 0.15),
		surface = 'var(--field, #f7f1ec)'
	}: {
		engine: SearchEngine;
		size?: number;
		radius?: number;
		pad?: number;
		surface?: string;
	} = $props();

	let failedFor = $state<string | null>(null);
	let lettered = $derived(failedFor === engine.id);
</script>

<span
	class="logo"
	class:lettered
	style:--size="{size}px"
	style:--radius="{radius}px"
	style:--pad="{pad}px"
	style:--surface={lettered ? engine.color : surface}
	style:--letter="{Math.max(9, Math.round(size * 0.4))}px"
>
	{#if lettered}
		{engine.initial}
	{:else}
		<img src={engine.logo} alt="" onerror={() => (failedFor = engine.id)} />
	{/if}
</span>

<style>
	.logo {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: var(--size);
		height: var(--size);
		padding: var(--pad);
		border-radius: var(--radius);
		background: var(--surface);
		color: #fff;
		font-size: var(--letter);
		font-weight: 700;
		line-height: 1;
		overflow: hidden;
		box-sizing: border-box;
	}

	.logo.lettered {
		padding: 0;
	}

	.logo img {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}
</style>
