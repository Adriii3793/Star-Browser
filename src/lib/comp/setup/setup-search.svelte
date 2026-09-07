<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import { Check } from '@lucide/svelte';
    import StepShell from './StepShell.svelte';
    import { setup, SEARCH_ENGINES } from '$lib/stores/setup.svelte';
    import EngineLogo from '../ui/EngineLogo.svelte';

    let { onnext }: { onnext: () => void } = $props();

</script>

<StepShell
    title="Choose your search engine"
    subtitle="You can change this anytime in Settings."
    width={360}
>
    <div class="list" role="radiogroup" aria-label="Search engine">
        {#each SEARCH_ENGINES as engine (engine.id)}
            {@const selected = setup.data.searchEngine === engine.id}
            <button
                class="row"
                class:selected
                type="button"
                role="radio"
                aria-checked={selected}
                onclick={() => (setup.data.searchEngine = engine.id)}
            >
                <EngineLogo {engine} size={28} radius={8} />
                <span class="name">{engine.name}</span>
                <span class="radio" class:on={selected}>
                    {#if selected}
                        <Check aria-hidden="true" />
                    {/if}
                </span>
            </button>
        {/each}
    </div>

    {#snippet footer()}
        <Button3D label="Continue" onclick={onnext} />
    {/snippet}
</StepShell>

<style>
    .list {
        display: flex;
        flex-direction: column;
        width: 100%;
        border: 1px solid var(--border);
        border-radius: 14px;
        overflow: hidden;
        box-shadow: 0 8px 28px var(--shadow);
    }

    .row {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 14px 16px;
        border: none;
        border-bottom: 1px solid var(--border);
        background: var(--bg-page, #fff);
        font-family: inherit;
        text-align: left;
        cursor: pointer;
        transition: background-color 0.15s ease;
    }

    .row:last-child {
        border-bottom: none;
    }

    .row:hover,
    .row.selected {
        background: var(--tab-hover, #fbf6f2);
    }

    .row:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: -2px;
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
        border: 1.5px solid var(--border-strong);
        color: var(--accent-contrast, #1c1917);
        transition:
            background-color 0.15s ease,
            border-color 0.15s ease;
    }

    .radio.on {
        background: var(--accent, #80a4d4);
        border-color: var(--accent, #80a4d4);
    }

    .radio :global(svg) {
        width: 13px;
        height: 13px;
        fill: none;
        stroke: currentColor;
        stroke-width: 3;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    @media (prefers-reduced-motion: reduce) {
        .row,
        .radio {
            transition: none;
        }
    }
</style>
