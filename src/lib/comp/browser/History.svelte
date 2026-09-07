<script lang="ts">
    import { history } from '$lib/stores/history.svelte';
    import { emit } from '@tauri-apps/api/event';
    import type { HistoryEntry } from '$lib/types';
    import CloseButton from '../ui/CloseButton.svelte';
    import Favicon from '../ui/Favicon.svelte';
    import { domainOf } from '$lib/services/favicon';

    let { onclose, onopen }: { onclose: () => void; onopen: (url: string) => void } = $props();

    let query = $state('');

    history.load(200);

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }

    const SEARCH_DEBOUNCE_MS = 160;
    let searchTimer: ReturnType<typeof setTimeout> | undefined;

    function runSearch() {
        clearTimeout(searchTimer);
        searchTimer = setTimeout(() => void history.search(query), SEARCH_DEBOUNCE_MS);
    }

    $effect(() => () => clearTimeout(searchTimer));

    function clearEverything() {
        clearTimeout(searchTimer);
        void emit('overlay-clear-data', {});
        query = '';
        history.clear();
    }


    function dayLabel(ms: number): string {
        const d = new Date(ms);
        const today = new Date();
        const yesterday = new Date(today);
        yesterday.setDate(today.getDate() - 1);
        const sameDay = (a: Date, b: Date) => a.toDateString() === b.toDateString();
        if (sameDay(d, today)) return 'Today';
        if (sameDay(d, yesterday)) return 'Yesterday';
        return d.toLocaleDateString(undefined, { weekday: 'long', month: 'short', day: 'numeric' });
    }

    let groups = $derived.by(() => {
        const out: { label: string; items: HistoryEntry[] }[] = [];
        for (const entry of history.entries) {
            const label = dayLabel(entry.visitedAt);
            const last = out[out.length - 1];
            if (last && last.label === label) last.items.push(entry);
            else out.push({ label, items: [entry] });
        }
        return out;
    });

    function formatTime(ms: number): string {
        return new Date(ms).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    }

    function open(url: string) {
        onopen(url);
        onclose();
    }
</script>

<svelte:window onkeydown={onkeydown} />

<div
    class="overlay"
    role="button"
    tabindex="0"
    aria-label="Close history"
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
    <div class="panel" role="dialog" tabindex="-1" aria-modal="true" aria-label="History">
        <header>
            <h2>History</h2>
            <CloseButton label="Close history" onclick={onclose} />
        </header>

        <div class="tools">
            <div class="search">
                <input
                    type="text"
                    placeholder="Search history"
                    bind:value={query}
                    oninput={runSearch}
                />
            </div>
            <button
                type="button"
                class="clear"
                onclick={clearEverything}
                disabled={history.entries.length === 0 || history.loading}
            >
                Clear all
            </button>
        </div>

        <div class="list">
            {#if history.error}
                <div class="state" role="alert">
                    <p class="state-title">History unavailable</p>
                    <p class="state-detail">{history.error}</p>
                    <button type="button" class="retry" onclick={() => history.retry()}>
                        Try again
                    </button>
                </div>
            {:else if !history.loaded && history.loading}
                <p class="empty">Loading history…</p>
            {:else if history.entries.length === 0}
                <p class="empty">{query.trim() ? `No results for “${query.trim()}”` : 'No history yet'}</p>
            {:else}
                {#each groups as group (group.label)}
                    <h3 class="day">{group.label}</h3>
                    {#each group.items as entry (entry.id)}
                        <button class="row" type="button" onclick={() => open(entry.url)}>
                            <span class="icon"><Favicon url={entry.url} size={18} /></span>
                            <span class="text">
                                <span class="title">{entry.query ?? entry.title}</span>
                                <span class="url">{domainOf(entry.url)}</span>
                            </span>
                            <span class="time">{formatTime(entry.visitedAt)}</span>
                        </button>
                    {/each}
                {/each}
            {/if}
        </div>
    </div>
</div>

<style>
    .overlay {
        position: fixed;
        inset: 0;
        z-index: 9000;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 24px;
        background: transparent;
    }

    .panel {
        display: flex;
        flex-direction: column;
        width: min(680px, 100%);
        max-height: min(720px, 100%);
        padding: 18px;
        background: var(--bg-page);
        border-radius: 14px;
        box-shadow: 0 18px 48px var(--shadow), 0 0 0 1px var(--border);
        animation: panel-in 160ms cubic-bezier(0.2, 0.8, 0.3, 1) both;
        will-change: transform, opacity;
    }

    @keyframes panel-in {
        from {
            opacity: 0;
            transform: translateY(8px) scale(0.985);
        }
        to {
            opacity: 1;
            transform: none;
        }
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 16px;
    }

    h2 {
        margin: 0;
        font-size: 19px;
        font-weight: 600;
        color: var(--text);
    }

    .tools {
        display: flex;
        gap: 10px;
        margin-bottom: 14px;
    }

    .search {
        flex: 1;
    }

    .search input {
        width: 100%;
        padding: 9px 14px;
        border: 1px solid var(--border-strong);
        border-radius: 999px;
        background: var(--field);
        color: var(--text);
        font: inherit;
        font-size: 13px;
        outline: none;
    }

    .search input:focus {
        border-color: var(--accent);
    }

    .clear {
        flex: 0 0 auto;
        padding: 8px 16px;
        border: none;
        border-radius: 999px;
        background: var(--field);
        color: var(--text-soft);
        font: inherit;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
    }

    .clear:hover:not(:disabled) {
        background: var(--tab-hover);
    }

    .clear:disabled {
        opacity: 0.5;
        cursor: default;
    }

    .list {
        flex: 1;
        min-height: 0;
        overflow-y: auto;
        overscroll-behavior: contain;
        display: flex;
        flex-direction: column;
        gap: 2px;
        margin-right: -10px;
        padding-right: 6px;
        scrollbar-width: thin;
        scrollbar-color: var(--border-strong) transparent;
    }

    .list::-webkit-scrollbar {
        width: 8px;
    }

    .list::-webkit-scrollbar-thumb {
        background: var(--border-strong);
        border: 2px solid transparent;
        border-radius: 999px;
        background-clip: content-box;
    }

    .list::-webkit-scrollbar-thumb:hover {
        background: var(--text-muted);
        background-clip: content-box;
    }

    .empty {
        margin: 32px 0;
        font-size: 13px;
        color: var(--text-muted);
        text-align: center;
    }

    .state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
        margin: 28px 0;
        padding: 0 16px;
        text-align: center;
    }

    .state-title {
        margin: 0;
        font-size: 13px;
        font-weight: 600;
        color: var(--text);
    }

    .state-detail {
        margin: 0;
        max-width: 42ch;
        font-size: 12px;
        line-height: 1.5;
        color: var(--text-muted);
        overflow-wrap: anywhere;
    }

    .retry {
        margin-top: 6px;
        padding: 7px 16px;
        border: 1px solid var(--border-strong);
        border-radius: 999px;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12.5px;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 120ms ease, border-color 120ms ease;
    }

    .retry:hover {
        background: var(--hover);
        border-color: var(--accent);
    }

    .row {
        display: flex;
        align-items: center;
        gap: 14px;
        padding: 9px 10px;
        border: none;
        border-radius: 10px;
        background: transparent;
        font-family: inherit;
        text-align: left;
        cursor: pointer;
        transition: background-color 110ms ease, color 110ms ease;
    }

    .row:hover {
        background: var(--tab-hover);
    }

    .row:active {
        background: var(--field-strong);
    }

    .row:hover .title {
        color: var(--accent);
    }

    .row:focus-visible,
    .clear:focus-visible,
    .retry:focus-visible,
    .search input:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 2px;
    }

    .day {
        margin: 16px 10px 6px;
        font-size: 12.5px;
        font-weight: 600;
        letter-spacing: 0.01em;
        color: var(--text-soft);
    }
    .list > .day:first-child { margin-top: 2px; }

    .icon {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 20px;
        height: 20px;
    }
    .text {
        display: flex;
        flex-direction: column;
        gap: 2px;
        flex: 1;
        min-width: 0;
    }

    .title {
        font-size: 13px;
        color: var(--text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .url {
        font-size: 11.5px;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .time {
        flex: 0 0 auto;
        font-size: 11px;
        color: var(--text-muted);
        white-space: nowrap;
        font-variant-numeric: tabular-nums;
    }

    @media (prefers-reduced-motion: reduce) {
        .panel {
            animation: none;
        }

        .row,
        .retry {
            transition: none;
        }
    }
</style>
