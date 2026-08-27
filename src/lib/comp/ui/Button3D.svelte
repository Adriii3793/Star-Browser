<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        label = 'Continue',
        type = 'button',
        disabled = false,
        onclick,
        children
    }: {
        label?: string;
        type?: 'button' | 'submit';
        disabled?: boolean;
        onclick?: () => void;
        children?: Snippet;
    } = $props();
</script>

<button class="pushable" {type} {disabled} {onclick}>
    <span class="edge"></span>
    <span class="front">
        {@render children?.()}
        {#if !children}{label}{/if}
    </span>
</button>

<style>
    .pushable {
        position: relative;
        display: inline-block;
        padding: 0 0 4px;
        border: none;
        border-radius: 14px;
        background: transparent;
        cursor: pointer;
        outline-offset: 4px;
        -webkit-tap-highlight-color: transparent;
    }

    .edge {
        position: absolute;
        inset: 0;
        border-radius: 14px;
        background: color-mix(in srgb, var(--accent, #80a4d4) 62%, #000);
        box-shadow: 0 8px 20px -8px color-mix(in srgb, var(--accent, #80a4d4) 75%, transparent);
        transition: box-shadow 0.2s ease;
    }

    .front {
        position: relative;
        display: block;
        min-width: 168px;
        padding: 13px 30px;
        border-radius: 14px;
        background: var(--accent, #80a4d4);
        color: var(--accent-contrast, #1c1917);
        font: inherit;
        font-size: 15px;
        font-weight: 600;
        letter-spacing: 0.01em;
        transition: transform 180ms cubic-bezier(0.32, 0.72, 0, 1);
    }

    .pushable:hover:not(:disabled) .front {
        transform: translateY(-2px);
    }

    .pushable:hover:not(:disabled) .edge {
        box-shadow: 0 12px 24px -8px color-mix(in srgb, var(--accent, #80a4d4) 80%, transparent);
    }

    .pushable:active:not(:disabled) .front {
        transform: translateY(4px);
        transition-duration: 40ms;
    }

    .pushable:disabled {
        cursor: not-allowed;
    }

    .pushable:disabled .front {
        background: var(--field-strong, #efe6de);
        color: var(--text-muted, #ac8064);
    }

    .pushable:disabled .edge {
        background: var(--border-strong);
        box-shadow: none;
    }

    .pushable:focus-visible {
        outline: 2px solid var(--accent, #80a4d4);
    }

    @media (prefers-reduced-motion: reduce) {
        .front,
        .edge {
            transition: none;
        }
        .pushable:hover:not(:disabled) .front {
            transform: none;
        }
    }
</style>
