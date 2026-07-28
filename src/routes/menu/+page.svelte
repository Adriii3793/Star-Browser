<script lang="ts">
    import { onMount } from 'svelte';
    import BrowserMenu from '$lib/comp/broswer/BrowserMenu.svelte';
    import { emit, listen } from '@tauri-apps/api/event';

    let zoom = $state(100);
    let anchor = $state({ x: 6, y: 44 });
    // The webview is now reused rather than rebuilt, so the component no longer
    // remounts on each open. Bumping this key remounts it so the open animation replays.
    let openToken = $state(0);

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
        // This webview has its own document, so the shell sends the resolved theme
        // variables and we write them onto <html> here.
        const unlistenTheme = listen<Record<string, string>>('menu-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });

        // Only announce readiness once every listener is registered, otherwise the
        // shell's first burst of state can arrive before we are listening for it.
        Promise.all([unlistenZoom, unlistenPosition, unlistenTheme]).then(() =>
            emit('menu-ready', {})
        );
        return () => {
            unlistenZoom.then((off) => off());
            unlistenPosition.then((off) => off());
            unlistenTheme.then((off) => off());
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
            oncleardata={() => send('cleardata')}
            onprint={() => send('print')}
            onfullscreen={() => send('fullscreen')}
            onsettings={() => send('settings')}
            {zoom}
            onzoomin={() => send('zoomin')}
            onzoomout={() => send('zoomout')}
            onzoomreset={() => send('zoomreset')}
        />
        {/key}
    </div>
</div>

<style>
    /* This route renders in its own webview, which never loads app.css. Without the
       border-box reset the menu rows measure padding on top of their 100% width and
       overflow the panel, which is what produced the stray horizontal scrollbar. */
    :global(*) {
        box-sizing: border-box;
    }

    /* Light defaults for this standalone webview. The shell overwrites them with the
       live theme as inline styles on <html>, which take priority over these. */
    :global(:root) {
        --bg-page: #ffffff;
        --text: #4a3a2e;
        --text-soft: #8a6b57;
        --text-muted: #ac8064;
        --field: #f7f1ec;
        --tab-hover: #fbf6f2;
        --accent: #80a4d4;
        --border: rgba(74, 58, 46, 0.08);
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
