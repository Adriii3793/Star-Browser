<script lang="ts">
    import { emit, listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';
    import CloseButton from '../ui/CloseButton.svelte';
    import Favicon from '../ui/Favicon.svelte';

    interface MediaTab {
        id: string;
        title: string;
        url?: string;
        muted?: boolean;
        audible?: boolean;
    }

    let { onclose, ongoto, ontoggle, onmute }: {
        onclose: () => void;
        ongoto: (tabId: string) => void;
        ontoggle: (tabId: string) => void;
        onmute: (tabId: string) => void;
    } = $props();

    let tabs = $state<MediaTab[]>([]);

    onMount(() => {
        emit('overlay-request-state', { kind: 'media' });
        const unlisten = listen<{ tabs: MediaTab[] }>('overlay-media-state', (e) => {
            tabs = e.payload?.tabs ?? [];
        });
        return () => {
            unlisten.then((off) => off());
        };
    });
</script>

<div class="scrim" role="presentation" onclick={onclose}></div>

<section class="panel" aria-label="Media playing in tabs">
    <header>
        <h1>Playing in tabs</h1>
        <CloseButton label="Close mini player" size="sm" onclick={onclose} />
    </header>

    {#if tabs.length === 0}
        <p class="empty">Nothing is playing right now.</p>
    {:else}
        <ul>
            {#each tabs as tab (tab.id)}
                <li class="row">
                    <span class="icon">
                        {#if tab.url}
                            <Favicon url={tab.url} size={18} />
                        {:else}
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 18V5l12-2v13" /><circle cx="6" cy="18" r="3" /><circle cx="18" cy="16" r="3" /></svg>
                        {/if}
                    </span>
                    <button class="name" type="button" title="Go to tab" onclick={() => ongoto(tab.id)}>
                        {tab.title}
                    </button>
                    <button class="ctl" type="button" aria-label={tab.audible ? 'Pause' : 'Play'} onclick={() => ontoggle(tab.id)}>
                        {#if tab.audible}
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 5v14M16 5v14" /></svg>
                        {:else}
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 4v16l13-8z" /></svg>
                        {/if}
                    </button>
                    <button class="ctl" type="button" aria-label={tab.muted ? 'Unmute' : 'Mute'} onclick={() => onmute(tab.id)}>
                        {#if tab.muted}
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 8a5 5 0 0 1 0 8M6 15H4a1 1 0 0 1-1-1v-4a1 1 0 0 1 1-1h2l4-4v14z" /><path d="M3 3l18 18" /></svg>
                        {:else}
                            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 8a5 5 0 0 1 0 8M6 15H4a1 1 0 0 1-1-1v-4a1 1 0 0 1 1-1h2l4-4v14z" /></svg>
                        {/if}
                    </button>
                </li>
            {/each}
        </ul>
    {/if}
</section>

<style>
    .scrim {
        position: fixed;
        inset: 0;
        background: transparent;
    }

    .panel {
        position: fixed;
        top: 92px;
        right: 14px;
        display: flex;
        flex-direction: column;
        width: min(320px, calc(100vw - 28px));
        max-height: min(420px, calc(100vh - 120px));
        padding: 12px 12px 10px;
        border: 1px solid var(--border);
        border-radius: 14px;
        background: var(--bg-page);
        color: var(--text);
        box-shadow: 0 14px 36px var(--shadow);
        animation: panel-in 0.16s cubic-bezier(0.32, 0.72, 0, 1);
        transform-origin: top right;
    }

    @keyframes panel-in {
        from { opacity: 0; transform: translateY(-6px) scale(0.97); }
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
    }

    h1 {
        margin: 0;
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.05em;
        text-transform: uppercase;
        color: var(--text-muted);
    }

    .empty {
        margin: 0;
        padding: 18px 4px 22px;
        font-size: 12.5px;
        color: var(--text-muted);
        text-align: center;
    }

    ul {
        margin: 0;
        padding: 0;
        list-style: none;
        overflow-y: auto;
        scrollbar-width: thin;
        scrollbar-color: var(--border-strong) transparent;
    }

    .row {
        display: flex;
        align-items: center;
        gap: 9px;
        padding: 7px 8px;
        border-radius: 9px;
    }
    .row:hover { background: var(--hover); }

    .icon {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 22px;
        height: 22px;
        border-radius: 6px;
        background: var(--field);
        color: var(--accent);
        overflow: hidden;
    }
    .icon > svg { width: 12px; height: 12px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }

    .name {
        min-width: 0;
        flex: 1;
        overflow: hidden;
        padding: 0;
        border: none;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12.5px;
        font-weight: 550;
        text-align: left;
        white-space: nowrap;
        text-overflow: ellipsis;
        cursor: pointer;
    }
    .name:hover { color: var(--accent); }

    .ctl {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 26px;
        height: 26px;
        padding: 0;
        border: none;
        border-radius: 7px;
        background: transparent;
        color: var(--text-soft);
        cursor: pointer;
    }
    .ctl:hover { background: var(--field); color: var(--text); }
    .ctl svg { width: 13px; height: 13px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }
</style>
