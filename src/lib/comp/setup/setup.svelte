<script lang="ts">
    import SetupLoading from './setup-loading.svelte';
    import SetupWelcome from './setup-welcome.svelte';
    import SetupUser from './setup-user.svelte';
    import SetupSearch from './setup-search.svelte';
    import SetupStyle from './setup-style.svelte';
    import SetupBrowser from './setup-browser.svelte';
    import Review from './review.svelte';
    import { setup, DOT_COUNT } from '$lib/stores/setup.svelte';

    let { oncomplete }: { oncomplete: () => void } = $props();

    let saving = $state(false);

    async function finish() {
        saving = true;
        await setup.save();
        saving = false;
        oncomplete();
    }
</script>

<div class="shell">
    {#if setup.dotIndex >= 0}
        <div class="topbar">
            <button
                class="back"
                type="button"
                aria-label="Back"
                disabled={!setup.canGoBack}
                onclick={() => setup.back()}
            >
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 6l-6 6l6 6" /></svg>
            </button>
            <div class="dots">
                {#each Array(DOT_COUNT) as _, i}
                    <span class="dot" class:active={setup.dotIndex === i}></span>
                {/each}
            </div>
        </div>
    {/if}

    <div class="stage">
        {#if setup.step === 'loading'}
            <SetupLoading ondone={() => setup.next()} />
        {:else if setup.step === 'welcome'}
            <SetupWelcome onnext={() => setup.next()} />
        {:else if setup.step === 'profile'}
            <SetupUser onnext={() => setup.next()} />
        {:else if setup.step === 'search'}
            <SetupSearch onnext={() => setup.next()} />
        {:else if setup.step === 'style'}
            <SetupStyle onnext={() => setup.next()} />
        {:else if setup.step === 'browser'}
            <SetupBrowser onnext={() => setup.next()} />
        {:else if setup.step === 'review'}
            <Review onfinish={finish} busy={saving} />
        {/if}
    </div>
</div>

<style>
    .shell {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: var(--bg-page, #fff);
    }

    .topbar {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        padding: 20px 20px 0;
    }

    .back {
        position: absolute;
        left: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: var(--field, #f7f1ec);
        color: var(--text-soft, #8a6b57);
        cursor: pointer;
        transition: background-color 0.15s ease;
    }

    .back:hover:not(:disabled) {
        background: var(--tab-hover, #fbf6f2);
    }

    .back:disabled {
        opacity: 0.4;
        cursor: default;
    }

    .back svg {
        width: 16px;
        height: 16px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .dots {
        display: flex;
        gap: 6px;
    }

    .dot {
        width: 20px;
        height: 4px;
        border-radius: 999px;
        background: var(--field, #f7f1ec);
        transition: background-color 0.2s ease;
    }

    .dot.active {
        background: var(--accent, #e8734a);
    }

    .stage {
        display: flex;
        align-items: flex-start;
        justify-content: center;
        flex: 1;
        min-height: 0;
        padding: 24px;
        overflow-y: auto;
    }

    .stage > :global(*) {
        margin: auto;
        animation: step-in 250ms ease-in-out;
    }

    @keyframes step-in {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .stage > :global(*) {
            animation: none;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .back,
        .dot {
            transition: none;
        }
    }
</style>
