<script lang="ts">
    import { onMount } from 'svelte';
    import BrowserMenu from '$lib/comp/broswer/BrowserMenu.svelte';
    import { emit, listen } from '@tauri-apps/api/event';

    let zoom = $state(100);
    let anchor = $state({ x: 6, y: 44 });

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
        });

        Promise.all([unlistenZoom, unlistenPosition]).then(() => emit('menu-ready', {}));
        return () => {
            unlistenZoom.then((off) => off());
            unlistenPosition.then((off) => off());
        };
    });
</script>

<div class="overlay">
    <div class="anchor" style="left:{anchor.x}px; top:{anchor.y}px;">
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
    </div>
</div>

<style>
    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        background: transparent !important;
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
