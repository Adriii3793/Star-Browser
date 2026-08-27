<script lang="ts">
    import { faviconSources } from '$lib/services/favicon';

    let {
        url,
        size = 16
    }: {
        url: string;
        size?: number;
    } = $props();

    let sources = $derived(faviconSources(url));

    let attempt = $state<{ url: string; step: number } | null>(null);
    let step = $derived(attempt?.url === url ? attempt.step : 0);
    let current = $derived(sources[step]);

    function failed(src: string) {
        if (src !== current) return;
        attempt = { url, step: step + 1 };
    }
</script>

<span class="favicon" style="--size:{size}px">
    {#if current}
        <img src={current} alt="" onerror={() => failed(current)} />
    {:else}
        <svg viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="9" />
            <path d="M3.6 9h16.8M3.6 15h16.8M11.5 3a17 17 0 0 0 0 18M12.5 3a17 17 0 0 1 0 18" />
        </svg>
    {/if}
</span>

<style>
    .favicon {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: var(--size, 16px);
        height: var(--size, 16px);
    }

    img {
        width: 100%;
        height: 100%;
        border-radius: 4px;
        object-fit: contain;
    }

    svg {
        width: 100%;
        height: 100%;
        fill: none;
        stroke: currentColor;
        stroke-width: 1.75;
        stroke-linecap: round;
        opacity: 0.55;
    }
</style>
