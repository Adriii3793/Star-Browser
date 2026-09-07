<script lang="ts">
    import { AppWindow, History, Download, Minimize, Maximize, Printer, Settings } from '@lucide/svelte';
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
        onclose: (reason?: 'backdrop') => void;
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

<div class="backdrop" role="presentation" onclick={() => onclose('backdrop')}></div>

<div class="menu" role="menu" aria-label="Browser menu">
    <button class="item" role="menuitem" onclick={() => run(onnewtab)}>
        <AppWindow class="ico" size={16} strokeWidth={1.8} />
        <span class="label">New Tab</span>
        <span class="shortcut">Ctrl+T</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" onclick={() => run(onhistory)}>
        <History class="ico" size={16} strokeWidth={1.8} />
        <span class="label">History</span>
        <span class="shortcut">Ctrl+H</span>
    </button>

    <button class="item" role="menuitem" onclick={() => run(ondownloads ?? (() => {}))}>
        <Download class="ico" size={16} strokeWidth={1.8} />
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
            <Minimize class="ico" size={16} strokeWidth={1.8} />
            <span class="label">Exit Full Screen</span>
        {:else}
            <Maximize class="ico" size={16} strokeWidth={1.8} />
            <span class="label">Full Screen</span>
        {/if}
        <span class="shortcut">F11</span>
    </button>

    <button class="item" role="menuitem" onclick={() => run(onprint)}>
        <Printer class="ico" size={16} strokeWidth={1.8} />
        <span class="label">Print...</span>
        <span class="shortcut">Ctrl+P</span>
    </button>

    <div class="sep"></div>

    <button class="item" role="menuitem" onclick={() => run(onsettings)}>
        <Settings class="ico" size={16} strokeWidth={1.8} />
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
        font-family: var(--font-ui);
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

    :global(.ico) {
        flex: 0 0 auto;
        color: var(--text-soft);
    }

    .item:disabled :global(.ico) {
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
