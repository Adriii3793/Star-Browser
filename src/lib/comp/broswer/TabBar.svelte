<script lang="ts">
    import Tab from './Tab.svelte';
    interface TabData { 
        id: string; title: string;
    }
    let { tabs, activeId, onselect, onclose, onnew}:
    {tabs: TabData[];activeId: string;
        onselect: (id: string) => void; onclose: (id: string) => void; onnew: () => void } = $props();
</script>

<div class="tabbar">
    {#each tabs as tab (tab.id)}
    <Tab 
        title={tab.title}
        active={tab.id === activeId}
        onselect={() => onselect(tab.id)}
        onclose={() => onclose(tab.id)}
    />
    {/each}
    <button class="new" onclick={onnew} aria-label="Nuovo Tab">
        <i class="ti ti-plus"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M12 5v14" />
    <path d="M5 12h14" />
</svg></i>

    </button>

</div>
<style>
    .tabbar {
        display: flex; align-items: center; gap: 4px;
        padding: 9px 12px; background: var(--bg-chrome);
        flex:1;
    }
    .new {
        width: 30px; height: 30px; border: none; background: transparent;
        border-radius: 8px; color: var(--text-soft); cursor: pointer;
        display: flex; align-items: center; justify-content: center; font-size: 17px;
    }
    .new:hover { background: var(--tab-hover); }
</style>