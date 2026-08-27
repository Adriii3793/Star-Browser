<script lang="ts">
	import Favicon from '../ui/Favicon.svelte';
	let {
		title,
		index,
		active = false,
		url = '',
		muted = false,
		audible = false,
		closable = true,
		dragging = false,
		dragOffset = 0,
		inGroup = false,
		groupColor = '',
		groupTarget = false,
		onselect,
		onclose,
		onmutetoggle,
		onpointerdown
	}: {
		title: string;
		index: number;
		active?: boolean;
		url?: string;
		muted?: boolean;
		audible?: boolean;
		closable?: boolean;
		dragging?: boolean;
		dragOffset?: number;
		inGroup?: boolean;
		groupColor?: string;
		groupTarget?: boolean;
		onselect: () => void;
		onclose: () => void;
		onmutetoggle?: () => void;
		onpointerdown?: (e: PointerEvent) => void;
	} = $props();


	function press(e: PointerEvent) {
		if (e.target instanceof Element && e.target.closest('[data-close], [data-audio]')) return;
		onpointerdown?.(e);
	}

	function keydown(e: KeyboardEvent) {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			onselect();
		} else if (e.key === 'Delete' || e.key === 'Backspace') {
			e.preventDefault();
			onclose();
		}
	}

	function close(e: MouseEvent) {
		e.stopPropagation();
		onclose();
	}
</script>

<div
	class="tab"
	class:active
	class:dragging
	class:group-target={groupTarget}
	class:in-group={inGroup}
	data-tab-index={index}
	style:--group-color={groupColor || 'transparent'}
	style:transform={dragging ? `translateX(${dragOffset}px)` : undefined}
	role="tab"
	aria-selected={active}
	tabindex={active ? 0 : -1}
	{title}
	onclick={onselect}
	onauxclick={(e) => e.button === 1 && onclose()}
	onpointerdown={press}
	onkeydown={keydown}
>
	<span class="icon"><Favicon {url} size={15} /></span>

	{#if muted || audible}
		<button
			class="audio"
			data-audio
			type="button"
			tabindex="-1"
			aria-label={muted ? 'Unmute tab' : 'Mute tab'}
			title={muted ? 'Unmute tab' : 'Mute tab'}
			onclick={(e) => { e.stopPropagation(); onmutetoggle?.(); }}
		>
			{#if muted}
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M15 8a5 5 0 0 1 0 8M6 15H4a1 1 0 0 1-1-1v-4a1 1 0 0 1 1-1h2l4-4v14z" />
					<path d="M3 3l18 18" />
				</svg>
			{:else}
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M15 8a5 5 0 0 1 0 8M17.7 5a9 9 0 0 1 0 14M6 15H4a1 1 0 0 1-1-1v-4a1 1 0 0 1 1-1h2l4-4v14z" />
				</svg>
			{/if}
		</button>
	{/if}

	<span class="label">{title}</span>

	{#if closable}
		<button
			class="close"
			data-close
			type="button"
			tabindex="-1"
			aria-label="Close tab"
			onclick={close}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M18 6 6 18M6 6l12 12" />
			</svg>
		</button>
	{/if}
</div>

<style>
	.tab {
		--tab-height: 34px;
		--tab-basis: 210px;
		--tab-max: 320px;
		--tab-min: 58px;

		position: relative;
		z-index: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		flex: 1 1 var(--tab-basis);
		width: auto;
		min-width: var(--tab-min);
		max-width: var(--tab-max);
		height: var(--tab-height);
		padding: 0 8px 0 10px;
		border: 0;
		border-radius: 9px;
		background: transparent;
		color: var(--text-soft);
		font-size: 12px;
		font-weight: 500;
		cursor: default;
		user-select: none;
		-webkit-user-select: none;
		transition:
			background-color 150ms ease,
			color 150ms ease,
			box-shadow 150ms ease;
		animation: tab-in 160ms cubic-bezier(.32, .72, 0, 1);
	}

	@keyframes tab-in {
		from { opacity: 0; transform: scale(.94); }
	}

	.tab:not(.active):hover {
		background: var(--hover);
		color: var(--text);
	}

	.tab.active {
		z-index: 2;
		background: var(--tab-active);
		color: var(--text);
		font-weight: 600;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06), 0 5px 14px var(--shadow);
	}

	.tab.dragging {
		z-index: 30;
		background: var(--tab-active);
		color: var(--text);
		box-shadow: 0 2px 6px rgba(0, 0, 0, 0.10), 0 10px 26px var(--shadow);
		transition: none;
		animation: none;
		cursor: grabbing;
	}

	.tab.group-target::after {
		content: '';
		position: absolute;
		inset: -2px;
		border: 2px solid var(--accent);
		border-radius: 11px;
		pointer-events: none;
	}
	.tab.group-target {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--text);
	}

	.tab:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
	}

	.icon {
		display: flex;
		flex: 0 0 auto;
	}
	.tab.active .icon {
		opacity: 1;
	}

	.audio {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 16px;
		height: 16px;
		padding: 0;
		border: 0;
		border-radius: 5px;
		background: transparent;
		color: inherit;
		cursor: default;
		opacity: 0.7;
	}
	.audio:hover {
		background: var(--hover);
		opacity: 1;
	}
	.audio svg {
		width: 12px;
		height: 12px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.8;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.label {
		flex: 1 1 auto;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		text-align: left;
		-webkit-mask-image: linear-gradient(to right, #000 calc(100% - 12px), transparent 100%);
		mask-image: linear-gradient(to right, #000 calc(100% - 12px), transparent 100%);
	}

	.close {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 19px;
		height: 19px;
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		cursor: default;
		opacity: 0;
		transition:
			opacity 150ms ease-in-out,
			background-color 150ms ease-in-out;
	}
	.close svg {
		width: 11px;
		height: 11px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
	}
	.tab:hover .close,
	.tab.active .close {
		opacity: 1;
	}
	.close:hover {
		background: var(--hover);
	}

	@media (prefers-reduced-motion: reduce) {
		.tab,
		.close {
			transition: none;
			animation: none;
		}
	}
</style>
