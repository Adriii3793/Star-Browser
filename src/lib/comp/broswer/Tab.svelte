<script lang="ts">
    let { title, active = false, onselect, onclose}:
    { title: string; active?: boolean; onselect: () => void; onclose: () => void } = $props();

</script>
<div class="tab" class:active>
    <button class="main" onclick={onselect}>
    <i class="ti ti-world icon"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" />
    <path d="M3.6 9h16.8" />
    <path d="M3.6 15h16.8" />
    <path d="M11.5 3a17 17 0 0 0 0 18" />
    <path d="M12.5 3a17 17 0 0 0 0 18" />
</svg></i>
    <span class="title">{ title }</span>
    </button>
    {#if active}
    <button class="close" onclick={(e) => {e.stopPropagation(); onclose();}} aria-label="chiudi">
    <i class="ti ti-x"><svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M18 6l-12 12" />
    <path d="M6 6l12 12" />
</svg></i>
    </button>
    {/if}
</div>

<style>
    .tab {
        display: flex; align-items: center; gap: 7px;
        padding: 7px 12px; border-radius:10px;
        font-size: 13px; color: var(--text-soft);
        cursor: pointer; user-select: none; max-width: 180px;
    }
    .tab:hover {
        background: var(--tab-hover);
        
    }
    .tab.active {
        background: var(--tab-active); color: var(--text); font-weight:500; 
    }
    .main {
        display: flex; align-items: center; gap:6px;
        border:none;background: transparent; cursor:pointer;
        padding: 8px 12px; font-size:inherit; color: inherit;
        flex: 1;overflow: hidden; font-family:inherit;
    }
    .icon {font-size: 14px;color: var(--text-muted);}
    .tab.active .icon { 
        color: var(--accent);
    }
    .title {
        white-space: nowrap; overflow: hidden; text-overflow: ellipsis; 
    }
    .close { 
        border: none; background: transparent; padding:0; margin-left:2px;
        color:var(--text-muted); cursor: pointer; display:flex;
        }
    .close:hover {
         color: var(--text)
        }
</style>