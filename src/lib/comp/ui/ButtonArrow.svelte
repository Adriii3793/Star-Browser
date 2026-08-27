<script lang="ts">
    let {
        label = 'Continue',
        disabled = false,
        onclick
    }: { label?: string; disabled?: boolean; onclick?: () => void } = $props();
</script>

<button class="arrow-btn" type="button" {disabled} {onclick}>
    <span class="icon">
        <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M5 12h13M13 6l6 6-6 6" />
        </svg>
    </span>
    <span class="label">{label}</span>
</button>

<style>
    .arrow-btn {
        position: relative;
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 168px;
        height: 48px;
        padding: 0 26px 0 8px;
        border: none;
        border-radius: 14px;
        background: var(--accent, #80a4d4);
        color: var(--accent-contrast, #1c1917);
        font: inherit;
        font-size: 15px;
        font-weight: 600;
        letter-spacing: 0.01em;
        cursor: pointer;
        box-shadow: 0 8px 20px -8px color-mix(in srgb, var(--accent, #80a4d4) 75%, transparent);
        transition:
            transform 180ms cubic-bezier(0.32, 0.72, 0, 1),
            box-shadow 0.2s ease;
    }

    .label {
        flex: 1;
        text-align: center;
    }

    .icon {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 32px;
        height: 32px;
        border-radius: 10px;
        background: var(--bg-page, #fff);
        color: var(--accent, #80a4d4);
    }

    .icon svg {
        width: 17px;
        height: 17px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2.2;
        stroke-linecap: round;
        stroke-linejoin: round;
        transition: transform 180ms cubic-bezier(0.32, 0.72, 0, 1);
    }

    .arrow-btn:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 12px 24px -8px color-mix(in srgb, var(--accent, #80a4d4) 80%, transparent);
    }

    .arrow-btn:hover:not(:disabled) .icon svg {
        transform: translateX(3px);
    }

    .arrow-btn:active:not(:disabled) {
        transform: translateY(1px);
        transition-duration: 40ms;
    }

    .arrow-btn:disabled {
        cursor: not-allowed;
        background: var(--field-strong, #efe6de);
        color: var(--text-muted, #ac8064);
        box-shadow: none;
    }

    .arrow-btn:disabled .icon {
        color: var(--text-muted, #ac8064);
    }

    .arrow-btn:focus-visible {
        outline: 2px solid var(--accent, #80a4d4);
        outline-offset: 3px;
    }

    @media (prefers-reduced-motion: reduce) {
        .arrow-btn,
        .icon svg {
            transition: none;
        }
        .arrow-btn:hover:not(:disabled) {
            transform: none;
        }
        .arrow-btn:hover:not(:disabled) .icon svg {
            transform: none;
        }
    }
</style>
