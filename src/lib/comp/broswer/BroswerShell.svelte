<script lang="ts">
    import TabBar from './TabBar.svelte';
    import AddressBar from './AddressBar.svelte';

    interface TabData {id: string; title: string; url: string;}

    let tabs = $state<TabData[]>([
        {id: crypto.randomUUID(), title: 'New tab', url: '' }
    ]);
    let activeId = $state(tabs[0].id);
    let showChat = $state(false);
    let activeTab = $derived(tabs.find((t) =>t.id === activeId));

    function newTab() {
        const id = crypto.randomUUID();
        tabs.push({id, title: 'New tab', url: ''});
        activeId = id;
    
    }
    function closeTab(id: string) {
        const i = tabs.findIndex((t) => t.id === id);
        if (i === -1) return;
        tabs.splice(i, 1);
        if (tabs.length === 0) { newTab(); return;}
        if (activeId === id) activeId = tabs[Math.max(0, i - 1)].id;
    }
    function selectTab(id: string) { activeId = id;}
    function navigate(input: string) {
        const tab = tabs.find((t) => t.id === activeId);
        if (!tab || !input.trim()) return;
        const isUrl = input.includes('.') && !input.includes(' ');
        const url = isUrl
            ? (input.startsWith('http') ? input :`https://${input}`)
            : `https://duckduckgo.com/?q=${encodeURIComponent(input)}`;

            tab.url = url;
            tab.title = input;
            console.log ('naviga a ', url);

    }    
</script>

<div class="shell">
    <TabBar 
        {tabs}
        {activeId}
        onselect={selectTab}
        onclose={closeTab}
        onnew={newTab}
    />
    <AddressBar 
        url ={activeTab?.url ?? ''}
        onnavigate={navigate}
        onchat = {() => (showChat = !showChat)}
    />

    <div class="body">
        <div class="content">
            {#if activeTab?.url}
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
                <h1> </h1>
                <p class="hint"> </p>
            </div>
            {/if}
        </div>

        {#if showChat}
        <aside class="ai-panel">
            <div class="ai-header">
                <span>Chat</span>
                <button onclick={() => (showChat = false)} aria-label="chiudi chat">
                <i class="ti ti-x"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M18 6l-12 12" />
    <path d="M6 6l12 12" />
</svg></i>
                </button>
            </div>
            <p class="ai-empty">Ai panel will be here soon.</p>
        </aside>
        {/if}
    </div>
</div>

<style>
    .shell {
        display: flex; flex-direction: column;
        height: 100vh; background: var(--bg-chrome);
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
    .hint {font-size: 14px; color: var(--text-muted); margin: 0;}
    .ai-panel {
        width: 320px; background: var(--bg-page);
        margin: 0 8px 8px 0; border-radius:10px;
        display:flex; flex-direction: column; padding: 12px;
    }
    .ai-header {
        display: flex; justify-content: space-between; align-items: center;
        font-weight: 500; color: var(--text); margin-bottom: 12px;
    }
    .ai-header button { border: none; background: transparent; cursor: pointer; color: var(--text-muted); display: flex;}
    .ai-empty {font-size: 13px; color: var(--text-muted);}
</style>