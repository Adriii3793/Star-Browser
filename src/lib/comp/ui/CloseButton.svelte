<script lang="ts">
    let {
        label = 'Close',
        title = label,
        size = 'md',
        onclick
    }: {
        label?: string;
        title?: string;
        size?: 'sm' | 'md';
        onclick: () => void;
    } = $props();
</script>

<button class="close-btn" class:sm={size === 'sm'} type="button" aria-label={label} {title} {onclick}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M18 6 6 18M6 6l12 12" />
    </svg>
</button>

<style>
    .close-btn {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 28px;
        height: 28px;
        padding: 0;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--text-soft, #8a6b57);
        cursor: pointer;
        transition:
            background-color 150ms ease,
            color 150ms ease;
    }

    .close-btn.sm {
        width: 24px;
        height: 24px;
        border-radius: 7px;
    }

    .close-btn:hover {
        background: var(--hover, rgba(0, 0, 0, 0.06));
        color: var(--text, #4a3a2e);
    }

    .close-btn:active {
        background: var(--field-strong, rgba(0, 0, 0, 0.1));
    }

    .close-btn:focus-visible {
        outline: 2px solid var(--accent, #80a4d4);
        outline-offset: 2px;
    }

    .close-btn svg {
        width: 16px;
        height: 16px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .close-btn.sm svg {
        width: 14px;
        height: 14px;
    }

    @media (prefers-reduced-motion: reduce) {
        .close-btn {
            transition: none;
        }
    }
</style>
