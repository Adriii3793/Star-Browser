<script lang="ts">
    import { onMount } from 'svelte';
    import { Download, Check, CircleAlert, X } from '@lucide/svelte';
    import { emit, listen } from '@tauri-apps/api/event';
    import type { DownloadEntry } from '$lib/stores/downloads.svelte';
    import CloseButton from '../ui/CloseButton.svelte';

    let { onclose }: { onclose: (reason?: 'backdrop') => void } = $props();

    let entries = $state<DownloadEntry[]>([]);

    onMount(() => {
        emit('overlay-request-downloads', {});
        const unlistenState = listen<{ entries: DownloadEntry[] }>('overlay-downloads-state', (e) => {
            entries = e.payload?.entries ?? [];
        });
        return () => {
            unlistenState.then((off) => off());
        };
    });

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }

    function timeLabel(at: number): string {
        const d = new Date(at);
        const today = new Date();
        const sameDay = d.toDateString() === today.toDateString();
        const time = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        return sameDay ? time : `${d.toLocaleDateString()} ${time}`;
    }
</script>

<svelte:window {onkeydown} />

<div class="scrim" role="presentation" onclick={() => onclose('backdrop')}></div>

<section class="panel" aria-label="Downloads">
    <header>
        <h1>Downloads</h1>
        <div class="head-actions">
            {#if entries.length}
                <button class="ghost" type="button" onclick={() => emit('overlay-downloads-clear', {})}>Clear all</button>
            {/if}
            <CloseButton label="Close downloads" onclick={onclose} />
        </div>
    </header>

    <p class="hint">Files are saved to your system Downloads folder.</p>

    {#if entries.length === 0}
        <div class="empty">
            <Download aria-hidden="true" />
            <p>No downloads yet</p>
        </div>
    {:else}
        <ul>
            {#each entries as entry (entry.id)}
                <li class="row" class:failed={entry.state === 'failed'}>
                    <span class="icon" class:ok={entry.state === 'complete'} class:bad={entry.state === 'failed'} aria-hidden="true">
                        {#if entry.state === 'complete'}
                            <Check aria-hidden="true" />
                        {:else if entry.state === 'failed'}
                            <CircleAlert aria-hidden="true" />
                        {:else}
                            <Download aria-hidden="true" />
                        {/if}
                    </span>
                    <span class="meta">
                        <span class="name">{entry.fileName}</span>
                        <span class="sub">
                            {entry.state === 'downloading' ? 'Downloading…' : entry.state === 'complete' ? 'Completed' : 'Failed'}
                            · {timeLabel(entry.at)}
                        </span>
                    </span>
                    <button class="remove" type="button" aria-label="Remove from list" onclick={() => emit('overlay-downloads-remove', { id: entry.id })}>
                        <X aria-hidden="true" />
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
        animation: downloads-in 0.16s cubic-bezier(0.32, 0.72, 0, 1);
    }

    @keyframes downloads-in {
        from {
            opacity: 0;
            transform: translate(-50%, -50%) scale(0.98);
        }
        to {
            opacity: 1;
            transform: translate(-50%, -50%) scale(1);
        }
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

    .remove :global(svg) { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; }

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
    .empty :global(svg) { width: 34px; height: 34px; fill: none; stroke: currentColor; stroke-width: 1.6; stroke-linecap: round; stroke-linejoin: round; opacity: .6; }
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
    .icon.ok { color: var(--success); background: color-mix(in srgb, var(--success) 18%, transparent); }
    .icon.bad { color: var(--danger); background: color-mix(in srgb, var(--danger) 18%, transparent); }
    .icon :global(svg) { width: 15px; height: 15px; fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; stroke-linejoin: round; }

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
