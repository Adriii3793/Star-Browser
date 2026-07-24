<script lang="ts">
    let {
        onclose,
        onnewtab,
        onhistory,
        oncleardata,
        onprint,
        onfullscreen,
        onsettings,
        zoom = 100,
        onzoomin,
        onzoomout,
        onzoomreset
    }: {
        onclose: () => void;
        onnewtab: () => void;
        onhistory: () => void;
        oncleardata: () => void;
        onprint: () => void;
        onfullscreen: () => void;
        onsettings: () => void;
        zoom?: number;
        onzoomin?: () => void;
        onzoomout?: () => void;
        onzoomreset?: () => void;
    } = $props();

    function run(action: () => void) {
        action();
        onclose();
    }

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }
</script>

<svelte:window onkeydown={onkeydown} />

<div class="backdrop" role="presentation" onclick={onclose}></div>

<div class="menu" role="menu" aria-label="Browser menu">
    <button class="item" role="menuitem" onclick={() => run(onnewtab)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z" />
            <path d="M4 9h16" />
        </svg>
        <span class="label">New Tab</span>
        <span class="shortcut">Ctrl+T</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" onclick={() => run(onhistory)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 8l0 4l2 2" />
            <path d="M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5" />
        </svg>
        <span class="label">History</span>
        <span class="shortcut">Ctrl+H</span>
    </button>

    <button class="item" role="menuitem" onclick={() => run(oncleardata)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 7l16 0" />
            <path d="M10 11l0 6" />
            <path d="M14 11l0 6" />
            <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
            <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
        </svg>
        <span class="label">Clear Browsing Data</span>
        <span class="shortcut">Ctrl+Shift+Del</span>
    </button>

    <div class="sep"></div>

    <div class="item zoomrow" role="presentation">
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 10a7 7 0 1 0 14 0a7 7 0 0 0 -14 0" />
            <path d="M21 21l-6 -6" />
        </svg>
        <span class="label">Zoom</span>
        <span class="zoomctl">
            <button class="zbtn" aria-label="Zoom out" onclick={onzoomout} disabled={!onzoomout}>−</button>
            <button class="zval" aria-label="Reset zoom" onclick={onzoomreset} disabled={!onzoomreset}>{zoom}%</button>
            <button class="zbtn" aria-label="Zoom in" onclick={onzoomin} disabled={!onzoomin}>+</button>
            <button class="zbtn full" aria-label="Full screen" onclick={() => run(onfullscreen)}>⛶</button>
        </span>
    </div>

    <button class="item" role="menuitem" onclick={() => run(onfullscreen)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 8v-2a2 2 0 0 1 2 -2h2" />
            <path d="M4 16v2a2 2 0 0 0 2 2h2" />
            <path d="M16 4h2a2 2 0 0 1 2 2v2" />
            <path d="M16 20h2a2 2 0 0 0 2 -2v-2" />
        </svg>
        <span class="label">Full Screen</span>
        <span class="shortcut">F11</span>
    </button>

    <button class="item" role="menuitem" onclick={() => run(onprint)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 17h2a2 2 0 0 0 2 -2v-4a2 2 0 0 0 -2 -2h-14a2 2 0 0 0 -2 2v4a2 2 0 0 0 2 2h2" />
            <path d="M17 9v-4a2 2 0 0 0 -2 -2h-6a2 2 0 0 0 -2 2v4" />
            <path d="M7 13m0 2a2 2 0 0 1 2 -2h6a2 2 0 0 1 2 2v4a2 2 0 0 1 -2 2h-6a2 2 0 0 1 -2 -2z" />
        </svg>
        <span class="label">Print...</span>
        <span class="shortcut">Ctrl+P</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" onclick={() => run(onsettings)}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.572 1.065c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.065 -2.572c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1 .608 2.296 .07 2.572 -1.065" />
            <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />
        </svg>
        <span class="label">Settings</span>
    </button>
</div>

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        z-index: 90;
        background: transparent;
    }

    .menu {
        position: absolute;
        top: calc(100% + 6px);
        left: 0;
        z-index: 1000;
        width: 300px;
        max-height: min(560px, calc(100vh - 90px));
        overflow-y: auto;
        padding: 6px;
        background: #fff;
        border: 1px solid rgba(74, 58, 46, 0.08);
        border-radius: 12px;
        box-shadow: 0 12px 32px rgba(74, 58, 46, 0.16);
    }

    .item {
        display: flex;
        align-items: center;
        gap: 12px;
        width: 100%;
        padding: 8px 10px;
        border: none;
        border-radius: 8px;
        background: transparent;
        font-family: inherit;
        font-size: 13px;
        color: var(--text);
        text-align: left;
        cursor: pointer;
    }

    .item:hover:not(:disabled):not(.zoomrow) {
        background: var(--tab-hover);
    }

    .item:disabled {
        color: var(--text-muted);
        opacity: 0.55;
        cursor: default;
    }

    .ico {
        flex: 0 0 auto;
        color: var(--text-soft);
    }

    .item:disabled .ico {
        color: var(--text-muted);
    }

    .label {
        flex: 1;
        min-width: 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .shortcut {
        flex: 0 0 auto;
        font-size: 11px;
        color: var(--text-muted);
    }

    .sep {
        height: 1px;
        margin: 6px 4px;
        background: rgba(74, 58, 46, 0.08);
    }

    .zoomrow {
        cursor: default;
    }

    .zoomctl {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .zbtn {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        padding: 0;
        border: none;
        border-radius: 6px;
        background: var(--field);
        font-family: inherit;
        font-size: 13px;
        color: var(--text);
        cursor: pointer;
    }

    .zbtn:hover:not(:disabled) {
        background: var(--tab-hover);
    }

    .zbtn:disabled {
        color: var(--text-muted);
        opacity: 0.55;
        cursor: default;
    }

    .zval {
        min-width: 38px;
        padding: 2px 0;
        border: none;
        border-radius: 6px;
        background: transparent;
        font-family: inherit;
        font-size: 12px;
        color: var(--text-soft);
        text-align: center;
        cursor: pointer;
    }

    .zval:hover:not(:disabled) {
        background: var(--tab-hover);
    }

    .zval:disabled {
        cursor: default;
    }
</style>
