<script lang="ts">
    let {
        onclose,
        onnewtab,
        onnewwindow,
        onsettings,
        onhistory,
        zoom = 100,
        onzoomin,
        onzoomout,
        onzoomreset
    }: {
        onclose: () => void;
        onnewtab: () => void;
        onnewwindow?: () => void;
        onsettings: () => void;
        onhistory?: () => void;
        zoom?: number;
        onzoomin?: () => void;
        onzoomout?: () => void;
        onzoomreset?: () => void;
    } = $props();

    function run(action?: () => void) {
        action?.();
        onclose();
    }

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }

    // Handle history button click
    function handleHistory() {
        if (onhistory) {
            run(onhistory);
        }
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

    <button class="item" role="menuitem" onclick={() => run(onnewwindow)} disabled={!onnewwindow}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2z" />
            <path d="M4 9h16" />
            <path d="M9 4v5" />
        </svg>
        <span class="label">New Window</span>
        <span class="shortcut">Ctrl+N</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M8 7a4 4 0 1 0 8 0a4 4 0 0 0 -8 0" />
            <path d="M4 21v-2a6 6 0 0 1 6 -6h4a6 6 0 0 1 6 6v2" />
        </svg>
        <span class="label">New Incognito Window</span>
        <span class="shortcut">Ctrl+Shift+N</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M16.555 3.843l3.602 3.602a2.877 2.877 0 0 1 0 4.069l-2.643 2.643a2.877 2.877 0 0 1 -4.069 0l-.301 -.301l-6.558 6.558a2 2 0 0 1 -1.239 .578l-.175 .008h-1.172a1 1 0 0 1 -.993 -.883l-.007 -.117v-1.172a2 2 0 0 1 .467 -1.284l.119 -.13l.414 -.414h2v-2h2v-2l2.144 -2.144l-.301 -.301a2.877 2.877 0 0 1 0 -4.069l2.643 -2.643a2.877 2.877 0 0 1 4.069 0z" />
            <path d="M15 9h.01" />
        </svg>
        <span class="label">Passwords and Autofill</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" onclick={handleHistory} disabled={!onhistory}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 8l0 4l2 2" />
            <path d="M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5" />
        </svg>
        <span class="label">History</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2 -2v-2" />
            <path d="M7 11l5 5l5 -5" />
            <path d="M12 4l0 12" />
        </svg>
        <span class="label">Downloads</span>
        <span class="shortcut">Ctrl+J</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 17.75l-6.172 3.245l1.179 -6.873l-5 -4.867l6.9 -1l3.086 -6.253l3.086 6.253l6.9 1l-5 4.867l1.179 6.873z" />
        </svg>
        <span class="label">Bookmarks and Lists</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 4h6v6h-6z" />
            <path d="M14 4h6v6h-6z" />
            <path d="M4 14h6v6h-6z" />
            <path d="M14 14h6v6h-6z" />
        </svg>
        <span class="label">Tab Groups</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M7 8l-4 4l4 4" />
            <path d="M17 8l4 4l-4 4" />
            <path d="M14 4l-4 16" />
        </svg>
        <span class="label">Extensions</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 7l16 0" />
            <path d="M10 11l0 6" />
            <path d="M14 11l0 6" />
            <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
            <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
        </svg>
        <span class="label">Clear Browsing Data...</span>
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
            <button class="zbtn full" aria-label="Full screen" disabled>⛶</button>
        </span>
    </div>

    <div class="sep"></div>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 17h2a2 2 0 0 0 2 -2v-4a2 2 0 0 0 -2 -2h-14a2 2 0 0 0 -2 2v4a2 2 0 0 0 2 2h2" />
            <path d="M17 9v-4a2 2 0 0 0 -2 -2h-6a2 2 0 0 0 -2 2v4" />
            <path d="M7 13m0 2a2 2 0 0 1 2 -2h6a2 2 0 0 1 2 2v4a2 2 0 0 1 -2 2h-6a2 2 0 0 1 -2 -2z" />
        </svg>
        <span class="label">Print...</span>
        <span class="shortcut">Ctrl+P</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 8v-2a2 2 0 0 1 2 -2h2" />
            <path d="M4 16v2a2 2 0 0 0 2 2h2" />
            <path d="M16 4h2a2 2 0 0 1 2 2v2" />
            <path d="M16 20h2a2 2 0 0 0 2 -2v-2" />
            <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0 -6 0" />
        </svg>
        <span class="label">Search This Tab with Google Lens</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 5h7" />
            <path d="M9 3v2c0 4.418 -2.239 8 -5 8" />
            <path d="M5 9c0 2.144 2.952 3.908 6.7 4" />
            <path d="M12 20l4 -9l4 9" />
            <path d="M19.1 18h-6.2" />
        </svg>
        <span class="label">Translate...</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2z" />
            <path d="M8 11a2 2 0 1 0 4 0a2 2 0 0 0 -4 0" />
            <path d="M16 16l-3.5 -3.5" />
        </svg>
        <span class="label">Find and Edit</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v10a2 2 0 0 1 -2 2h-8" />
            <path d="M3 15a3 3 0 0 1 3 3" />
            <path d="M3 19a7 7 0 0 1 0 -0.01" />
            <path d="M3 12a9 9 0 0 1 9 9" />
        </svg>
        <span class="label">Cast, Save and Share</span>
        <span class="chev">›</span>
    </button>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 21h4l11 -11a1.5 1.5 0 0 0 -4 -4l-11 11v4" />
            <path d="M14.5 5.5l4 4" />
        </svg>
        <span class="label">More Tools</span>
        <span class="chev">›</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" disabled>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 1 0 18 0a9 9 0 0 0 -18 0" />
            <path d="M12 16v.01" />
            <path d="M12 13a2 2 0 0 0 .914 -3.782a1.98 1.98 0 0 0 -2.414 .483" />
        </svg>
        <span class="label">Help</span>
        <span class="chev">›</span>
    </button>

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

    .chev {
        flex: 0 0 auto;
        font-size: 15px;
        line-height: 1;
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