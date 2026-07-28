<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import { setDefaultBrowser } from '$lib/services/setup';

    let { onnext }: { onnext: () => void } = $props();

    type Status = 'idle' | 'working' | 'opened' | 'applied' | 'failed';
    let status = $state<Status>('idle');

    async function makeDefault() {
        status = 'working';
        try {
            // Resolves true only where the OS still allows setting this
            // programmatically (Linux). Windows and macOS open their settings
            // panel instead, so the user completes the change themselves.
            const applied = await setDefaultBrowser();
            status = applied ? 'applied' : 'opened';
        } catch {
            status = 'failed';
        }
    }
</script>

<div class="wrap">
    <h1>Make star your browser</h1>
    <p class="sub">Open links from other apps in star. You can change this later.</p>

    <div class="card">
        <div class="mark" aria-hidden="true">
            <svg viewBox="0 0 24 24">
                <path d="M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z" />
            </svg>
        </div>
        <p class="cardtext">
            {#if status === 'applied'}
                star is now your default browser.
            {:else if status === 'opened'}
                Your system settings are open — pick <strong>star</strong> in the list to finish.
            {:else if status === 'failed'}
                Couldn't open the settings panel. You can set this later from your system settings.
            {:else}
                Windows and macOS ask you to confirm this in system settings.
            {/if}
        </p>

        <button class="action" type="button" disabled={status === 'working'} onclick={makeDefault}>
            {status === 'working' ? 'Opening…' : status === 'idle' ? 'Set as default' : 'Try again'}
        </button>
    </div>

    <Button3D label={status === 'idle' ? 'Skip for now' : 'Continue'} onclick={onnext} />
</div>

<style>
    .wrap {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 18px;
    }

    h1 {
        margin: 0;
        font-size: 26px;
        font-weight: 600;
        color: var(--text);
        text-align: center;
    }

    .sub {
        margin: -10px 0 0;
        font-size: 13px;
        color: var(--text-muted);
        text-align: center;
    }

    .card {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 14px;
        width: min(360px, 100%);
        padding: 24px 20px;
        border-radius: 14px;
        background: var(--field, #f7f1ec);
        text-align: center;
    }

    .mark {
        display: grid;
        place-items: center;
        width: 52px;
        height: 52px;
        border-radius: 14px;
        background: var(--accent, #80A4D4);
        color: #fff;
    }

    .mark svg {
        width: 26px;
        height: 26px;
        fill: currentColor;
    }

    .cardtext {
        margin: 0;
        font-size: 13px;
        line-height: 1.45;
        color: var(--text-soft, #8a6b57);
    }

    .action {
        padding: 9px 18px;
        border: none;
        border-radius: 999px;
        background: var(--bg-page, #fff);
        color: var(--text, #4a3a2e);
        font: inherit;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
    }

    .action:hover:not(:disabled) {
        background: var(--tab-hover, #fbf6f2);
    }

    .action:disabled {
        opacity: 0.6;
        cursor: default;
    }
</style>
