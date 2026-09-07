<script lang="ts">
    import { faviconCandidates, faviconKey, faviconStep, type FaviconProgress } from '$lib/services/favicon';
    import { Globe } from '@lucide/svelte';

    let {
        url,
        iconUrl = null,
        size = 16
    }: {
        url: string;
        iconUrl?: string | null;
        size?: number;
    } = $props();

    let sources = $derived(faviconCandidates(url, iconUrl));

    let progress = $state<FaviconProgress | null>(null);
    let step = $derived(faviconStep(sources, progress));
    let current = $derived(sources[step]);

    function failed(src: string) {
        if (src !== current) return;
        progress = { key: faviconKey(sources), step: step + 1 };
    }
</script>

<span class="favicon" style="--size:{size}px">
    {#if current}
        <img src={current} alt="" onerror={() => failed(current)} />
    {:else}
        <Globe aria-hidden="true" />
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
</style>
