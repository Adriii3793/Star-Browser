<script lang="ts">
    import { onMount } from 'svelte';
    import BrowserMenu from '$lib/comp/browser/BrowserMenu.svelte';
    import { emit, listen } from '@tauri-apps/api/event';

    let zoom = $state(100);
    let anchor = $state({ x: 6, y: 44 });
    let openToken = $state(0);
    let fullscreen = $state(false);

    function send(action: string) {
        emit('menu-action', { action });
    }

    function close() {
        emit('menu-close', {});
    }

    onMount(() => {
        const unlistenZoom = listen<{ zoom: number }>('menu-zoom-sync', (e) => {
            zoom = e.payload.zoom;
        });
        const unlistenPosition = listen<{ x: number; y: number }>('menu-position', (e) => {
            anchor = e.payload;
            openToken += 1;
        });
        const unlistenTheme = listen<Record<string, string>>('menu-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });
        const unlistenState = listen<{ fullscreen?: boolean }>('menu-state', (e) => {
            fullscreen = Boolean(e.payload?.fullscreen);
        });

        Promise.all([unlistenZoom, unlistenPosition, unlistenTheme, unlistenState]).then(() =>
            emit('menu-ready', {})
        );
        return () => {
            unlistenZoom.then((off) => off());
            unlistenPosition.then((off) => off());
            unlistenTheme.then((off) => off());
            unlistenState.then((off) => off());
        };
    });
</script>

<div class="overlay">
    <div class="anchor" style="left:{anchor.x}px; top:{anchor.y}px;">
        {#key openToken}
        <BrowserMenu
            onclose={close}
            onnewtab={() => send('newtab')}
            onhistory={() => send('history')}
            ondownloads={() => send('downloads')}
            onprint={() => send('print')}
            onfullscreen={() => send('fullscreen')}
            onsettings={() => send('settings')}
            {zoom}
            {fullscreen}
            onzoomin={() => send('zoomin')}
            onzoomout={() => send('zoomout')}
            onzoomreset={() => send('zoomreset')}
        />
        {/key}
    </div>
</div>

<style>
    :global(*) {
        box-sizing: border-box;
    }

    :global(:root) {
        --bg-page: #ffffff;
        --text: #4a3a2e;
        --text-soft: #8a6b57;
        --text-muted: #ac8064;
        --field: #f7f1ec;
        --field-strong: #efe6de;
        --tab-active: #ffffff;
        --accent-hover: #6b8fc4;
        --success: #27875a;
        --danger: #c0392b;
        --overlay: rgba(74, 58, 46, 0.28);
        --tab-hover: #fbf6f2;
        --accent: #80a4d4;
        --border: rgba(74, 58, 46, 0.08);
        --border-strong: rgba(74, 58, 46, 0.24);
        --hover: rgba(0, 0, 0, 0.06);
        --accent-contrast: #1c1917;
        --shadow: rgba(74, 58, 46, 0.16);
    }

    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        background: transparent !important;
        font-family: Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto,
            sans-serif;
    }

    :global(body) {
        -webkit-user-select: none;
        user-select: none;
        cursor: default;
    }

    :global(input),
    :global(textarea) {
        -webkit-user-select: text;
        user-select: text;
    }

    .overlay {
        position: fixed;
        inset: 0;
        background: transparent;
    }

    .anchor {
        position: fixed;
    }

    .overlay :global(.menu) {
        position: relative;
        top: 0;
        left: 0;
    }

    .overlay :global(.backdrop) {
        position: fixed;
        inset: 0;
    }
</style>
