<script lang="ts">
    import SetupLoading from './setup-loading.svelte';
    import SetupWelcome from './setup-welcome.svelte';
    import SetupUser from './setup-user.svelte';
    import SetupSearch from './setup-search.svelte';
    import SetupStyle from './setup-style.svelte';
    import Review from './review.svelte';
    import { setup } from '$lib/stores/setup.svelte';

    let { oncomplete }: { oncomplete: () => void } = $props();

    let saving = $state(false);

    async function finish() {
        saving = true;
        await setup.save();
        saving = false;
        oncomplete();
    }

    function handleEnter(e: KeyboardEvent) {
        if (e.key !== 'Enter' || e.repeat) return;
        if (setup.step === 'loading' || saving) return;
        const target = e.target as HTMLElement | null;
        if (target instanceof HTMLButtonElement || target instanceof HTMLTextAreaElement) return;
        e.preventDefault();
        if (setup.step === 'review') void finish();
        else setup.next();
    }
</script>

<svelte:window onkeydown={handleEnter} />

<div class="stage">
    {#key setup.step}
        <div class="step-in">
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
            {:else if setup.step === 'review'}
                <Review onfinish={finish} busy={saving} />
            {/if}
        </div>
    {/key}
</div>

<style>
    .stage {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        min-height: 0;
        padding: 8px 32px 32px;
        overflow-y: auto;
    }

    .step-in {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        margin: auto;
        animation: step-in 320ms cubic-bezier(0.32, 0.72, 0, 1) backwards;
    }

    @keyframes step-in {
        from {
            opacity: 0;
            transform: translateY(12px);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .step-in {
            animation: none;
        }
    }
</style>
