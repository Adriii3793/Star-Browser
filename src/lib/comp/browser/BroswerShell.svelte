<script lang="ts">
    import TabBar, { type TabGroup } from './TabBar.svelte';
    import AddressBar from './AddressBar.svelte';
    import WindowControls from './WindowControls.svelte';
    import Aipanel from '../ai/Aipanel.svelte';
    import  {history} from '$lib/stores/history.svelte';
    import { ai } from '$lib/stores/ai.svelte';
    import { memory } from '$lib/stores/memory.svelte';
    import { reading } from '$lib/stores/reading.svelte';
    import { downloads } from '$lib/stores/downloads.svelte';
    import {SvelteSet} from 'svelte/reactivity';
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
        onTabTitleChanged,
        onTabShortcut,
        onTabAudioChanged,
        onDownloadStarted,
        onDownloadFinished,
        openMenuWebview,
        closeMenuWebview,
        openOverlayWebview,
        warmOverlayWebview,
        closeOverlayWebview,
        tabBack,
        tabForward,
        tabReload,
        tabPrint,
        setTabZoomWebview,
        setTabMutedWebview,
        tabMediaToggle,
        tabStopMedia,
        setAdblockEnabled
    } from '$lib/services/webview';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    import { listen, emit } from '@tauri-apps/api/event';
    import { onMount, untrack } from 'svelte';
    import { platform as detectPlatform } from '@tauri-apps/plugin-os';
    import { windowChrome } from '$lib/stores/windowChrome.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import { imageLuminance, PRESET_THEMES, theme, themeVars } from '$lib/stores/theme.svelte';
    import { loadTabSession, saveTabSession, type TabSession } from '$lib/services/tabs';
    interface TabData {id: string; title: string; url: string; searchText?: string; hasNavigated?: boolean; hist: string[]; cursor: number; zoom: number; favicon?: string; groupId?: string; muted?: boolean; audible?: boolean; hadAudio?: boolean;}
    type OS = 'macos' | 'windows' | 'linux';
    let os = $state<OS>('windows');

    let restored = $state(false);
    let saveTimer: ReturnType<typeof setTimeout> | undefined;

    onMount(() => {
        try {
            const p = detectPlatform();
            os = p === 'macos' ? 'macos' : p === 'linux' ? 'linux' : 'windows';
        } catch {
            os = /Macintosh|Mac OS X/.test(navigator.userAgent) ? 'macos' : 'windows';
        }

        void restoreTabSession();
        void warmOverlayWebview(new DOMRect(0, 0, window.innerWidth, window.innerHeight)).catch(() => {});
        void windowChrome.init();

        const captureFs = (e: KeyboardEvent) => {
            if (e.key === 'F11' || e.code === 'F11' || (e as KeyboardEvent & { keyCode?: number }).keyCode === 122) {
                e.preventDefault();
                e.stopImmediatePropagation();
                void toggleFullscreen();
            }
            if (e.key === 'Escape' && windowChrome.fullscreen) {
                e.preventDefault();
                void windowChrome.setFullscreen(false);
            }
        };
        window.addEventListener('keydown', captureFs, true);

        const unlistenClose = getCurrentWindow().onCloseRequested(async () => {
            clearTimeout(saveTimer);
            window.removeEventListener('keydown', captureFs, true);
            await Promise.race([
                saveTabSession(buildTabSession()).catch(() => {}),
                new Promise((resolve) => setTimeout(resolve, 800))
            ]);
        });

        return () => {
            unlistenClose.then((off) => off());
            window.removeEventListener('keydown', captureFs, true);
            windowChrome.destroy();
        };
    });

    const ZOOM_MIN = 0.25;
    const ZOOM_MAX = 5;
    const ZOOM_STEP = 0.1;

    const initialTabId = crypto.randomUUID() as string;
    let tabs = $state<TabData[]>([
        {id: initialTabId, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1 }
    ]);
    let activeId = $state(initialTabId);
    let tabGroups = $state<TabGroup[]>([]);
    const groupColors = ['#7b9eea', '#d98880', '#70ad7d', '#d6a85d', '#a78bcb', '#56aeb2'];
    let showChat = $state(false);
    let activeTab = $derived(tabs.find((t) =>t.id === activeId));

    history.load();
    reading.init();
    downloads.init();
    void setAdblockEnabled(localStorage.getItem('star.adblock') !== 'off').catch(() => {});

    let showSettings = $state(false);
    let showHistory = $state(false);
    let showDownloads = $state(false);
    let menuOpen = $state(false);
    let favBarExpanded = $state(false);
    let profileOpen = $state(false);
    type DownloadToast = {
        id: string;
        fileName: string;
        state: 'starting' | 'complete' | 'failed';
    };
    let downloadToasts = $state<DownloadToast[]>([]);
    const downloadToastTimers = new Map<string, ReturnType<typeof setTimeout>>();

    function dismissDownloadToast(id: string) {
        const timer = downloadToastTimers.get(id);
        if (timer) clearTimeout(timer);
        downloadToastTimers.delete(id);
        downloadToasts = downloadToasts.filter((toast) => toast.id !== id);
    }

    function scheduleDownloadToastDismiss(id: string, delay = 4500) {
        const existing = downloadToastTimers.get(id);
        if (existing) clearTimeout(existing);
        downloadToastTimers.set(id, setTimeout(() => dismissDownloadToast(id), delay));
    }

    function showDownloadStarted(fileName: string) {
        const id = crypto.randomUUID();
        downloadToasts = [...downloadToasts, { id, fileName, state: 'starting' }];
        scheduleDownloadToastDismiss(id, 12_000);
        downloads.started(fileName);
    }

    function showDownloadFinished(fileName: string, success: boolean) {
        downloads.finished(fileName, success);
        const toast = [...downloadToasts].reverse().find((item) => item.fileName === fileName && item.state === 'starting');
        if (toast) {
            downloadToasts = downloadToasts.map((item) =>
                item.id === toast.id ? { ...item, state: success ? 'complete' : 'failed' } : item
            );
            scheduleDownloadToastDismiss(toast.id);
            return;
        }

        const id = crypto.randomUUID();
        downloadToasts = [...downloadToasts, { id, fileName, state: success ? 'complete' : 'failed' }];
        scheduleDownloadToastDismiss(id);
    }

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

    function openFavorite(url: string) {
        const current = activeTab;
        if (current && !current.hasNavigated) {
            navigate(url);
            return;
        }
        const nextTabs = [...tabs];
        const id = crypto.randomUUID();
        nextTabs.push({ id, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1 });
        tabs = nextTabs;
        activeId = id;
        navigate(url);
    }

    function toggleCurrentFavorite() {
        const tab = activeTab;
        if (!tab || !tab.url) return;
        const existing = favorites.items.find((item) => item.url.trim() === tab.url.trim());
        if (existing) {
            favorites.remove(existing.id);
            return;
        }
        favorites.upsertFromUrl(tab.title || domainOf(tab.url), tab.url);
    }

    $effect(() => {
        if (!profileOpen) return;
        const key = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                profileOpen = false;
                restoreActiveTabVisibility();
            }
        };
        window.addEventListener('keydown', key);
        return () => {
            window.removeEventListener('keydown', key);
        };
    });

    function grabFavorite(index: number, e: PointerEvent) {
        if (e.button !== 0 || !favEditing) return;
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

    function resetAllData() {
        history.clear();
        ai.reset();
        memory.clear();
        reading.clear();
        downloads.clear();
    }

    let now = $state(new Date());
    let greeting = $derived(
        setup.data.name.trim()
            ? `${greetingFor(now)}, ${setup.data.name.trim()}`
            : greetingFor(now)
    );
    let clock = $derived(
        now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    );

    let isFullscreen = $derived(windowChrome.fullscreen);
    let squared = $derived(windowChrome.squared);
    const WINDOW_RADIUS = 12;
    let winRadius = $derived(squared ? 0 : WINDOW_RADIUS);
    let tabCorners = $derived({
        radius: winRadius,
        roundBottomLeft: !squared,
        roundBottomRight: !squared && !showChat
    });

    $effect(() => {
        void windowChrome.fullscreen;
        void windowChrome.maximized;
        untrack(() => {
            sendMenuState();
            reapplyTabBounds();
        });
    });

    setup.load();

    let homeText = $state('var(--text)');
    $effect(() => {
        const bg = setup.data.background;
        if (!bg) {
            homeText = 'var(--text)';
            return;
        }
        imageLuminance(bg).then((l) => (homeText = l > 0.5 ? '#1c1917' : '#ffffff'));
    });

    $effect(() => {
        const timer = setInterval(() => (now = new Date()), 60_000);
        return () => clearInterval(timer);
    });

    function initialOf(text: string): string {
        return (text.trim()[0] ?? '?').toUpperCase();
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
            img.style.opacity = '0';
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

    function domainOf(url: string): string {
        try {
            return new URL(url).hostname.replace(/^www\./, '');
        } catch {
            return url;
        }
    }

    function fallbackTitle(url: string, input = ''): string {
        const text = input.trim();
        if (text && !text.includes('.') && !text.includes(' ')) {
            return text;
        }
        const pdfMatch = /[?&]url=([^&]+\.pdf)/i.exec(url);
        if (pdfMatch) {
            try {
                const file = decodeURIComponent(pdfMatch[1]);
                const name = file.substring(file.lastIndexOf('/') + 1);
                if (name) return name.replace(/\.pdf$/i, '');
            } catch {}
        }
        return domainOf(url);
    }

    let contentEl = $state<HTMLElement>();
    let menuBtnEl = $state<HTMLElement>();
    let profileBtnEl = $state<HTMLElement>();
        const openedViews = new SvelteSet<string>();
    const currentFavoriteActive = $derived(Boolean(activeTab?.url && favorites.hasUrl(activeTab.url)));
    function currentBounds() {
        return contentEl?.getBoundingClientRect();
    }

    function applyBounds(id: string): Promise<void> {
        const react = currentBounds();
        if (react && openedViews.has(id)) {
            return setTabBounds(id, react, tabCorners);
        }
        return Promise.resolve();
    }

    let resyncTimers: ReturnType<typeof setTimeout>[] = [];
    function reapplyTabBounds() {
        for (const timer of resyncTimers) clearTimeout(timer);
        resyncTimers = [];
        if (!openedViews.has(activeId)) return;
        const id = activeId;
        const run = () => {
            if (!openedViews.has(id)) return;
            void applyBounds(id).catch(() => {});
            const zoom = tabs.find((t) => t.id === id)?.zoom ?? 1;
            if (zoom !== 1) setTabZoomWebview(id, zoom);
        };
        run();
        resyncTimers = [120, 320, 700].map((delay) => setTimeout(run, delay));
    }

    let miniPlayerOpen = $state(false);
    let tabMenu = $state<{ x: number; y: number; tabId: string | null; grouped: boolean } | null>(null);
    let tabMenuReset = $state(0);
    function closeTabMenu() {
        if (tabMenu) {
            tabMenu = null;
            tabMenuReset += 1;
        }
    }
    let mediaTabs = $derived(tabs.filter((t) => t.hadAudio && openedViews.has(t.id)));

    $effect(() => {
        if (miniPlayerOpen && mediaTabs.length === 0) miniPlayerOpen = false;
    });

    let visibilityToken = 0;
    const hiddenViews = new Set<string>();

    function markShown(id: string) {
        hiddenViews.delete(id);
        for (const other of openedViews) {
            if (other !== id) hiddenViews.add(other);
        }
    }

    $effect(() => {
        const token = ++visibilityToken;
        const nextActive = activeId;
        const activeVisible = openedViews.has(nextActive) && tabs.find((x) => x.id === nextActive)?.hasNavigated;

        for (const id of openedViews) {
            if (id === nextActive && activeVisible) continue;
            if (hiddenViews.has(id)) continue;
            hiddenViews.add(id);
            hideTabWebview(id).catch(() => hiddenViews.delete(id));
        }

        if (activeVisible) {
            applyBounds(nextActive)
                .then(() => {
                    if (token !== visibilityToken) return;
                    return showTabWebview(nextActive).then(() => markShown(nextActive));
                })
                .catch(() => {});
        }
    });

    let menuAnchor = {x:6,y: 44};

    function sendMenuState() {
        emit('menu-position', menuAnchor);
        emit('menu-zoom-sync', {zoom: Math.round((activeTab?.zoom ?? 1) *100)});
        emit('menu-theme', utilityVars());
        emit('menu-state', { fullscreen: isFullscreen });
    }

    function utilityVars() {
        return { ...themeVars(theme.current), '--win-radius': `${winRadius}px` };
    }

    function sendThemeToUtilityViews() {
        const vars = utilityVars();
        void emit('menu-theme', vars);
        void emit('overlay-theme', vars);
        void emit('permission-theme', vars);
    }

    $effect(() => {
        void theme.preference;
        void theme.current.bg;
        void winRadius;
        sendThemeToUtilityViews();
    });

    async function openMenu() {
        if (menuOpen) {
            closeMenu();
            return;
        }
        const MENU_WIDTH = 300;
        const btn = menuBtnEl?.getBoundingClientRect();
        menuAnchor = {
            x: btn
                ? Math.max(6, Math.min(btn.right - MENU_WIDTH, window.innerWidth - MENU_WIDTH - 6))
                : 6,
            y: btn ? btn.bottom + 6 : 44
        };
        const rect = new DOMRect(0, 0, window.innerWidth, window.innerHeight);
        menuOpen = true;
        await openMenuWebview(rect);
       sendMenuState();
    }

    function closeMenu() {
        menuOpen = false;
        closeMenuWebview();
    }

    type OverlayKind = 'settings' | 'history' | 'downloads' | 'media' | 'profile' | 'tabmenu' | 'groupedit' | null;

    function currentOverlayKind(): OverlayKind {
        if (showSettings) return 'settings';
        if (showHistory) return 'history';
        if (showDownloads) return 'downloads';
        if (miniPlayerOpen) return 'media';
        if (profileOpen) return 'profile';
        if (tabMenu) return 'tabmenu';
        if (renamePop) return 'groupedit';
        return null;
    }

    function sendOverlayShow(kind: OverlayKind) {
        if (!kind) return;
        emit('overlay-show', {
            kind,
            themeId: theme.preference,
            searchEngine: setup.data.searchEngine,
            ...(kind === 'settings' ? { background: setup.data.background } : {})
        });
        emit('overlay-theme', utilityVars());
        if (kind === 'media') pushMediaState();
        if (kind === 'profile') pushProfileState();
        if (kind === 'tabmenu') pushTabMenuState();
        if (kind === 'groupedit') pushGroupEditState();
        if (kind === 'downloads') pushDownloadsState();
    }

    let overlayToken = 0;
    let overlayTask: Promise<void> = Promise.resolve();
    $effect(() => {
        const kind = currentOverlayKind();
        const token = ++overlayToken;
        const rect = new DOMRect(0, 0, window.innerWidth, window.innerHeight);
        overlayTask = overlayTask
            .catch(() => {})
            .then(async () => {
                if (token !== overlayToken) return;
                if (!kind) {
                    await closeOverlayWebview();
                    return;
                }
                await openOverlayWebview(rect);
                if (token === overlayToken) sendOverlayShow(kind);
            })
            .catch(() => {});
    });

    $effect(() => {
        const unlisten = listen<{ kind: OverlayKind }>('overlay-request-state', (e) => {
            if (e.payload?.kind === 'media') pushMediaState();
            else if (e.payload?.kind === 'profile') pushProfileState();
            else if (e.payload?.kind === 'tabmenu') pushTabMenuState();
            else if (e.payload?.kind === 'groupedit') pushGroupEditState();
        });
        return () => unlisten.then((off) => off());
    });

    function pushMediaState() {
        emit('overlay-media-state', {
            tabs: mediaTabs.map((t) => ({
                id: t.id,
                title: t.title,
                favicon: t.favicon,
                muted: t.muted ?? false,
                audible: t.audible ?? false
            }))
        });
    }

    function pushDownloadsState() {
        emit('overlay-downloads-state', { entries: $state.snapshot(downloads.entries) });
    }
    $effect(() => {
        void downloads.entries;
        if (showDownloads) pushDownloadsState();
    });

    function pushProfileState() {
        const box = profileBtnEl?.getBoundingClientRect();
        emit('overlay-profile-state', {
            name: setup.data.name,
            avatar: setup.data.avatar,
            x: box ? Math.max(6, box.left) : 8,
            y: box ? box.bottom + 6 : 48
        });
    }

    function pushGroupEditState() {
        if (!renamePop) return;
        emit('overlay-groupedit-state', {
            groupId: renamePop.groupId,
            name: renamePop.value,
            color: renamePop.color,
            colors: groupColors,
            x: renamePop.x,
            y: renamePop.y
        });
    }

    function pushTabMenuState() {
        if (!tabMenu) return;
        const tab = tabMenu.tabId ? tabs.find((t) => t.id === tabMenu!.tabId) : null;
        emit('overlay-tabmenu-state', {
            x: tabMenu.x,
            y: tabMenu.y,
            tabId: tabMenu.tabId,
            grouped: tabMenu.grouped,
            muted: tab?.muted ?? false,
            tabGroupId: tab?.groupId ?? null,
            groups: tabGroups.map((g) => ({ id: g.id, name: g.name, color: g.color }))
        });
    }

    $effect(() => {
        if (miniPlayerOpen) pushMediaState();
    });
    $effect(() => {
        if (profileOpen) pushProfileState();
    });
    $effect(() => {
        if (tabMenu) pushTabMenuState();
    });
    $effect(() => {
        if (renamePop) pushGroupEditState();
    });
    $effect(() => {
        if (showDownloads) pushDownloadsState();
    });

    function restoreActiveTabVisibility() {
        const t = tabs.find((x) => x.id === activeId);
        if (t?.hasNavigated && openedViews.has(activeId)) {
            applyBounds(activeId)
                .then(() => showTabWebview(activeId))
                .then(() => markShown(activeId))
                .catch(() => {});
        }
    }

    $effect(() => {
        const unlistenClose = listen('overlay-close', () => {
            showSettings = false;
            showHistory = false;
            showDownloads = false;
            miniPlayerOpen = false;
            profileOpen = false;
            renamePop = null;
            closeTabMenu();
            restoreActiveTabVisibility();
        });
        const unlistenClearData = listen('overlay-clear-data', () => {
            resetAllData();
        });
        const unlistenTabAction = listen<{ action: string; tabId: string; groupId?: string; x?: number; y?: number }>('overlay-tab-action', (e) => {
            const { action, tabId, groupId, x, y } = e.payload ?? {};
            closeTabMenu();
            if (action === 'mute' && tabId) toggleMuteTab(tabId);
            else if (action === 'duplicate' && tabId) duplicateTab(tabId);
            else if (action === 'closeothers' && tabId) closeOtherTabs(tabId);
            else if (action === 'creategroup' && tabId) createTabGroup(tabId, { x, y });
            else if (action === 'addtogroup' && tabId && groupId) addExistingTabToGroup(tabId, groupId);
            else if (action === 'removefromgroup' && tabId) ungroupTab(tabId);
        });
        const unlistenGroupSave = listen<{ groupId: string; name: string; color: string }>('overlay-group-save', (e) => {
            const payload = e.payload;
            if (payload?.groupId) renameGroup(payload.groupId, payload.name, payload.color);
            renamePop = null;
        });
        const unlistenGoto = listen<{ tabId: string }>('overlay-goto-tab', (e) => {
            miniPlayerOpen = false;
            restoreActiveTabVisibility();
            if (e.payload?.tabId) void selectTab(e.payload.tabId);
        });
        const unlistenToggle = listen<{ tabId: string }>('overlay-media-toggle', (e) => {
            if (e.payload?.tabId) tabMediaToggle(e.payload.tabId);
        });
        const unlistenMute = listen<{ tabId: string }>('overlay-media-mute', (e) => {
            if (e.payload?.tabId) toggleMuteTab(e.payload.tabId);
        });
        const unlistenOpenSettings = listen('overlay-open-settings', () => {
            profileOpen = false;
            restoreActiveTabVisibility();
            showSettings = true;
        });
        const unlistenProfileUpdated = listen<{ name?: string; avatar?: string | null }>('overlay-profile-updated', (e) => {
            const name = e.payload?.name?.trim();
            if (!name) return;
            setup.data.name = name.slice(0, 40);
            setup.data.avatar = e.payload?.avatar ?? null;
            void setup.save();
        });
        const unlistenNavigate = listen<{ url: string }>('overlay-navigate', (e) => {
            showSettings = false;
            showHistory = false;
            showDownloads = false;
            if (e.payload?.url) navigate(e.payload.url);
        });
        const unlistenReady = listen('overlay-ready', () => {
            sendOverlayShow(currentOverlayKind());
        });
        const unlistenDownloadsState = listen('overlay-request-downloads', () => pushDownloadsState());
        const unlistenDownloadsClear = listen('overlay-downloads-clear', () => downloads.clear());
        const unlistenDownloadsRemove = listen<{ id: string }>('overlay-downloads-remove', (e) => {
            if (e.payload?.id) downloads.remove(e.payload.id);
        });
        const unlistenSettings = listen<{ theme?: string; searchEngine?: string; adblock?: boolean; showFavorites?: boolean; showRecent?: boolean; skipUngroupedTabs?: boolean; aiProvider?: string; background?: string | null }>('settings-changed', (e) => {
            const next = e.payload ?? {};
            let changed = false;

            if (typeof next.adblock === 'boolean') {
                void setAdblockEnabled(next.adblock).catch(() => {});
            }

            if (typeof next.showFavorites === 'boolean') {
                prefs.setFavorites(next.showFavorites);
            }
            if (typeof next.showRecent === 'boolean') {
                prefs.setRecent(next.showRecent);
            }
            if (typeof next.skipUngroupedTabs === 'boolean') {
                prefs.setSkipUngroupedTabs(next.skipUngroupedTabs);
            }

            if (typeof next.aiProvider === 'string' && next.aiProvider !== prefs.aiProvider) {
                prefs.selectProvider(next.aiProvider as typeof prefs.aiProvider);
            }

            if (next.theme && (next.theme === 'system' || PRESET_THEMES.some((item) => item.id === next.theme))) {
                setup.data.theme = next.theme;
                theme.set(next.theme);
                changed = true;
            }
            if (next.background !== undefined) {
                setup.data.background = next.background;
                changed = true;
            }
            if (next.searchEngine && setup.data.searchEngine !== next.searchEngine) {
                setup.data.searchEngine = next.searchEngine;
                changed = true;
            }
            if (changed) {
                void setup.save();
                sendThemeToUtilityViews();
            }
        });
        return () => {
            unlistenClose.then((off) => off());
            unlistenClearData.then((off) => off());
            unlistenGoto.then((off) => off());
            unlistenToggle.then((off) => off());
            unlistenMute.then((off) => off());
            unlistenOpenSettings.then((off) => off());
            unlistenProfileUpdated.then((off) => off());
            unlistenTabAction.then((off) => off());
            unlistenGroupSave.then((off) => off());
            unlistenNavigate.then((off) => off());
            unlistenReady.then((off) => off());
            unlistenDownloadsState.then((off) => off());
            unlistenDownloadsClear.then((off) => off());
            unlistenDownloadsRemove.then((off) => off());
            unlistenSettings.then((off) => off());
        };
    });

    $effect(() => {
        const unlisten = listen('permission-ready', () => sendThemeToUtilityViews());
        return () => unlisten.then((off) => off());
    });

    $effect(() => {
        const unlistenAction = listen<{ action: string }>('menu-action', (e) => {
            const action = e.payload.action;
            if (action === 'zoomin') { zoomIn(); return; }
            if (action === 'zoomout') { zoomOut(); return; }
            if (action === 'zoomreset') { zoomReset(); return; }
            closeMenu();
            if (action === 'newtab') newTab();
            else if (action === 'settings') showSettings = true;
            else if (action === 'history') showHistory = true;
            else if (action === 'downloads') showDownloads = true;
            else if (action === 'print') printActiveTab();
            else if (action === 'fullscreen') toggleFullscreen();
        });
        const unlistenClose = listen('menu-close', () => closeMenu());
        const unlistenReady = listen('menu-ready', () => sendMenuState());
        return () => {
            unlistenAction.then((off) => off());
            unlistenClose.then((off) => off());
            unlistenReady.then((off) => off());
        };
    });

    $effect(() => {
        const unlisten = onTabShortcut(({ tabId, action }) => {
            if (tabId !== activeId) return;
            if (action === 'zoomin') zoomIn();
            else if (action === 'zoomout') zoomOut();
            else if (action === 'zoomreset') zoomReset();
            else if (action === 'newtab') newTab();
            else if (action === 'closetab') closeTab(activeId);
            else if (action === 'history') showHistory = true;
            else if (action === 'print') printActiveTab();
            else if (action === 'fullscreen') void toggleFullscreen();
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    function toggleMaximizeOnDrag(e: MouseEvent) {
        if (e.button !== 0) return;
        windowChrome.toggle();
    }

    async function toggleFullscreen() {
        const win = getCurrentWindow();
        try {
            const next = !(await win.isFullscreen());
            await win.setFullscreen(next);
            await windowChrome.refresh();
            sendMenuState();
        } catch (e) {
            console.error('toggleFullscreen failed, falling back:', e);
            try {
                await windowChrome.setFullscreen(!windowChrome.fullscreen);
            } catch (e2) {
                console.error('windowChrome.setFullscreen fallback also failed:', e2);
            }
        }
    }

    function printActiveTab() {
        if (activeTab?.hasNavigated) tabPrint(activeId);
    }

    $effect (() => {
        if (!contentEl) return;
        let frame: number | null = null;
        const observer: ResizeObserver = new ResizeObserver(() => {
            if (frame !== null) return;
            frame = requestAnimationFrame(() => {
                frame = null;
                const rect = currentBounds();
                if (rect && openedViews.has(activeId)) {
                    setTabBounds(activeId, rect, tabCorners);
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

    const navPending = new Map<string, number>();
    const NAV_WINDOW_MS = 4000;

    $effect(() => {
        const unlisten = onTabUrlChanged(({ tabId, url }) => {
            if (url === 'about:blank' || url.startsWith('data:')) return;
            const tab = tabs.find((t) => t.id === tabId);
            if (!tab) return;
            if (tab.url && tab.url !== url && (tab.audible || tab.hadAudio)) {
                tab.audible = false;
                tab.hadAudio = false;
                if (!tab.muted) {
                }
            }
            tab.url = url;
            tab.favicon = faviconFor(url);
            void reading.capture(url);

            const pending = navPending.get(tabId);
            if (pending !== undefined) {
                navPending.delete(tabId);
                if (Date.now() - pending < NAV_WINDOW_MS) return;
            }

            if (tab.hist[tab.cursor] === url) return;
            const sinceAction = Date.now() - (lastActionAt.get(tabId) ?? 0);
            if (sinceAction < REDIRECT_WINDOW_MS && tab.cursor >= 0) {
                tab.hist[tab.cursor] = url;
                history.record(url, tab.title, null);
                return;
            }
            tab.hist = tab?.hist.slice(0, tab.cursor + 1);
            tab.hist.push(url);
            tab.cursor = tab?.hist.length - 1;
            history.record(url, tab.title, null);
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    $effect(() => {
        const unlisten = onTabAudioChanged(({ tabId, audible, muted }) => {
            const tab = tabs.find((t) => t.id === tabId);
            if (!tab) return;
            tab.audible = audible;
            tab.muted = muted;
            if (audible) tab.hadAudio = true;
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    $effect(() => {
        const unlisten = listen<{ tabId: string; url: string }>('tab-popup', (e) => {
            const url = e.payload?.url;
            if (!url || !/^https?:/i.test(url)) return;
            const id = crypto.randomUUID();
            tabs = [...tabs, { id, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1 }];
            activeId = id;
            navigate(url);
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    $effect(() => {
        const unlisten = onTabTitleChanged(({ tabId, title }) => {
            const tab = tabs.find((t) => t.id === tabId);
            if (!tab || !title.trim()) return;
            if (navPending.has(tabId)) return;
            tab.title = title.trim();
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    $effect(() => {
        const unlistenStart = onDownloadStarted(({ fileName }) => showDownloadStarted(fileName));
        const unlistenFinish = onDownloadFinished(({ fileName, success }) => showDownloadFinished(fileName, success));
        return () => {
            unlistenStart.then((off) => off());
            unlistenFinish.then((off) => off());
            for (const timer of downloadToastTimers.values()) clearTimeout(timer);
            downloadToastTimers.clear();
        };
    });
    

    function newTab() {
        const nextTabs = [...tabs];
        const id = crypto.randomUUID();
        nextTabs.push({ id, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1 });
        tabs = nextTabs;
        activeId = id;
    }

    let closeQueue: Promise<void> = Promise.resolve();
    function closeTab(id: string): Promise<void> {
        closeQueue = closeQueue.then(() => doCloseTab(id)).catch(() => {});
        return closeQueue;
    }
    async function doCloseTab(id: string) {
        const i = tabs.findIndex((t) => t.id === id);
        if (i === -1) return;
        const wasLast = tabs.length === 1;
        const remainingTabs = tabs.filter((_, idx) => idx !== i);
        if (!wasLast && activeId === id) {
            const nextActive = remainingTabs[Math.min(i, remainingTabs.length - 1)];
            activeId = nextActive.id;
            if (openedViews.has(nextActive.id) && nextActive.hasNavigated) {
                await applyBounds(nextActive.id).catch(() => {});
                await showTabWebview(nextActive.id).then(() => markShown(nextActive.id)).catch(() => {});
            }
        }

        tabs = remainingTabs;
        tabGroups = tabGroups.filter((group) => tabs.some((tab) => tab.groupId === group.id));
        lastActionAt.delete(id);
        navPending.delete(id);
        if (openedViews.has(id)) {
            openedViews.delete(id);
            hiddenViews.delete(id);
            await tabStopMedia(id).catch(() => {});
            await closeTabWebview(id).catch(() => {});
        }
        if (wasLast) {
            clearTimeout(saveTimer);
            await saveTabSession(buildTabSession()).catch(() => {});
            await getCurrentWindow().close().catch(() => {});
            return;
        }
    }
    async function selectTab(id: string) {
        const tab = tabs.find((t) => t.id === id);
        if (tab && tab.hasNavigated && tab.url && !openedViews.has(id)) {
            const rect = currentBounds();
            if (rect) {
                await openTabWebview(id, tab.url, rect, tabCorners).catch(() => {});
                openedViews.add(id);
                if (tab.zoom !== 1) setTabZoomWebview(id, tab.zoom);
                if (tab.muted) setTabMutedWebview(id, true);
            }
        }
        activeId = id;
    }

    function toggleMuteTab(id: string) {
        const tab = tabs.find((t) => t.id === id);
        if (!tab) return;
        tab.muted = !tab.muted;
        if (openedViews.has(id)) setTabMutedWebview(id, tab.muted);
    }

    function duplicateTab(id: string) {
        const i = tabs.findIndex((t) => t.id === id);
        if (i === -1) return;
        const src = tabs[i];
        const copy: TabData = {
            ...$state.snapshot(src) as TabData,
            id: crypto.randomUUID(),
            hist: [...src.hist],
            muted: false,
            audible: false,
            hadAudio: false
        };
        const next = [...tabs];
        next.splice(i + 1, 0, copy);
        tabs = next;
        void selectTab(copy.id);
    }

    function closeOtherTabs(id: string) {
        for (const other of tabs.filter((t) => t.id !== id)) {
            void closeTab(other.id);
        }
    }
    function pruneGroups(list: TabData[]) {
        tabGroups = tabGroups.filter((group) => list.some((tab) => tab.groupId === group.id));
    }
    function moveTab(from: number, to: number) {
        const newTabs = [...tabs];
        const [tab] = newTabs.splice(from, 1);
        newTabs.splice(to, 0, tab);

        const before = newTabs[to - 1]?.groupId;
        const after = newTabs[to + 1]?.groupId;
        let groupId: string | undefined;
        if (before && before === after) groupId = before;
        else if (tab.groupId && (before === tab.groupId || after === tab.groupId)) groupId = tab.groupId;
        newTabs[to] = { ...tab, groupId };

        tabs = newTabs;
        pruneGroups(newTabs);
    }
    function newGroupId(): string {
        const id = crypto.randomUUID();
        tabGroups = [...tabGroups, {
            id,
            name: 'Untitled group',
            color: groupColors[tabGroups.length % groupColors.length]
        }];
        return id;
    }
    function createTabGroup(tabId?: string | null, anchor?: { x?: number; y?: number }) {
        const targetId = tabId ?? activeId;
        if (!tabs.some((tab) => tab.id === targetId)) return;
        const id = newGroupId();
        const nextTabs = tabs.map((tab) => tab.id === targetId ? { ...tab, groupId: id } : tab);
        tabs = nextTabs;
        pruneGroups(nextTabs);
        openGroupRename(id, anchor?.x ?? 100, anchor?.y ?? 80);
    }
    function addTabToGroup(groupId: string) {
        const id = crypto.randomUUID();
        const newTab: TabData = {id, title: 'New tab', url: '', searchText: '', hasNavigated: false, hist: [], cursor: -1, zoom: 1, groupId};
        const nextTabs = [...tabs];
        let lastMember = -1;
        nextTabs.forEach((tab, i) => { if (tab.groupId === groupId) lastMember = i; });
        nextTabs.splice(lastMember + 1, 0, newTab);
        tabs = nextTabs;
        activeId = id;
    }
    function addExistingTabToGroup(tabId: string, groupId: string) {
        const tab = tabs.find((t) => t.id === tabId);
        if (!tab || tab.groupId === groupId) return;

        const nextTabs = tabs.filter((t) => t.id !== tabId);
        let lastMember = -1;
        nextTabs.forEach((t, i) => { if (t.groupId === groupId) lastMember = i; });
        nextTabs.splice(lastMember + 1, 0, { ...tab, groupId });

        tabs = nextTabs;
        pruneGroups(nextTabs);
    }
    function renameGroup(groupId: string, name: string, color?: string) {
        const trimmed = name.trim();
        if (!trimmed && !color) return;
        tabGroups = tabGroups.map((g) => g.id === groupId
            ? { ...g, name: trimmed || g.name, color: color ?? g.color }
            : g);
    }

    let renamePop = $state<{ groupId: string; value: string; color: string; x: number; y: number } | null>(null);
    function openGroupRename(groupId: string, x: number, y: number) {
        const group = tabGroups.find((item) => item.id === groupId);
        if (!group) return;
        const width = 250;
        const height = 150;
        renamePop = {
            groupId: group.id,
            value: group.name,
            color: group.color,
            x: Math.max(8, Math.min(x, window.innerWidth - width - 8)),
            y: Math.max(8, Math.min(y, window.innerHeight - height - 8))
        };
    }
    function ungroupTab(tabId: string) {
        const nextTabs = tabs.map((tab) => tab.id === tabId ? { ...tab, groupId: undefined } : tab);
        tabs = nextTabs;
        pruneGroups(nextTabs);
    }
    function groupDroppedTabs(sourceId: string, targetId: string) {
        const source = tabs.find((tab) => tab.id === sourceId);
        const target = tabs.find((tab) => tab.id === targetId);
        if (!source || !target || source.id === target.id) return;

        const groupId = target.groupId ?? newGroupId();

        const nextTabs = tabs
            .filter((tab) => tab.id !== source.id)
            .map((tab) => tab.id === target.id ? { ...tab, groupId } : tab);
        let lastMember = -1;
        nextTabs.forEach((tab, i) => { if (tab.groupId === groupId) lastMember = i; });
        nextTabs.splice(lastMember + 1, 0, { ...source, groupId });

        tabs = nextTabs;
        pruneGroups(nextTabs);
    }

    function buildTabSession(): TabSession {
        const persisted = prefs.skipUngroupedTabs ? tabs.filter((t) => t.groupId) : tabs;
        const activeIndex = Math.max(0, persisted.findIndex((t) => t.id === activeId));
        return {
            tabs: persisted.map((t) => ({
                url: t.url,
                title: t.title,
                searchText: t.searchText ?? null,
                hasNavigated: Boolean(t.hasNavigated),
                hist: t.hist,
                cursor: t.cursor,
                zoom: t.zoom,
                groupId: t.groupId ?? null
            })),
            groups: tabGroups.map((g) => ({ id: g.id, name: g.name, color: g.color })),
            activeIndex
        };
    }

    async function restoreTabSession() {
        try {
            const session = await loadTabSession();
            if (!session || session.tabs.length === 0) return;

            const restoredTabs: TabData[] = session.tabs.map((entry) => ({
                id: crypto.randomUUID(),
                title: entry.title,
                url: entry.url,
                searchText: entry.searchText ?? '',
                hasNavigated: entry.hasNavigated,
                hist: entry.hist,
                cursor: entry.cursor,
                zoom: entry.zoom,
                favicon: entry.hasNavigated ? faviconFor(entry.url) : undefined,
                groupId: entry.groupId ?? undefined
            }));
            const restoredGroups: TabGroup[] = session.groups.map((g) => ({
                id: g.id,
                name: g.name,
                color: g.color
            }));
            const activeIndex = Math.min(Math.max(session.activeIndex, 0), restoredTabs.length - 1);
            const active = restoredTabs[activeIndex];

            tabs = restoredTabs;
            tabGroups = restoredGroups;
            activeId = active.id;

            if (active.hasNavigated && active.url) {
                const rect = currentBounds();
                if (rect) {
                    await openTabWebview(active.id, active.url, rect, tabCorners).catch(() => {});
                    openedViews.add(active.id);
                    if (active.zoom !== 1) setTabZoomWebview(active.id, active.zoom);
                }
            }
        } catch {
        } finally {
            restored = true;
        }
    }

    $effect(() => {
        if (!restored) return;
        const session = buildTabSession();
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            void saveTabSession(session);
        }, 700);
    });

    function navigate(input: string) {
        const tab = tabs.find((t) => t.id === activeId);
        if (!tab || !input.trim()) return;
        const isUrl = input.includes('.') && !input.includes(' ');
        const url = isUrl
            ? (/^https?:\/\//i.test(input) ? input : `https://${input}`)
            : setup.searchUrl(input);
            tab.hist = tab.hist.slice(0, tab.cursor +1 );
            tab.hist.push(url);
            tab.cursor = tab.hist.length -1;
            tab.url = url;
            tab.searchText = input;
            tab.hasNavigated = true;
            tab.title = fallbackTitle(url, input);
            tab.favicon = faviconFor(url);
            history.record(url, input, isUrl ? null : input);

            lastActionAt.set(tab.id, Date.now());
            const rect = currentBounds();
            if (rect) {
                if (openedViews.has(tab.id)) {
                    navigateTabWebview(tab.id, url);
                } else {
                    openTabWebview(tab.id, url, rect, tabCorners);
                    openedViews.add(tab.id);
                    if (tab.zoom !== 1) setTabZoomWebview(tab.id, tab.zoom);
                    if (tab.muted) setTabMutedWebview(tab.id, true);
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
        if (e.key === 'Escape' && miniPlayerOpen) { miniPlayerOpen = false; restoreActiveTabVisibility(); return;}
        if (e.key === 'F11' || e.code === 'F11' || e.keyCode === 122) {
            e.preventDefault();
            e.stopImmediatePropagation();
            e.stopPropagation();
            void toggleFullscreen();
            return;
        }

        if (!(e.ctrlKey || e.metaKey)) return;

        const key = e.key.toLowerCase();

        if (e.shiftKey) return;

        if (key === 'a') {
            const target = e.target as HTMLElement | null;
            const editable = target instanceof HTMLInputElement
                || target instanceof HTMLTextAreaElement
                || target?.isContentEditable;
            if (!editable) e.preventDefault();
            return;
        }

        if (e.key === '=' || e.key === '+') { e.preventDefault(); zoomIn(); }
        else if (e.key === '-' || e.key === '_') { e.preventDefault(); zoomOut(); }
        else if (e.key === '0') { e.preventDefault(); zoomReset(); }
        else if (key === 't') { e.preventDefault(); newTab(); }
        else if (key === 'h') { e.preventDefault(); showHistory = true; }
        else if (key === 'p') { e.preventDefault(); printActiveTab(); }
        else if (key === 'w') { e.preventDefault(); closeTab(activeId); }
    }
    function goBack() {
        const tab = activeTab;
        if (!tab || tab.cursor < 0) return;

        if (tab.cursor === 0) {
            tab.cursor = -1;
            tab.hasNavigated = false;
            tab.title = 'New tab';
            tab.favicon = '';
            return;
        }

        tab.cursor -= 1;
        const target = tab.hist[tab.cursor];
        tab.url = target;
        tab.title = domainOf(target);
        tab.favicon = faviconFor(target);
        lastActionAt.set(tab.id, Date.now());
        navPending.set(tab.id, Date.now());
        tabBack(tab.id);
    }

    function goForward() {
        const tab = activeTab;
        if (!tab || tab.cursor >= tab.hist.length - 1) return;

        if (tab.cursor === -1) {
            tab.cursor = 0;
            tab.url = tab.hist[0];
            tab.title = domainOf(tab.hist[0]);
            tab.favicon = faviconFor(tab.hist[0]);
            tab.hasNavigated = true;
            return;
        }

        tab.cursor += 1;
        const target = tab.hist[tab.cursor];
        tab.url = target;
        tab.title = domainOf(target);
        tab.favicon = faviconFor(target);
        lastActionAt.set(tab.id, Date.now());
        navPending.set(tab.id, Date.now());
        tabForward(tab.id);
    }

    function focusAddressBar() {
        const input = document.querySelector<HTMLInputElement>('.navbar input');
        if (input) {
            input.focus();
            input.select();
        }
    }
    $effect(() => {
        const unlisten = listen<string>('global-shortcut', (e) => {
            const action = e.payload;
            if (action === 'newtab') newTab();
            else if (action === 'closetab') closeTab(activeId);
            else if (action === 'history') showHistory = true;
            else if (action === 'print') printActiveTab();
            else if (action === 'search') focusAddressBar();
            else if (action === 'chat') showChat = !showChat;
        });
        return () => { unlisten.then((off) => off()); };
    });

</script>

<svelte:window onkeydown={handleShortcuts} />

<div class="shell" class:fullscreen={isFullscreen}>
    {#if downloadToasts.length}
        <div class="download-stack" aria-live="polite" aria-label="Download status">
            {#each downloadToasts as toast (toast.id)}
                <div class="download-toast" class:complete={toast.state === 'complete'} class:failed={toast.state === 'failed'}>
                    <span class="download-icon" aria-hidden="true">
                        {#if toast.state === 'complete'}
                            <svg viewBox="0 0 24 24"><path d="M5 12l4 4L19 7" /></svg>
                        {:else if toast.state === 'failed'}
                            <svg viewBox="0 0 24 24"><path d="M12 8v5M12 16h.01" /><circle cx="12" cy="12" r="9" /></svg>
                        {:else}
                            <svg viewBox="0 0 24 24"><path d="M12 3v12M7 10l5 5 5-5M5 20h14" /></svg>
                        {/if}
                    </span>
                    <span class="download-copy">
                        <span class="download-title">{toast.state === 'starting' ? 'Downloading' : toast.state === 'complete' ? 'Download complete' : 'Download failed'}</span>
                        <span class="download-file">{toast.fileName}</span>
                    </span>
                    <button class="download-close" type="button" aria-label="Dismiss download notification" onclick={() => dismissDownloadToast(toast.id)}>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
                    </button>
                </div>
            {/each}
        </div>
    {/if}
    {#snippet menuButton()}
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
    {/snippet}

    {#snippet profileButton()}
        <button bind:this={profileBtnEl} class="top-profile" type="button" title="Profile" aria-haspopup="menu" aria-expanded={profileOpen} onclick={() => (profileOpen = !profileOpen)}>
            <span class="top-avatar">
                {#if setup.data.avatar}
                    <img src={setup.data.avatar} alt="" />
                {:else}
                    {initialOf(setup.data.name || 'Profile')}
                {/if}
            </span>
            <span class="top-username">{setup.data.name.trim() || 'Profile'}</span>
        </button>
    {/snippet}

    {#snippet favoritePages()}
        {#if favorites.items.length}
            <div class="favorite-pages" class:expanded={favBarExpanded} aria-label="Favorite pages">
                {#each (favBarExpanded ? favorites.items : favorites.items.slice(0, 3)) as favorite (favorite.id)}
                    <button class="favorite-page" type="button" title={favorite.title} aria-label={favorite.title} onclick={() => openFavorite(favorite.url)}>
                        <img src={faviconFor(favorite.url)} alt="" onerror={(e) => faviconFallback(e, favorite.url)} />
                    </button>
                {/each}
                {#if favorites.items.length > 3}
                    <button class="favorite-more" type="button" aria-label={favBarExpanded ? 'Show less' : 'Show more favorites'} onclick={() => (favBarExpanded = !favBarExpanded)}>
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d={favBarExpanded ? 'M15 6l-6 6l6 6' : 'M9 6l6 6l-6 6'} /></svg>
                    </button>
                {/if}
            </div>
        {/if}
    {/snippet}

    <div class="topbar" class:mac={os === 'macos'}>
        {#if os === 'macos'}
            <WindowControls platform={os} />
        {:else}
            {@render menuButton()}
        {/if}

        {@render profileButton()}
        {@render favoritePages()}

        <TabBar
            {tabs}
            {activeId}
            groups={tabGroups}
            onselect={selectTab}
            onclose={closeTab}
            onnew={newTab}
            onreorder={moveTab}
            ongroupdrop={groupDroppedTabs}
            onaddtogroup={addTabToGroup}
            oneditgroup={openGroupRename}
            onmute={toggleMuteTab}
            onmenustate={(state) => (tabMenu = state)}
            menuResetToken={tabMenuReset}
        />

        <div class="drag-region" data-tauri-drag-region role="presentation" ondblclick={toggleMaximizeOnDrag}></div>

        {#if os === 'macos'}
            {@render menuButton()}
        {:else}
            <WindowControls platform={os} />
        {/if}
    </div>
    <div class="addressbar-wrap">
        <AddressBar
            url={activeTab?.hasNavigated ? (activeTab?.url || activeTab?.searchText || '') : ''}
            onnavigate={navigate}
            onchat={() => (showChat = !showChat)}
            chatOpen={showChat}
            mediaActive={mediaTabs.length > 0}
            mediaOpen={miniPlayerOpen}
            onmedia={() => (miniPlayerOpen = !miniPlayerOpen)}
            onback={goBack}
            onforward={goForward}
            onreload={() => activeTab?.hasNavigated && tabReload(activeId)}
            canBack={(activeTab?.cursor ?? -1) > -1}
            canForward={(activeTab?.cursor ?? -1) < (activeTab?.hist.length ?? 0) - 1}
            favoriteActive={currentFavoriteActive}
            onfavorite={toggleCurrentFavorite}
        />
    </div>

    <div class="body">
        <div class="content" bind:this={contentEl}>
            {#if activeTab?.hasNavigated}
            <div class="placeholder">
                <i class="ti ti-world"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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
            <div
                class="home"
                class:has-bg={!!setup.data.background}
                style="zoom: {activeTab?.zoom ?? 1};
                       {setup.data.background
                           ? `background-image:url(${setup.data.background});`
                           : ''}
                       --home-text:{homeText};"
            >
                <div class="home-inner">
                    <h1 class="greeting">{greeting}</h1>
                    <p class="clock">{clock}</p>

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
                                onpointerdown={(e) => grabFavorite(i, e)}
                            >
                                <button class="fav" type="button" onclick={() => !favEditing && openFavorite(fav.url)}>
                                    <span class="fav-icon"><img src={faviconFor(fav.url)} alt="" onerror={(e) => faviconFallback(e, fav.url)} /></span>
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
                                <span class="recent-icon">
                                    <img src={faviconFor(entry.url)} alt="" onerror={(e) => faviconFallback(e, entry.url)} />
                                </span>
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
        <Aipanel
            username={setup.data.name.trim() || 'there'}
            pageUrl={activeTab?.hasNavigated ? (activeTab?.url ?? null) : null}
            onclose={() => (showChat = false)}
        />
        {/if}
    </div>
</div>



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
        align-items: center;
        min-width: 0;
        height: 44px;
        background: transparent;
        box-sizing: border-box;
        position: relative;
        z-index: 10001;
        user-select: none;
        -webkit-user-select: none;
    }

    .download-stack {
        position: fixed;
        z-index: 30;
        top: 96px;
        left: 50%;
        transform: translateX(-50%);
        display: grid;
        gap: 8px;
        width: min(360px, calc(100vw - 28px));
        pointer-events: none;
    }

    .download-toast {
        display: flex;
        align-items: center;
        gap: 10px;
        min-height: 44px;
        padding: 8px 12px 8px 8px;
        overflow: hidden;
        border: 1px solid var(--border-strong);
        border-radius: 12px;
        background: var(--bg-page);
        box-shadow: 0 16px 36px -10px color-mix(in srgb, #000 34%, transparent), 0 2px 8px color-mix(in srgb, #000 12%, transparent);
        color: var(--text);
        pointer-events: auto;
        animation: download-in .22s cubic-bezier(0.32, 0.72, 0, 1) forwards;
    }

    .download-icon {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 26px;
        height: 26px;
        border-radius: 8px;
        background: color-mix(in srgb, var(--accent) 17%, transparent);
        color: var(--accent);
    }
    .download-toast.complete .download-icon { color: var(--success); background: color-mix(in srgb, var(--success) 18%, transparent); }
    .download-toast.failed .download-icon { color: var(--danger); background: color-mix(in srgb, var(--danger) 18%, transparent); }
    .download-icon svg, .download-close svg { width: 14px; height: 14px; fill: none; stroke: currentColor; stroke-width: 2.3; stroke-linecap: round; stroke-linejoin: round; }

    .download-copy { display: flex; flex-direction: column; gap: 1px; min-width: 0; flex: 1; }
    .download-title { font-size: 13px; font-weight: 700; line-height: 1.3; }
    .download-file { min-width: 0; overflow: hidden; color: var(--text-muted); font-size: 11.5px; font-weight: 500; line-height: 1.3; text-overflow: ellipsis; white-space: nowrap; }
    .download-close { display: grid; place-items: center; flex: 0 0 auto; width: 24px; height: 24px; padding: 0; border: 0; border-radius: 6px; background: transparent; color: var(--text-muted); cursor: pointer; }
    .download-close:hover { background: var(--hover); color: var(--text); }

    @keyframes download-in {
        from { opacity: 0; transform: translateY(-10px) scale(0.96); }
        to { opacity: 1; transform: translateY(0) scale(1); }
    }

    .drag-region {
        flex: 1 1 auto;
        min-width: 40px;
        align-self: stretch;
        -webkit-app-region: drag;
    }

    .menuwrap {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 36px;
        height: 100%;
        margin-left: 2px;
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
        background: var(--hover);
    }

    .menubtn svg {
        transition: transform 0.22s cubic-bezier(0.32, 0.72, 0, 1);
    }

    .menubtn.open svg {
        transform: rotate(180deg);
    }

    .top-profile {
        display: flex;
        align-items: center;
        gap: 7px;
        flex: 0 0 auto;
        min-width: 34px;
        max-width: 150px;
        height: 30px;
        margin-left: 6px;
        padding: 3px 10px 3px 4px;
        overflow: hidden;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        -webkit-app-region: no-drag;
        transition: background-color .14s ease;
    }
    .top-profile:hover { background: var(--hover); }
    .top-avatar {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 22px;
        height: 22px;
        overflow: hidden;
        border-radius: 50%;
        background: var(--accent);
        color: var(--accent-contrast);
        font-size: 11px;
        font-weight: 800;
    }
    .top-avatar img { width: 100%; height: 100%; object-fit: cover; }
    .top-username { min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; }

    .favorite-pages {
        display: flex;
        align-items: center;
        gap: 2px;
        flex: 0 0 auto;
        height: 32px;
        margin-left: 6px;
        -webkit-app-region: no-drag;
    }
    .favorite-page {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 28px;
        height: 28px;
        padding: 0;
        overflow: hidden;
        border: 0;
        border-radius: 8px;
        background: transparent;
        cursor: default;
        transition: background-color .14s ease;
    }
    .favorite-page:hover { background: var(--hover); }
    .favorite-page img { width: 16px; height: 16px; border-radius: 4px; object-fit: contain; }

    .favorite-pages.expanded {
        max-width: 260px;
        overflow-x: auto;
        scrollbar-width: none;
    }
    .favorite-pages.expanded::-webkit-scrollbar { display: none; }
    .favorite-more {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 22px;
        height: 28px;
        padding: 0;
        border: 0;
        border-radius: 6px;
        background: transparent;
        color: var(--text-soft);
        cursor: pointer;
        transition: background-color .14s ease;
    }
    .favorite-more:hover { background: var(--hover); }

    @media (prefers-reduced-motion: reduce) {
        .menubtn svg {
            transition: none;
        }
        .download-toast {
            animation: none;
        }
    }

    .body { 
        flex: 1; 
        min-height: 0; 
        display: flex; 
        overflow: hidden;
        background: var(--bg-chrome);
        border-bottom-left-radius: var(--win-radius, 0px);
        border-bottom-right-radius: var(--win-radius, 0px);
        padding: 0 var(--win-edge, 0px) var(--win-edge, 0px);
    }

    .content {
       flex: 1; 
       background: var(--bg-chrome);
       margin: 0; 
       border-radius: 0;
       display:flex; 
       align-items: stretch; 
       justify-content: stretch;
       overflow:hidden;
    }

    @media (max-width: 980px) {
        .top-profile { flex-basis: auto; padding-right: 8px; }
        .top-username { max-width: 60px; font-size: 11px; }
        .favorite-pages { margin-left: 2px; }
        .drag-region { min-width: 4px; }
    }    @media (max-width: 720px) {
        .top-username { display: none; }
        .top-profile { flex-basis: 34px; padding-right: 4px; }
        .favorite-pages { display: none; }
        .drag-region { min-width: 4px; }
    }
    .placeholder { text-align: center; color: var(--text-muted); margin: auto;}    .placeholder i {font-size: 32px;}
    .placeholder code {
        font-size: 12px; color: var(--text-soft);
        background: var(--field); padding: 3px 8px; border-radius: 6px;
        -webkit-user-select: text;
        user-select: text;
        cursor: auto;
    }
    .placeholder, .home {
        animation: content-fade 180ms ease-out;
    }
    @keyframes content-fade {
        from {opacity: 0;}
    }
    @media (prefers-reduced-motion: reduce) {
        .placeholder, .home { animation: none !important}
    }
    .home {
        flex: 1;
        overflow-y: auto;
        padding: 48px 32px 56px;
        background-size: cover;
        background-position: center;
        background-repeat: no-repeat;
    }
    .home.has-bg .greeting,
    .home.has-bg .section h2 {
        color: var(--home-text);
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
    }

    .home.has-bg .fav-title {
        color: var(--home-text);
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
    }

    .home.has-bg .editbtn {
        background: rgba(255, 255, 255, 0.88);
        backdrop-filter: blur(6px);
        color: #1c1917;
    }

    .home-inner {
        width: 100%;
        max-width: 820px;
        margin: 0 auto;
    }

    .greeting {
        margin: 0;
        font-size: 30px;
        font-weight: 500;
        color: var(--text);
        text-align: center;
    }

    .clock {
        margin: 6px 0 40px;
        font-size: 14px;
        font-weight: 500;
        color: var(--text-muted);
        text-align: center;
        font-variant-numeric: tabular-nums;
    }
    .home.has-bg .clock {
        color: var(--home-text);
        opacity: 0.85;
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
    }

    .section { margin-bottom: 52px; }
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
        user-select: none;
        -webkit-user-select: none;
        -webkit-tap-highlight-color: transparent;
    }

    .fav-tile.editing {
        cursor: grab;
    }

    .fav-tile.dragging {
        opacity: 0.6;
        z-index: 2;
        cursor: grabbing;
        pointer-events: none;
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
        background: transparent;
        color: var(--accent);
        font-size: 22px;
        font-weight: 600;
        transition: background-color 0.14s ease;
    }
    .fav-icon img {
        width: 28px;
        height: 28px;
        border-radius: 6px;
        object-fit: contain;
    }

    .fav:hover .fav-icon {
        background: color-mix(in srgb, var(--text) 7%, transparent);
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
        color: var(--accent-contrast);
        cursor: pointer;
    }

    .fav-ctl.remove {
        background: color-mix(in srgb, var(--danger) 20%, var(--bg-page));
        color: var(--danger);
    }

    .recent-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
        gap: 12px;
    }

    .recent {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 10px 12px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--bg-page);
        font-family: inherit;
        text-align: left;
        cursor: pointer;
        transition: background 0.14s ease, border-color 0.14s ease;
    }

    .recent:hover {
        background: var(--tab-hover);
        border-color: var(--border-strong);
    }

    .recent-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 24px;
        height: 24px;
        background: transparent;
    }
    .recent-icon img {
        width: 20px;
        height: 20px;
        border-radius: 4px;
        object-fit: contain;
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

    .shell.fullscreen {
        height: 100vh;
        height: 100dvh;
    }

    .shell.fullscreen .content {
        border-radius: 0;
    }
</style>
