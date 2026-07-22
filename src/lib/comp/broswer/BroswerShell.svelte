<script lang="ts">
    import TabBar from './TabBar.svelte';
    import AddressBar from './AddressBar.svelte';
    import WindowControls from './WindowControls.svelte';
    import Aipanel from '../ai/Aipanel.svelte';
    import  {history} from '$lib/stores/history.svelte';
    import {SvelteSet} from 'svelte/reactivity';
    import {
        openTabWebview,
        navigateTabWebview,
        setTabBounds,
        showTabWebview,
        hideTabWebview,
        closeTabWebview,
        onTabUrlChanged
    } from '$lib/services/webview';
    import { getCurrentWindow } from '@tauri-apps/api/window';
    interface TabData {id: string; title: string; url: string; searchText?: string; hasNavigated?: boolean;}

    let tabs = $state<TabData[]>([
        {id: crypto.randomUUID(), title: 'New tab', url: '', searchText: '', hasNavigated: false }
    ]);
    let activeId = $state(tabs[0].id);
    let showChat = $state(false);
    let activeTab = $derived(tabs.find((t) =>t.id === activeId));

    history.load();

    let contentEl = $state<HTMLElement>();
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
            if (id === activeId) {
                applyBounds(id);
                showTabWebview(id);
            } else {
                hideTabWebview(id);
            }
        }
    });

    $effect (() => {
        if (!contentEl) return;
            const observer: ResizeObserver = new ResizeObserver(() => {
            const rect = currentBounds();
            if (rect && openedViews.has(activeId)) {
                setTabBounds(activeId, rect);
            }
        });
        observer.observe(contentEl);
        return () => observer.disconnect();
    });

    $effect(() => {
        const unlisten = onTabUrlChanged(({ tabId, url }) => {
            const tab = tabs.find((t) => t.id === tabId);
            if (tab) tab.url = url;
        });
        return () => {
            unlisten.then((off) => off());
        };
    });
    

    function newTab() {
        const id = crypto.randomUUID();
        tabs.push({id, title: 'New tab', url: '', searchText: '', hasNavigated: false});
        activeId = id;
    
    }
    function closeTab(id: string) {
        const i = tabs.findIndex((t) => t.id === id);
        if (i === -1) return;
        tabs.splice(i, 1);
        if (openedViews.has(id)) {
            closeTabWebview(id);
            openedViews.delete(id);
        }
        if (tabs.length === 0) {
            getCurrentWindow().close();
            return;
        }
        if (activeId === id) activeId = tabs[Math.max(0, i - 1)].id;
    }
    function selectTab(id: string) { activeId = id;}
    function moveTab(from: number, to: number) {
        const [tab] = tabs.splice(from, 1);
        tabs.splice(to, 0, tab);
    }
    function navigate(input: string) {
        const tab = tabs.find((t) => t.id === activeId);
        if (!tab || !input.trim()) return;
        const isUrl = input.includes('.') && !input.includes(' ');
        const url = isUrl
            ? (input.startsWith('http') ? input :`https://${input}`)
            : `https://www.google.com/search?q=${encodeURIComponent(input)}`;

            tab.url = url;
            tab.searchText = input;
            tab.hasNavigated = true;
            tab.title = input;
            history.record(url, input, isUrl ? null : input);

            const rect = currentBounds();
            if (rect) {
                if (openedViews.has(tab.id)) {
                    navigateTabWebview(tab.id, url);
                } else {
                    openTabWebview(tab.id, url, rect);
                    openedViews.add(tab.id)
                }
            }

    }    
</script>

<div class="shell">
    <div class="topbar">
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
            <div class="home">
                <h1>Search</h1>
                {#if history.entries.length}
                <ul class="recent">
                    {#each history.entries as entry (entry.id)}
                    <li>
                        <button type="button" onclick={() => navigate(entry.query ?? entry.url)}>
                        <span class="term">{entry.query ?? entry.title}</span>
                        <span class="count">{entry.visitCount}</span>
                        </button>
                    </li>
                    {/each}
                </ul>
                {/if}
            </div>
            {/if}
        </div>

        {#if showChat}
        <Aipanel onclose={() => (showChat = false)} />
        {/if}
    </div>
</div>

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

    .body { flex: 1; display: flex; overflow: hidden;}
    .content {
       flex: 1; background: var(--bg-page);
       margin: 0 8px 8px; border-radius:10px;
       display:flex; align-items: center; justify-content: center;
       overflow:hidden;
    }
    .placeholder { text-align: center; color: var(--text-muted);}
    .placeholder i {font-size: 32px;}
    .placeholder code {
        font-size: 12px; color: var(--text-soft);
        background: var(--field); padding: 3px 8px; border-radius: 6px;

    }
    .home { text-align: center;}
    .home h1 {font-size: 28px; font-weight: 500; color: var(--text); margin: 0 0 8px;}
</style>