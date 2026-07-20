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
</script>

<div
	class="tab"
	class:active
	class:dragging
	onpointerdown={(e) => onpointerdown?.(e)}
	role="tab"
	aria-selected={active}
	tabindex="-1"
>
	<button class="surface" type="button" onclick={onselect} title={title}>
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
	</button>

	{#if closable}
		<button
			class="close"
			type="button"
			aria-label="Chiudi scheda"
			onclick={(e) => {
				e.stopPropagation();
				onclose();
			}}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M18 6 6 18M6 6l12 12" />
			</svg>
		</button>
	{/if}
</div>

<style>
	.tab {
		display: flex;
		align-items: center;
		flex: 0 0 auto;
		height: 32px;
		max-width: 180px;
		min-width: 0;
		padding: 0 8px 0 14px;
		border-radius: 7px 7px 0 0;
		background: #ececec;
		color: #444;
		font-size: 13px;
		font-weight: 500;
		font-family:
			Inter,
			-apple-system,
			BlinkMacSystemFont,
			'SF Pro Text',
			'Segoe UI',
			sans-serif;
		user-select: none;
		touch-action: none;
		transition:
			background-color 0.18s ease,
			box-shadow 0.18s ease;
	}

	.tab.dragging {
		opacity: 0.7;
		box-shadow: 0 2px 6px rgba(0, 0, 0, 0.12);
	}

	.tab:not(.active):hover {
		background: #e3e3e3;
	}

	.tab.active {
		background: #ffffff;
		box-shadow: 0 -1px 3px rgba(0, 0, 0, 0.06);
	}

	.surface {
		display: flex;
		align-items: center;
		gap: 7px;
		flex: 1;
		min-width: 0;
		padding: 0;
		border: 0;
		background: transparent;
		font: inherit;
		color: inherit;
		cursor: default;
	}

	.icon {
		display: flex;
		flex: 0 0 auto;
		width: 14px;
		height: 14px;
	}
	.icon img {
		width: 100%;
		height: 100%;
		border-radius: 3px;
		object-fit: contain;
	}
	.icon svg {
		width: 100%;
		height: 100%;
		fill: none;
		stroke: #8a8a8a;
		stroke-width: 2;
		stroke-linecap: round;
	}
	.tab.active .icon svg {
		stroke: #6e6e6e;
	}

	.label {
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.close {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 0 0 auto;
		width: 16px;
		height: 16px;
		margin-left: 6px;
		padding: 0;
		border: 0;
		border-radius: 4px;
		background: transparent;
		color: #7a7a7a;
		cursor: default;
		opacity: 0;
		transition:
			opacity 0.18s ease,
			background-color 0.18s ease;
	}
	.close svg {
		width: 11px;
		height: 11px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2.2;
		stroke-linecap: round;
	}
	.tab:hover .close,
	.tab.active .close {
		opacity: 1;
	}
	.close:hover {
		background: rgba(0, 0, 0, 0.08);
		color: #333;
	}

	@media (prefers-reduced-motion: reduce) {
		.tab,
		.close {
			transition: none;
		}
	}
</style>