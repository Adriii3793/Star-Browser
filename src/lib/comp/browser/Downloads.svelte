<script lang="ts">
    import { onMount } from 'svelte';
    import { listen } from '@tauri-apps/api/event';
    import { downloads } from '$lib/stores/downloads.svelte';

    let { onclose }: { onclose: () => void } = $props();

    downloads.init();

    onMount(() => {
        const unlistenStart = listen('download-started', () => setTimeout(() => downloads.reload(), 150));
        const unlistenFinish = listen('download-finished', () => setTimeout(() => downloads.reload(), 150));
        return () => {
            unlistenStart.then((off) => off());
            unlistenFinish.then((off) => off());
        };
    });

    function timeLabel(at: number): string {
        const d = new Date(at);
        const today = new Date();
        const sameDay = d.toDateString() === today.toDateString();
        const time = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        return sameDay ? time : `${d.toLocaleDateString()} ${time}`;
    }
</script>

<div class="scrim" role="presentation" onclick={onclose}></div>

<section class="panel" aria-label="Downloads">
    <header>
        <h1>Downloads</h1>
        <div class="head-actions">
            {#if downloads.entries.length}
                <button class="ghost" type="button" onclick={() => downloads.clear()}>Clear all</button>
            {/if}
            <button class="close" type="button" aria-label="Close" onclick={onclose}>
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
            </button>
        </div>
    </header>

    <p class="hint">Files are saved to your system Downloads folder.</p>

    {#if downloads.entries.length === 0}
        <div class="empty">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3v12M7 10l5 5 5-5M5 20h14" /></svg>
            <p>No downloads yet</p>
        </div>
    {:else}
        <ul>
            {#each downloads.entries as entry (entry.id)}
                <li class="row" class:failed={entry.state === 'failed'}>
                    <span class="icon" class:ok={entry.state === 'complete'} class:bad={entry.state === 'failed'} aria-hidden="true">
                        {#if entry.state === 'complete'}
                            <svg viewBox="0 0 24 24"><path d="M5 12l4 4L19 7" /></svg>
                        {:else if entry.state === 'failed'}
                            <svg viewBox="0 0 24 24"><path d="M12 8v5M12 16h.01" /><circle cx="12" cy="12" r="9" /></svg>
                        {:else}
                            <svg viewBox="0 0 24 24"><path d="M12 3v12M7 10l5 5 5-5M5 20h14" /></svg>
                        {/if}
                    </span>
                    <span class="meta">
                        <span class="name">{entry.fileName}</span>
                        <span class="sub">
                            {entry.state === 'downloading' ? 'Downloading…' : entry.state === 'complete' ? 'Completed' : 'Failed'}
                            · {timeLabel(entry.at)}
                        </span>
                    </span>
                    <button class="remove" type="button" aria-label="Remove from list" onclick={() => downloads.remove(entry.id)}>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
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
        background: var(--overlay);
    }

    .panel {
        position: fixed;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        display: flex;
        flex-direction: column;
        width: min(560px, calc(100vw - 32px));
        max-height: min(620px, calc(100vh - 48px));
        padding: 18px 20px 16px;
        border: 1px solid var(--border);
        border-radius: 14px;
        background: var(--bg-page);
        color: var(--text);
        box-shadow: 0 18px 48px var(--shadow);
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 2px;
    }

    h1 {
        margin: 0;
        font-size: 17px;
        font-weight: 600;
    }

    .head-actions {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .ghost {
        padding: 6px 12px;
        border: none;
        border-radius: 999px;
        background: var(--field);
        color: var(--text-soft);
        font: inherit;
        font-size: 12px;
        cursor: pointer;
    }
    .ghost:hover { background: var(--tab-hover); }

    .close {
        display: grid;
        place-items: center;
        width: 28px;
        height: 28px;
        padding: 0;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
    }
    .close:hover { background: var(--hover); color: var(--text); }
    .close svg, .remove svg { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; }

    .hint {
        margin: 0 0 12px;
        font-size: 12px;
        color: var(--text-muted);
    }

    .empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        padding: 44px 0 52px;
        color: var(--text-muted);
    }
    .empty svg { width: 34px; height: 34px; fill: none; stroke: currentColor; stroke-width: 1.6; stroke-linecap: round; stroke-linejoin: round; opacity: .6; }
    .empty p { margin: 0; font-size: 13px; }

    ul {
        margin: 0;
        padding: 0;
        list-style: none;
        overflow-y: auto;
    }

    .row {
        display: flex;
        align-items: center;
        gap: 11px;
        padding: 9px 6px;
        border-radius: 10px;
    }
    .row:hover { background: var(--hover); }

    .icon {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 30px;
        height: 30px;
        border-radius: 9px;
        background: color-mix(in srgb, var(--accent) 16%, transparent);
        color: var(--accent);
    }
    .icon.ok { color: #27875a; background: rgba(39, 135, 90, .14); }
    .icon.bad { color: #c14545; background: rgba(193, 69, 69, .13); }
    .icon svg { width: 15px; height: 15px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }

    .meta {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
        flex: 1;
    }
    .name {
        overflow: hidden;
        font-size: 13px;
        font-weight: 550;
        white-space: nowrap;
        text-overflow: ellipsis;
    }
    .sub { font-size: 11.5px; color: var(--text-muted); }

    .remove {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 26px;
        height: 26px;
        padding: 0;
        border: none;
        border-radius: 7px;
        background: transparent;
        color: var(--text-muted);
        cursor: pointer;
        opacity: 0;
    }
    .row:hover .remove { opacity: 1; }
    .remove:hover { background: var(--hover); color: var(--text); }
</style>
