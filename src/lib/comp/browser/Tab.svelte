<script lang="ts">
	let {
		title,
		index,
		active = false,
		favicon = '',
		muted = false,
		audible = false,
		closable = true,
		dragging = false,
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
		favicon?: string;
		muted?: boolean;
		audible?: boolean;
		closable?: boolean;
		dragging?: boolean;
		inGroup?: boolean;
		groupColor?: string;
		groupTarget?: boolean;
		onselect: () => void;
		onclose: () => void;
		onmutetoggle?: () => void;
		onpointerdown?: (e: PointerEvent) => void;
	} = $props();

	let iconFailed = $state(false);
	$effect(() => { void favicon; iconFailed = false; })

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

	function close(e:MouseEvent) {
		e.stopPropagation();
		onclose();
	}
</script>

<div
	class="tab"
	class:active
	class:dragging
	class:groupTarget
	class:in-group={inGroup}
	class:grouped={Boolean(groupColor)}
	data-tab-index={index}
	style:--group-color={groupColor || 'transparent'}
	role="tab"
	aria-selected={active}
	tabindex={active ? 0 : -1}
	{title}
	onclick={onselect}
	onauxclick={(e) => e.button === 1 && onclose()}
	onpointerdown={press}
	onkeydown={keydown}
>
	<span class="icon">
		{#if favicon && !iconFailed}
			<img src={favicon} alt="" onerror={() => (iconFailed = true)} />
		{:else}
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="9" />
				<path d="M3.6 9h16.8M3.6 15h16.8M11.5 3a17 17 0 0 0 0 18M12.5 3a17 17 0 0 1 0 18" />
			</svg>
		{/if}
	</span>

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
			aria-label="Close Tab"
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
		position: relative;
		display: flex;
		align-items: center;
		gap: 7px;
		flex: 0 1 132px;
		width: auto;
		min-width: 84px;
		max-width: 168px;
		height: 30px;
		padding: 0 7px 0 9px;
		border: 0;
		border-radius: 9px;
		background: transparent;
		color: var(--text-soft);
		font-size: 12px;
		font-weight: 500;
		cursor: default;
		transition: background-color 150ms ease, color 150ms ease, height 150ms cubic-bezier(.32, .72, 0, 1), max-width 150ms cubic-bezier(.32, .71, 0, 1), box-shadow 150ms ease;
		animation: tab-in 160ms cubic-bezier(.32, .72, 0,);
	}

	@keyframes tab-in {
		from { opacity: 0; transform: scale(.92);}
	}

	.tab.in-group {
		max-width: 132px;
	}
	.tab:not(.active):hover {
		background: var(--hover);
		color: var(--text);
	}

	.tab.active {
		z-index: 2;
		height: 34px;
		max-width: 190px;
		background: var(--tab-active, #ffffff);
		color: var(--text);
		font-weight: 600;
		border-radius: 10px;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.06), 0 5px 14px var(--shadow);
	}

	.tab.dragging {
		opacity: 0.7;
	}
	.tab.groupTarget {
		outline: 2px solid var(--accent);
		outline-offset: -2px;
		background: var(--tab-hover);
	}

	.tab:focus-visible {
		outline: 2px solid var(--accent, #1a73e8);
		outline-offset: -2px;
	}

	.icon {
		display: flex;
		flex: 0 0 auto;
		width: 15px;
		height: 15px;
	}
	.icon img {
		width: 100%;
		height: 100%;
		border-radius: 4px;
		object-fit: contain;
	}
	.icon svg {
		width: 100%;
		height: 100%;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.75;
		stroke-linecap: round;
		opacity: 0.55;
	}
	.tab.active .icon svg {
		opacity: 0.75;
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
		flex: 1;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		text-align: left;
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
		}
	}
</style>