<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import { setup, SEARCH_ENGINES } from '$lib/stores/setup.svelte';

    let { onnext }: { onnext: () => void } = $props();
</script>

<div class="wrap">
    <h1>Choose your search engine</h1>
    <p class="sub">You can change this anytime in Settings.</p>

    <div class="list" role="radiogroup" aria-label="Search engine">
        {#each SEARCH_ENGINES as engine (engine.id)}
            <button
                class="row"
                type="button"
                role="radio"
                aria-checked={setup.data.searchEngine === engine.id}
                onclick={() => (setup.data.searchEngine = engine.id)}
            >
                <span class="badge" style="background:{engine.color}">{engine.initial}</span>
                <span class="name">{engine.name}</span>
                <span class="radio" class:on={setup.data.searchEngine === engine.id}>
                    {#if setup.data.searchEngine === engine.id}
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 13l4 4L19 7" /></svg>
                    {/if}
                </span>
            </button>
        {/each}
    </div>

    <Button3D label="Continue" onclick={onnext} />
</div>

<style>
    .wrap {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 20px;
    }

    h1 {
        margin: 0;
        font-size: 26px;
        font-weight: 600;
        color: var(--text);
        text-align: center;
    }

    .sub {
        margin: -12px 0 0;
        font-size: 13px;
        color: var(--text-muted);
        text-align: center;
    }

    .list {
        display: flex;
        flex-direction: column;
        width: min(360px, 100%);
        border-radius: 14px;
        overflow: hidden;
        box-shadow: 0 8px 28px rgba(74, 58, 46, 0.1);
    }

    .row {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        border: none;
        border-bottom: 1px solid rgba(74, 58, 46, 0.06);
        background: var(--bg-page, #fff);
        font-family: inherit;
        text-align: left;
        cursor: pointer;
    }

    .row:last-child {
        border-bottom: none;
    }

    .row:hover {
        background: var(--tab-hover, #fbf6f2);
    }

    .badge {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 28px;
        height: 28px;
        border-radius: 8px;
        color: #fff;
        font-size: 13px;
        font-weight: 700;
    }

    .name {
        flex: 1;
        font-size: 14px;
        font-weight: 500;
        color: var(--text);
    }

    .radio {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 22px;
        height: 22px;
        border-radius: 50%;
        border: 1.5px solid rgba(74, 58, 46, 0.22);
        color: #fff;
    }

    .radio.on {
        background: var(--accent, #80A4D4);
        border-color: var(--accent, #80A4D4);
    }

    .radio svg {
        width: 13px;
        height: 13px;
        fill: none;
        stroke: currentColor;
        stroke-width: 3;
        stroke-linecap: round;
        stroke-linejoin: round;
    }
</style>
