<script lang="ts">
    import { history } from '$lib/stores/history.svelte';
    import { emit } from '@tauri-apps/api/event';
    import type { HistoryEntry } from '$lib/types';

    let { onclose, onopen }: { onclose: () => void; onopen: (url: string) => void } = $props();

    let query = $state('');

    history.load(200);

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }

    function runSearch() {
        history.search(query);
    }

    function clearEverything() {
        void emit('overlay-clear-data', {});
        query = '';
        history.clear();
    }

    function domainOf(url: string): string {
        try {
            return new URL(url).hostname.replace(/^www\./, '');
        } catch {
            return url;
        }
    }

    function faviconFor(rawUrl: string): string {
        try {
            const u = new URL(rawUrl);
            return `${u.origin}/favicon.ico`;
        } catch {
            return '';
        }
    }

    function faviconFallback(e: Event, rawUrl: string) {
        const img = e.currentTarget as HTMLImageElement;
        if (img.dataset.fallback === 'ddg') {
            img.style.display = 'none';
            return;
        }
        img.dataset.fallback = 'ddg';
        try {
            const host = new URL(rawUrl).hostname;
            img.src = `https://icons.duckduckgo.com/ip3/${host}.ico`;
        } catch {
            img.style.display = 'none';
        }
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
            <button type="button" class="close" aria-label="Close history" onclick={onclose}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 6L6 18" />
                    <path d="M6 6l12 12" />
                </svg>
            </button>
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
                disabled={history.entries.length === 0}
            >
                Clear all
            </button>
        </div>

        <div class="list">
            {#if history.entries.length === 0}
                <p class="empty">No history yet</p>
            {:else}
                {#each groups as group (group.label)}
                    <h3 class="day">{group.label}</h3>
                    {#each group.items as entry (entry.id)}
                        <button class="row" type="button" onclick={() => open(entry.url)}>
                            <span class="icon">
                                <img src={faviconFor(entry.url)} alt="" onerror={(e) => faviconFallback(e, entry.url)} />
                            </span>
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
        padding: 20px 8px 12px 24px;
        background: var(--bg-page);
        border-radius: 14px;
        box-shadow: 0 18px 48px var(--shadow), 0 0 0 1px var(--border);
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
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .empty {
        margin: 24px 0;
        font-size: 13px;
        color: var(--text-muted);
        text-align: center;
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
    }

    .row:hover {
        background: var(--tab-hover);
    }

    .row:hover .title {
        color: var(--accent);
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
    .icon img {
        width: 18px;
        height: 18px;
        border-radius: 3px;
        object-fit: contain;
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
    }
</style>
