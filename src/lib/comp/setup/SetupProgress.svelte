<script lang="ts">
    let {
        count,
        index
    }: {
        count: number;
        index: number;
    } = $props();
</script>

<div
    class="progress"
    role="progressbar"
    aria-label="Setup progress"
    aria-valuemin={1}
    aria-valuemax={count}
    aria-valuenow={index + 1}
    aria-valuetext="Step {index + 1} of {count}"
>
    {#each { length: count } as _, i}
        <span class="seg" class:done={i < index} class:active={i === index}>
            <span class="fill"></span>
        </span>
    {/each}
</div>

<style>
    .progress {
        display: flex;
        align-items: center;
        gap: 7px;
    }

    .seg {
        position: relative;
        width: 7px;
        height: 7px;
        border-radius: 999px;
        overflow: hidden;
        background: var(--field-strong, rgba(0, 0, 0, 0.09));
        transition:
            width 460ms cubic-bezier(0.34, 1.36, 0.5, 1),
            box-shadow 320ms ease;
    }

    .seg.active {
        width: 30px;
        box-shadow: 0 0 0 3.5px color-mix(in srgb, var(--accent) 13%, transparent);
    }

    .fill {
        position: absolute;
        inset: 0;
        border-radius: inherit;
        background: var(--accent, #80a4d4);
        opacity: 0;
        transform: scaleX(0);
        transform-origin: left center;
        transition:
            transform 420ms cubic-bezier(0.32, 0.72, 0, 1),
            opacity 300ms ease;
    }

    .seg.done .fill {
        opacity: 0.45;
        transform: scaleX(1);
    }

    .seg.active .fill {
        opacity: 1;
        transform: scaleX(1);
    }

    .seg.active::after {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: inherit;
        background: linear-gradient(
            90deg,
            transparent 0%,
            color-mix(in srgb, var(--accent-contrast, #fff) 55%, transparent) 50%,
            transparent 100%
        );
        animation: sheen 2600ms cubic-bezier(0.4, 0, 0.2, 1) infinite;
    }

    @keyframes sheen {
        0% {
            transform: translateX(-120%);
        }
        60%,
        100% {
            transform: translateX(220%);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .seg,
        .fill {
            transition: none;
        }
        .seg.active::after {
            animation: none;
            opacity: 0;
        }
    }
</style>
