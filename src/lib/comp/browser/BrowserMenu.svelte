<script lang="ts">
    let {
        onclose,
        onnewtab,
        onhistory,
        ondownloads,
        onprint,
        onfullscreen,
        onsettings,
        zoom = 100,
        fullscreen = false,
        onzoomin,
        onzoomout,
        onzoomreset
    }: {
        onclose: () => void;
        onnewtab: () => void;
        onhistory: () => void;
        ondownloads?: () => void;
        onprint: () => void;
        onfullscreen: () => void;
        onsettings: () => void;
        zoom?: number;
        fullscreen?: boolean;
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

    <button class="item" role="menuitem" onclick={() => run(ondownloads ?? (() => {}))}>
        <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 3v12" />
            <path d="M7 10l5 5 5-5" />
            <path d="M5 20h14" />
        </svg>
        <span class="label">Downloads</span>
    </button>

    <div class="sep"></div>

    <div class="item zoomrow" role="presentation">
        <span class="label">Zoom</span>
        <span class="zoomctl">
            <button class="zbtn" aria-label="Zoom out" onclick={onzoomout} disabled={!onzoomout}>−</button>
            <button class="zval" aria-label="Reset zoom" onclick={onzoomreset} disabled={!onzoomreset}>{zoom}%</button>
            <button class="zbtn" aria-label="Zoom in" onclick={onzoomin} disabled={!onzoomin}>+</button>
        </span>
    </div>

    <button class="item" role="menuitem" onclick={() => run(onfullscreen)}>
        {#if fullscreen}
            <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M8 3v3a2 2 0 0 1 -2 2h-3" />
                <path d="M21 8h-3a2 2 0 0 1 -2 -2v-3" />
                <path d="M16 21v-3a2 2 0 0 1 2 -2h3" />
                <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            </svg>
            <span class="label">Exit Full Screen</span>
        {:else}
            <svg class="ico" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 8v-2a2 2 0 0 1 2 -2h2" />
                <path d="M4 16v2a2 2 0 0 0 2 2h2" />
                <path d="M16 4h2a2 2 0 0 1 2 2v2" />
                <path d="M16 20h2a2 2 0 0 0 2 -2v-2" />
            </svg>
            <span class="label">Full Screen</span>
        {/if}
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
        font-family: Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif;
        -webkit-font-smoothing: antialiased;
        text-rendering: optimizeLegibility; 
        position: absolute;
        top: calc(100% + 6px);
        left: 0;
        z-index: 1000;
        width: 300px;
        max-height: min(560px, calc(100vh - 90px));
        overflow-y: auto;
        overflow-x: hidden;
        padding: 6px;
        background: var(--bg-page, #fff);
        border: 1px solid var(--border, rgba(74, 58, 46, 0.08));
        border-radius: 14px;
        box-shadow: 0 12px 32px var(--shadow, rgba(74, 58, 46, 0.16));
        transform-origin: top center;
        animation: menu-in 0.18s cubic-bezier(0.32, 0.72, 0, 1);
    }

    @keyframes menu-in {
        from {
            opacity: 0;
            transform: translateY(-8px) scale(0.96);
        }
        to {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }

    .item,
    .sep {
        animation: item-in 0.22s cubic-bezier(0.32, 0.72, 0, 1) backwards;
    }

    .menu > *:nth-child(1) { animation-delay: 0.02s; }
    .menu > *:nth-child(2) { animation-delay: 0.03s; }
    .menu > *:nth-child(3) { animation-delay: 0.04s; }
    .menu > *:nth-child(4) { animation-delay: 0.05s; }
    .menu > *:nth-child(5) { animation-delay: 0.06s; }
    .menu > *:nth-child(6) { animation-delay: 0.07s; }
    .menu > *:nth-child(7) { animation-delay: 0.08s; }
    .menu > *:nth-child(8) { animation-delay: 0.09s; }
    .menu > *:nth-child(n + 9) { animation-delay: 0.1s; }

    @keyframes item-in {
        from {
            opacity: 0;
            transform: translateY(-4px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .menu,
        .item,
        .sep {
            animation: none;
        }
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
        background: var(--border);
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
