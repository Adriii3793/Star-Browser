<script lang="ts">
    import TabBar from './TabBar.svelte';
    import AddressBar from './AddressBar.svelte';
    import WindowControls from './WindowControls.svelte';
    import Aipanel from '../ai/Aipanel.svelte';
    import  {history} from '$lib/stores/history.svelte';
    import {SvelteSet} from 'svelte/reactivity';
    import Settings from './Settings.svelte';
    import FavoriteDialog from './FavoriteDialog.svelte';
    import { prefs } from '$lib/stores/prefs.svelte';
    import { favorites } from '$lib/stores/favorites.svelte';
    import type { Favorite } from '$lib/data/favorites';
    import {
        openTabWebview,
        navigateTabWebview,
        setTabBounds,
        showTabWebview,
        hideTabWebview,
        closeTabWebview,
        onTabUrlChanged,
        openMenuWebview,
        closeMenuWebview,
        tabBack,
        tabForward,
        tabReload,
        setTabZoomWebview
    } from '$lib/services/webview';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { listen, emit } from '@tauri-apps/api/event';
    interface TabData {id: string; title: string; url: string; searchText?: string; hasNavigated?: boolean; hist: string[]; cursor: number; zoom: number;}

    const ZOOM_MIN = 0.25;
    const ZOOM_MAX = 5;
    const ZOOM_STEP = 0.1;

    const initialTabId = crypto.randomUUID() as string;
    let tabs = $state<TabData[]>([
        {id: initialTabId, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1 }
    ]);
    let activeId = $state(initialTabId);
    let showChat = $state(false);
    let activeTab = $derived(tabs.find((t) =>t.id === activeId));

    history.load();

    let showSettings = $state(false);
    let menuOpen = $state(false);

    prefs.init();
    favorites.init();

    let favEditing = $state(false);
    let favDialog = $state<{ id: string | null; title: string; url: string } | null>(null);
    let favGridEl = $state<HTMLElement>();
    let favDragIndex = $state<number | null>(null);
    let favPendingIndex = 0;
    let favStartX = 0;
    let favStartY = 0;
    const FAV_DRAG_THRESHOLD = 4;

    function openAddFavorite() {
        favDialog = { id: null, title: '', url: '' };
    }
    function openEditFavorite(fav: Favorite) {
        favDialog = { id: fav.id, title: fav.title, url: fav.url };
    }
    function saveFavorite(title: string, url: string) {
        if (favDialog?.id) {
            favorites.update(favDialog.id, title, url);
        } else {
            favorites.add(title, url);
        }
        favDialog = null;
    }

    function grabFavorite(index: number, e: PointerEvent) {
        if (e.button !== 0) return;
        favPendingIndex = index;
        favStartX = e.clientX;
        favStartY = e.clientY;
        window.addEventListener('pointermove', dragFavorite);
        window.addEventListener('pointerup', releaseFavorite, { once: true });
        window.addEventListener('pointercancel', releaseFavorite, { once: true });
    }

    function dragFavorite(e: PointerEvent) {
        if (favDragIndex === null) {
            const dx = Math.abs(e.clientX - favStartX);
            const dy = Math.abs(e.clientY - favStartY);
            if (dx < FAV_DRAG_THRESHOLD && dy < FAV_DRAG_THRESHOLD) return;
            favDragIndex = favPendingIndex;
        }

        const tiles = [...(favGridEl?.querySelectorAll('[data-fav-index]') ?? [])] as HTMLElement[];
        const target = tiles.findIndex((el) => {
            const box = el.getBoundingClientRect();
            return e.clientX >= box.left && e.clientX <= box.right && e.clientY >= box.top && e.clientY <= box.bottom;
        });

        if (target === -1 || target === favDragIndex) return;
        favorites.reorder(favDragIndex, target);
        favDragIndex = target;
    }

    function releaseFavorite() {
        window.removeEventListener('pointermove', dragFavorite);
        favDragIndex = null;
    }

    function greetingFor(date: Date): string {
        const h = date.getHours();
        if (h >= 5 && h < 12) return 'Good Morning';
        if (h >= 12 && h < 18) return 'Good Afternoon';
        if (h >= 18 && h < 22) return 'Good Evening';
        return 'Good Night';
    }

    let now = $state(new Date());
    let greeting = $derived(greetingFor(now));

    $effect(() => {
        const timer = setInterval(() => (now = new Date()), 60_000);
        return () => clearInterval(timer);
    });

    function initialOf(text: string): string {
        return (text.trim()[0] ?? '?').toUpperCase();
    }

    function domainOf(url: string): string {
        try {
            return new URL(url).hostname.replace(/^www\./, '');
        } catch {
            return url;
        }
    }

    let contentEl = $state<HTMLElement>();
    let menuBtnEl = $state<HTMLElement>();
        const openedViews = new SvelteSet<string>();
    function currentBounds() {
        return contentEl?.getBoundingClientRect();
    }

    function applyBounds(id: string) {
        const react = currentBounds();
        if (react && openedViews.has(id)) {
            setTabBounds(id, react);
        }
    }

    $effect (() => {
        for (const id of  openedViews) {
            const t = tabs.find((x) => x.id === id);
            if (id === activeId && t?.hasNavigated) {
                applyBounds(id);
                showTabWebview(id);
            } else {
                hideTabWebview(id);
            }
        }
    });

    async function openMenu() {
        if (menuOpen) {
            closeMenu();
            return;
        }
        const btn = menuBtnEl?.getBoundingClientRect();
        const anchorX = btn ? btn.left : 6;
        const anchorY = btn ? btn.bottom + 6 : 44;
        const rect = new DOMRect(0, 0, window.innerWidth, window.innerHeight);
        menuOpen = true;
        await openMenuWebview(rect);
        emit('menu-position', { x: anchorX, y: anchorY });
        emit('menu-zoom-sync', { zoom: Math.round((activeTab?.zoom ?? 1) * 100) });
    }

    function closeMenu() {
        menuOpen = false;
        closeMenuWebview();
    }

    $effect(() => {
        const unlistenAction = listen<{ action: string }>('menu-action', (e) => {
            const action = e.payload.action;
            if (action === 'zoomin') { zoomIn(); return; }
            if (action === 'zoomout') { zoomOut(); return; }
            if (action === 'zoomreset') { zoomReset(); return; }
            closeMenu();
            if (action === 'newtab') newTab();
            else if (action === 'settings') showSettings = true;
            else if (action === 'history') history.load();
        });
        const unlistenClose = listen('menu-close', () => closeMenu());
        return () => {
            unlistenAction.then((off) => off());
            unlistenClose.then((off) => off());
        };
    });

    $effect (() => {
        if (!contentEl) return;
        let frame: number | null = null;
        const observer: ResizeObserver = new ResizeObserver(() => {
            if (frame !== null) return;
            frame = requestAnimationFrame(() => {
                frame = null;
                const rect = currentBounds();
                if (rect && openedViews.has(activeId)) {
                    setTabBounds(activeId, rect);
                }
            });
        });
        observer.observe(contentEl);
        return () => {
            observer.disconnect();
            if (frame !== null) cancelAnimationFrame(frame);
        };
    });

    const lastActionAt = new Map<string, number>();
    const REDIRECT_WINDOW_MS = 1500;

    $effect(() => {
        const unlisten = onTabUrlChanged(({ tabId, url }) => {
            const tab = tabs.find((t) => t.id === tabId);
            if (!tab) return;
            tab.url = url;
            if (tab.hist[tab.cursor] === url) return;
            const sinceAction = Date.now() - (lastActionAt.get(tabId) ?? 0);
            if (sinceAction < REDIRECT_WINDOW_MS && tab.cursor >= 0) {
                tab.hist[tab.cursor] = url;
                return;
            }
            tab.hist = tab?.hist.slice(0, tab.cursor + 1);
            tab.hist.push(url);
            tab.cursor = tab?.hist.length - 1;
        });
        return () => {
            unlisten.then((off) => off());
        };
    });
    

    function newTab() {
        const id = crypto.randomUUID();
        tabs = [...tabs, {id, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1}];
        activeId = id;
    }
    async function closeTab(id: string) {
        const i = tabs.findIndex((t) => t.id === id);
        if (i === -1) return;
        const wasLast = tabs.length === 1;
        tabs = tabs.filter((_, idx) => idx !== i);
        if (openedViews.has(id)) {
            openedViews.delete(id);
            await closeTabWebview(id).catch(() => {});
        }
        if (wasLast) {
            await getCurrentWindow().close().catch(() => {});
            return;
        }
        if (activeId === id) activeId = tabs[Math.max(0, i - 1)].id;
    }
    function selectTab(id: string) { activeId = id;}
    function moveTab(from: number, to: number) {
        const newTabs = [...tabs];
        const [tab] = newTabs.splice(from, 1);
        newTabs.splice(to, 0, tab);
        tabs = newTabs;
    }
    function navigate(input: string) {
        const tab = tabs.find((t) => t.id === activeId);
        if (!tab || !input.trim()) return;
        const isUrl = input.includes('.') && !input.includes(' ');
        const url = isUrl
            ? (input.startsWith('http') ? input :`https://${input}`)
            : `https://www.google.com/search?q=${encodeURIComponent(input)}`;
            tab.hist = tab.hist.slice(0, tab.cursor +1 );
            tab.hist.push(url);
            tab.cursor = tab.hist.length -1;
            tab.url = url;
            tab.searchText = input;
            tab.hasNavigated = true;
            tab.title = input;
            history.record(url, input, isUrl ? null : input);

            lastActionAt.set(tab.id, Date.now());
            const rect = currentBounds();
            if (rect) {
                if (openedViews.has(tab.id)) {
                    navigateTabWebview(tab.id, url);
                } else {
                    openTabWebview(tab.id, url, rect);
                    openedViews.add(tab.id);
                    if (tab.zoom !== 1) setTabZoomWebview(tab.id, tab.zoom);
                }
            }

    }

    function setZoom(factor: number) {
        const tab = activeTab;
        if (!tab) return;
        const clamped = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, Math.round(factor * 100) / 100));
        tab.zoom = clamped;
        if (openedViews.has(tab.id)) setTabZoomWebview(tab.id, clamped);
        if (menuOpen) emit('menu-zoom-sync', { zoom: Math.round(clamped * 100) });
    }
    function zoomIn() { setZoom((activeTab?.zoom ?? 1) + ZOOM_STEP); }
    function zoomOut() { setZoom((activeTab?.zoom ?? 1) - ZOOM_STEP); }
    function zoomReset() { setZoom(1); }

    function handleShortcuts(e: KeyboardEvent) {
        if (!(e.ctrlKey || e.metaKey)) return;
        if (e.key === '=' || e.key === '+') { e.preventDefault(); zoomIn(); }
        else if (e.key === '-' || e.key === '_') { e.preventDefault(); zoomOut(); }
        else if (e.key === '0') { e.preventDefault(); zoomReset(); }
    }
    function goBack() {
        const tab = activeTab;
        if (!tab || tab.cursor < 0) return;
        lastActionAt.set(tab.id, Date.now());
        if (tab.cursor > 0) {
            tab.cursor -=1;
            tab.url = tab.hist[tab.cursor];
            navigateTabWebview(tab.id, tab.hist[tab.cursor]);
        } else if (tab.cursor === 0) {
            tab.cursor =-1;
            tab.hasNavigated = false
        }
    }

    function goForward() {
        const tab = activeTab;
        if (!tab || tab.cursor >= tab.hist.length - 1) return;
        lastActionAt.set(tab.id, Date.now());
        if (tab.cursor === -1 && tab.hist.length > 0) {
            tab.cursor = 0;
            tab.url = tab.hist[0];
            tab.hasNavigated = true;
            navigateTabWebview(tab.id, tab.hist[0]);
        } else if (tab.cursor < tab.hist.length -1) {
            tab.cursor +=1;
            tab.url = tab.hist[tab.cursor];
            navigateTabWebview(tab.id, tab.hist[tab.cursor]);
        }
    }
</script>

<svelte:window onkeydown={handleShortcuts} />

<div class="shell">
    <div class="topbar">
        <div class="menuwrap" bind:this={menuBtnEl}>
            <button
                class="menubtn"
                class:open={menuOpen}
                aria-label="Browser menu"
                aria-haspopup="menu"
                aria-expanded={menuOpen}
                onclick={openMenu}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M6 9l6 6l6 -6" />
                </svg>
            </button>
        </div>
        <TabBar 
            {tabs}
            {activeId}
            onselect={selectTab}
            onclose={closeTab}
            onnew={newTab}
            onreorder={moveTab}
        />
        <div class="drag-region" data-tauri-drag-region></div>
        <WindowControls />
    </div>
    {#key activeTab?.id}
        <AddressBar
            url={activeTab?.url || activeTab?.searchText || ''}
            onnavigate={navigate}
            onchat={() => (showChat = !showChat)}
            onback={goBack}
            onforward={goForward}
            onreload={() => activeTab?.hasNavigated && tabReload(activeId)}
            canBack={(activeTab?.cursor ?? -1) > -1}
            canForward={(activeTab?.cursor ?? -1) < (activeTab?.hist.length ?? 0) - 1}
            onnewtab={newTab}
            onsettings={() => (showSettings = true)}
        />
    {/key}

    <div class="body">
        <div class="content" bind:this={contentEl}>
            {#if activeTab?.hasNavigated}
            <div class="placeholder">
                <i class="ti ti-world"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" />
    <path d="M3.6 9h16.8" />
    <path d="M3.6 15h16.8" />
    <path d="M11.5 3a17 17 0 0 0 0 18" />
    <path d="M12.5 3a17 17 0 0 0 0 18" />
</svg></i>
                <p></p>
                <code>{activeTab.url}</code>
            </div>
            {:else}
            <div class="home" style="zoom: {activeTab?.zoom ?? 1}">
                <div class="home-inner">
                    <h1 class="greeting">{greeting}</h1>

                    {#if prefs.showFavorites}
                    <section class="section">
                        <div class="section-head">
                            <h2>Favorites</h2>
                            <button class="editbtn" type="button" onclick={() => (favEditing = !favEditing)}>
                                {favEditing ? 'Done' : 'Edit'}
                            </button>
                        </div>
                        <div class="fav-grid" bind:this={favGridEl}>
                            {#each favorites.items as fav, i (fav.id)}
                            <div
                                class="fav-tile"
                                class:editing={favEditing}
                                class:dragging={favDragIndex === i}
                                data-fav-index={i}
                                role="group"
                                aria-label={fav.title}
                                onpointerdown={(e) => favEditing && grabFavorite(i, e)}
                            >
                                <button class="fav" type="button" onclick={() => !favEditing && navigate(fav.url)}>
                                    <span class="fav-icon">{initialOf(fav.title)}</span>
                                    <span class="fav-title">{fav.title}</span>
                                </button>
                                {#if favEditing}
                                <div class="fav-controls">
                                    <button class="fav-ctl" type="button" aria-label="Edit favorite" onclick={() => openEditFavorite(fav)}>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4" />
                                        </svg>
                                    </button>
                                    <button class="fav-ctl remove" type="button" aria-label="Remove favorite" onclick={() => favorites.remove(fav.id)}>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M18 6 6 18M6 6l12 12" />
                                        </svg>
                                    </button>
                                </div>
                                {/if}
                            </div>
                            {/each}
                            {#if favEditing}
                            <button class="fav add" type="button" onclick={openAddFavorite}>
                                <span class="fav-icon add-icon">+</span>
                                <span class="fav-title">Add</span>
                            </button>
                            {/if}
                        </div>
                    </section>
                    {/if}

                    {#if prefs.showRecent && history.entries.length}
                    <section class="section">
                        <h2>Recently Visited</h2>
                        <div class="recent-grid">
                            {#each history.entries.slice(0, 9) as entry (entry.id)}
                            <button class="recent" type="button" onclick={() => navigate(entry.query ?? entry.url)}>
                                <span class="recent-icon">{initialOf(entry.query ?? entry.title ?? entry.url)}</span>
                                <span class="recent-text">
                                    <span class="recent-title">{entry.query ?? entry.title}</span>
                                    <span class="recent-url">{domainOf(entry.url)}</span>
                                </span>
                            </button>
                            {/each}
                        </div>
                    </section>
                    {/if}
                </div>
            </div>
            {/if}
        </div>

        {#if showChat}
        <Aipanel onclose={() => (showChat = false)} />
        {/if}
    </div>
</div>

{#if showSettings}
<Settings onclose={() => (showSettings = false)} />
{/if}

{#if favDialog}
<FavoriteDialog
    title={favDialog.title}
    url={favDialog.url}
    onsave={saveFavorite}
    oncancel={() => (favDialog = null)}
/>
{/if}

<style>
    .shell {
        display: flex; flex-direction: column;
        height: 100vh; background: var(--bg-chrome);
        width: 100%;
    }

    .topbar {
        display: flex;
        align-items: stretch;
        min-width: 0;
        height: 38px;
        background: transparent;
        box-sizing: border-box;
    }

    .drag-region {
        flex: 1 1 auto;
        min-width: 72px;
        align-self: stretch;
        -webkit-app-region: drag;
    }

    .menuwrap {
        position: relative;
        display: flex;
        align-items: center;
        flex: 0 0 auto;
        padding: 0 4px 0 6px;
        -webkit-app-region: no-drag;
    }

    .menubtn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        padding: 0;
        border: none;
        border-radius: 999px;
        background: transparent;
        color: var(--text);
        cursor: pointer;
    }

    .menubtn:hover,
    .menubtn.open {
        background: rgba(0, 0, 0, 0.06);
    }

    .body { flex: 1; display: flex; overflow: hidden;}
    .content {
       flex: 1; background: var(--bg-page);
       margin: 0 8px 8px; border-radius:10px;
       display:flex; align-items: stretch; justify-content: stretch;
       overflow:hidden;
    }
    .placeholder { text-align: center; color: var(--text-muted); margin: auto;}
    .placeholder i {font-size: 32px;}
    .placeholder code {
        font-size: 12px; color: var(--text-soft);
        background: var(--field); padding: 3px 8px; border-radius: 6px;

    }

    .home {
        flex: 1;
        overflow-y: auto;
        padding: 48px 32px 56px;
    }

    .home-inner {
        width: 100%;
        max-width: 820px;
        margin: 0 auto;
    }

    .greeting {
        margin: 0 0 40px;
        font-size: 30px;
        font-weight: 500;
        color: var(--text);
        text-align: center;
    }

    .section { margin-bottom: 40px; }
    .section:last-child { margin-bottom: 0; }

    .section h2 {
        margin: 0;
        font-size: 17px;
        font-weight: 600;
        color: var(--text);
    }

    .section-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 16px;
    }

    .editbtn {
        padding: 5px 12px;
        border: none;
        border-radius: 999px;
        background: var(--field);
        color: var(--text-soft);
        font: inherit;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
    }

    .editbtn:hover {
        background: var(--tab-hover);
    }

    .fav-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
        gap: 18px 12px;
    }

    .fav-tile {
        position: relative;
    }

    .fav-tile.editing {
        cursor: grab;
        animation: fav-wiggle 0.22s ease-in-out infinite alternate;
    }

    .fav-tile.dragging {
        opacity: 0.6;
        animation: none;
        z-index: 2;
    }

    @keyframes fav-wiggle {
        from { transform: rotate(-0.6deg); }
        to { transform: rotate(0.6deg); }
    }

    .fav {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 0;
        border: none;
        background: transparent;
        font-family: inherit;
        cursor: pointer;
    }

    .fav-tile.editing .fav {
        cursor: grab;
        pointer-events: none;
    }

    .fav-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 56px;
        height: 56px;
        border-radius: 14px;
        background: var(--field);
        color: var(--accent);
        font-size: 22px;
        font-weight: 600;
        transition: transform 0.14s ease, box-shadow 0.14s ease;
    }

    .fav:hover .fav-icon {
        transform: translateY(-2px);
        box-shadow: 0 6px 16px rgba(74, 58, 46, 0.14);
    }

    .add-icon {
        color: var(--text-muted);
        font-size: 26px;
        font-weight: 400;
    }

    .fav-title {
        max-width: 84px;
        font-size: 11.5px;
        color: var(--text-soft);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .fav-controls {
        position: absolute;
        top: -6px;
        right: 6px;
        display: flex;
        gap: 4px;
    }

    .fav-ctl {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 20px;
        height: 20px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: var(--accent);
        color: #fff;
        cursor: pointer;
    }

    .fav-ctl.remove {
        background: #c0392b;
    }

    .recent-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
        gap: 10px;
    }

    .recent {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px 12px;
        border: 1px solid rgba(74, 58, 46, 0.07);
        border-radius: 10px;
        background: var(--bg-page);
        font-family: inherit;
        text-align: left;
        cursor: pointer;
        transition: background 0.14s ease, border-color 0.14s ease;
    }

    .recent:hover {
        background: var(--tab-hover);
        border-color: rgba(74, 58, 46, 0.12);
    }

    .recent-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 34px;
        height: 34px;
        border-radius: 9px;
        background: var(--field);
        color: var(--accent);
        font-size: 14px;
        font-weight: 600;
    }

    .recent-text {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .recent-title {
        font-size: 13px;
        color: var(--text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .recent-url {
        font-size: 11.5px;
        color: var(--text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    @media (max-width: 560px) {
        .home { padding: 32px 20px 40px; }
        .greeting { font-size: 25px; margin-bottom: 30px; }
    }
</style>