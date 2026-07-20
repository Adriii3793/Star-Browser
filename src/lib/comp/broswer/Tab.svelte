<script lang="ts">
	let {
		title,
		active = false,
		favicon = '',
		closable = true,
		dragging = false,
		onselect,
		onclose,
		onpointerdown
	}: {
		title: string;
		active?: boolean;
		favicon?: string;
		closable?: boolean;
		dragging?: boolean;
		onselect: () => void;
		onclose: () => void;
		onpointerdown?: (e: PointerEvent) => void;
	} = $props();

	function press(e: PointerEvent) {
		if (e.target instanceof Element && e.target.closest('[data-close]')) return;
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
		{#if favicon}
			<img src={favicon} alt="" />
		{:else}
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="9" />
				<path d="M3.6 9h16.8M3.6 15h16.8M11.5 3a17 17 0 0 0 0 18M12.5 3a17 17 0 0 1 0 18" />
			</svg>
		{/if}
	</span>

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
		gap: 8px;
		flex: 0 1 auto;
		width: 144px;
		min-width: 112px;
		height: 32px;
		padding: o 8px;
		border-radius: 8px;
		background: transparent;
		color: var(--text, #444);
		font-size: 13px;
		font-weight: 500;
		font-family:
			Inter
			-apple-system,
			BlinckMacSystemFont,
			'SF Pro Text',
			'Segoe UI',
			sans-serif;
		box-sizing: border-box;
		user-select: none;
		touch-action: none;
		cursor: default;
		transition: background-color 150ms ease-in-out;
	}

	.tab + .tab:before {
		content: '';
		position: absolute;
		left: 0;
		top: 8px;
		bottom: 8px;
		width: 1px;
		background: rgba(0, 0, 0, 0.08);
		transition: opacity 150ms ease-in-out
	}

	.tab:hover::before,
	.tab.active::before,
	.tab:hover + .tab::before,
	.tab.active + .tab::before {
		opacity: 0;
	}

	.tab:not(.active):hover {
		background: rgba(0, 0, 0, 0.04);
	}

	.tab.active {
		background: var(--tab-activce, #ffffff);
	}

	.tab.dragging {
		opacity: 0.7;
	}

	.tab:focus-visible {
		outline: 2px solid var(--accent, #1a73e8);
		outline-offset: -2px;
	}

	.icon {
		display: flex;
		flex: 0 0 auto;
		width: 16px; 
		height: 16px;
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
		opacity: 0,75;
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
		width: 24px;
		height: 24px;
		padding: 0;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: inherit;
		cursor: default;
		opacity: 0;
		transition:
			opacity 150ms ease-in-out,
			background-color 150ms ease-in-out;
	}
	.color svg {
		width: 12px;
		height: 12px;
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
		background: rgba(0, 0, 0, 0.08);
	}

	@media (preferes-reduced-motion: reduce) {
		.tab,
		.tab + .tab::before,
		.close {
			transition: none;
		}
	}
</style>