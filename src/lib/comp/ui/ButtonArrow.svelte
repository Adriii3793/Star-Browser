<script lang="ts">
    import { ArrowRight } from '@lucide/svelte';

    let {label = 'Continue', disabled = false, onclick}:
    { label?: string; disabled?:boolean;onclick?: () => void } = $props();

</script>

<button class="arrow-btn" type="button"  {disabled} {onclick}>
    <span class="label">{label}</span>
    <span class="icon">
        <ArrowRight aria-hidden="true" />
    
    </span>
</button>
<style>
    .arrow-btn {
        position: relative;
        display: flex;
        align-items: center;
        height: 2.8em;
        padding: 0.35em 1.2em 0.35em 3.3em;
        border: none;
        border-radius: 0.9em;
        background: var(--accent, #80A4D4);
        color: var(--accent-contrast, #1c1917);
        font: inherit;
        font-size: 16px;
        font-weight: 600;
        letter-spacing: 0.03em;
        overflow: hidden;
        cursor: pointer;
        box-shadow: inset 0 0 1.6em -0.6em color-mix(in srgb, var(--accent, #80A4D4) 65%, black);
    }
    .arrow-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    .icon {
        position: absolute;
        left: 0.3em;
        display:flex;
        align-items: center;
        justify-content: center;
        height: 2.2em;
        width: 2.2em;
        border-radius: 0.7em;
        background: var(--bg-page, #fff);
        box-shadow: 0.1em 0.1em 0.6em 0.2em color-mix(in srgb, var(--accent, #80A4D4) 55%, black);
        transition: width 0.3s ease;
    }

    .arrow-btn:hover:not(:disabled) .icon {
        width: calc(100% - 0.6em);
    }
    .icon :global(svg) {
        width: 1.1em;
        height: 1.1em;
        color: var(--accent, #80A4D4);
        transition: transform 0.3s ease;
    }
    .arrow-btn:hover:not(:disabled) .icon :global(svg) {
        transform: translateX(0.15em);
    }
    .arrow-btn:active:not(:disabled) .icon {
        transform: scale(0.96);
    }
    .arrow-btn:focus-visible {
        outline: 2px solid var(--accent, #80A4D4);
        outline-offset: 3px;
    }

    @media (prefers-reduced-motion: reduce) {
        .icon,
        .icon :global(svg) {
            transition: none;
        }
    }
</style>