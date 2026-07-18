<script lang="ts">
    import { getCurrentWindow } from '@tauri-apps/api/window';
    const window = getCurrentWindow();

    function minimize() {
        window.minimize();

    }
    async function maximize() {
        const max = await window.isMaximized();
        if (max) {
            window.unmaximize();
        }
        else {
            window.maximize();
        }
    }

    function close() {
        window.close();
    }
</script>
<div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-content">
        <slot />
        </div>
        <div class="titlebar-controls">
            <button onclick={minimize} class="control"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M5 12h14" />
        </svg></button>
            <button onclick={maximize} class="control"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="4" y="4" width="16" height="16" rx="2" />
        </svg></button>
            <button onclick={close} class="control close"><svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6l-12 12" />
            <path d="M6 6l12 12" />
        </svg></button>
        
        </div>
    </div>

<style>
    .titlebar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        height: 32px;
        background:trasparent;
        user-select: none;
    }
    .titlebar-controls {display:flex; }
    .control {
        width:46px;
        height:32px;
        border:none;
        background:traspanrent;
        color:white;
        cursor:pointer;
    }
    .control:hover { background:white; }
    .control.close:hover { background:red; }
</style>