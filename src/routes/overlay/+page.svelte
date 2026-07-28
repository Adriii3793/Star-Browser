<script lang="ts">
    import { onMount } from 'svelte';
    import Settings from '$lib/comp/broswer/Settings.svelte';
    import History from '$lib/comp/broswer/History.svelte';
    import { emit, listen } from '@tauri-apps/api/event';

    // Hosts Settings and History in their own webview so they float above the page a tab
    // is showing. As DOM inside the main webview they were always painted over by the
    // tab's native child webview.
    type Kind = 'settings' | 'history' | null;
    let kind = $state<Kind>(null);

    function close() {
        kind = null;
        emit('overlay-close', {});
    }

    function openUrl(url: string) {
        kind = null;
        emit('overlay-navigate', { url });
    }

    onMount(() => {
        const unlistenShow = listen<{ kind: Kind }>('overlay-show', (e) => {
            kind = e.payload?.kind ?? null;
        });
        const unlistenTheme = listen<Record<string, string>>('overlay-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });

        Promise.all([unlistenShow, unlistenTheme]).then(() => emit('overlay-ready', {}));
        return () => {
            unlistenShow.then((off) => off());
            unlistenTheme.then((off) => off());
        };
    });
</script>

{#if kind === 'settings'}
    <Settings onclose={close} />
{:else if kind === 'history'}
    <History onclose={close} onopen={openUrl} />
{/if}

<style>
    /* This route renders in its own webview and never loads app.css. */
    :global(*) {
        box-sizing: border-box;
    }

    :global(:root) {
        --bg-page: #ffffff;
        --bg-chrome: #faf7f7;
        --text: #4a3a2e;
        --text-soft: #8a6b57;
        --text-muted: #ac8064;
        --field: #f7f1ec;
        --tab-hover: #fbf6f2;
        --accent: #80a4d4;
        --accent-contrast: #1c1917;
        --border: rgba(74, 58, 46, 0.14);
        --border-strong: rgba(74, 58, 46, 0.24);
        --hover: rgba(0, 0, 0, 0.06);
        --overlay: rgba(74, 58, 46, 0.28);
        --shadow: rgba(74, 58, 46, 0.16);
    }

    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
        background: transparent !important;
        font-family: Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto,
            sans-serif;
        color: var(--text);
    }
</style>
