<script lang="ts">
    import { prefs } from '$lib/stores/prefs.svelte';

    let { onclose }: { onclose: () => void } = $props();

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }
</script>

<svelte:window on:keydown={onkeydown} />

<div
    class="overlay"
    role="button"
    tabindex="0"
    aria-label="Close settings"
    onclick={(e) => {
        if (e.target === e.currentTarget) onclose();
    }}
    onkeydown={(e) => {
        if ((e.key === 'Enter' || e.key === ' ') && e.target === e.currentTarget) {
            e.preventDefault();
            onclose();
        }
    }}
>
<div
    class="panel"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label="Settings"
>
        <header>
            <h2>Settings</h2>
            <button type="button" class="close" aria-label="Close settings" onclick={onclose}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 6l-12 12" />
                    <path d="M6 6l12 12" />
                </svg>
            </button>
        </header>

        <section>
            <h3>Home Page</h3>
            <p class="hint">Choose which sections appear on the start page.</p>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Show Favorites</span>
                    <span class="rowsub">Quick access tiles at the top of the start page.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={prefs.showFavorites}
                    onchange={(e) => prefs.setFavorites(e.currentTarget.checked)}
                />
            </label>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Show Recently Visited</span>
                    <span class="rowsub">Pages and searches you opened recently.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={prefs.showRecent}
                    onchange={(e) => prefs.setRecent(e.currentTarget.checked)}
                />
            </label>
        </section>
    </div>
</div>


<style>
    .overlay {
        position: fixed;
        inset: 0;
        z-index: 200;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 24px;
        background: rgba(74, 58, 46, 0.28);
    }

    .panel {
        width: min(520px, 100%);
        max-height: min(640px, 100%);
        overflow-y: auto;
        padding: 22px 24px 26px;
        background: var(--bg-page);
        border-radius: 14px;
        box-shadow: 0 18px 48px rgba(74, 58, 46, 0.24);
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 20px;
    }

    h2 {
        margin: 0;
        font-size: 19px;
        font-weight: 600;
        color: var(--text);
    }

    .close {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        padding: 0;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--text-soft);
        cursor: pointer;
    }

    .close:hover {
        background: var(--field);
    }

    h3 {
        margin: 0 0 4px;
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.02em;
        text-transform: uppercase;
        color: var(--text-soft);
    }

    .hint {
        margin: 0 0 12px;
        font-size: 12px;
        color: var(--text-muted);
    }

    .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 12px 14px;
        margin-bottom: 8px;
        background: var(--field);
        border-radius: 10px;
        cursor: pointer;
    }


    .rowtext {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .rowtitle {
        font-size: 13px;
        font-weight: 500;
        color: var(--text);
    }

    .rowsub {
        font-size: 11.5px;
        color: var(--text-muted);
    }

    /* Toggle switch built from a checkbox */
    .switch {
        appearance: none;
        -webkit-appearance: none;
        position: relative;
        flex: 0 0 auto;
        width: 40px;
        height: 22px;
        margin: 0;
        border-radius: 999px;
        background: rgba(74, 58, 46, 0.18);
        cursor: pointer;
        transition: background 0.16s ease;
    }

    .switch::after {
        content: '';
        position: absolute;
        top: 2px;
        left: 2px;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: #fff;
        box-shadow: 0 1px 3px rgba(74, 58, 46, 0.28);
        transition: transform 0.16s ease;
    }

    .switch:checked {
        background: var(--accent);
    }

    .switch:checked::after {
        transform: translateX(18px);
    }
</style>