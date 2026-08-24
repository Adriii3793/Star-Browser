<script lang="ts">
    import { emit, listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';

    interface MenuState {
        x: number;
        y: number;
        tabId: string | null;
        grouped: boolean;
        muted?: boolean;
        groups?: { id: string; name: string; color: string }[];
        tabGroupId?: string | null;
    }

    let {
        onclose,
        onmute,
        onduplicate,
        oncloseothers,
        oncreategroup,
        onaddtogroup,
        onremovefromgroup
    }: {
        onclose: () => void;
        onmute: (tabId: string) => void;
        onduplicate: (tabId: string) => void;
        oncloseothers: (tabId: string) => void;
        oncreategroup: (tabId: string, x: number, y: number) => void;
        onaddtogroup: (tabId: string, groupId: string) => void;
        onremovefromgroup: (tabId: string) => void;
    } = $props();

    let menu = $state<MenuState | null>(null);

    onMount(() => {
        emit('overlay-request-state', { kind: 'tabmenu' });
        const unlisten = listen<MenuState>('overlay-tabmenu-state', (e) => {
            menu = e.payload ?? null;
            if (!menu) onclose();
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    function run(action: (m: MenuState) => void) {
        const snapshot = menu ? { ...menu } : null;
        menu = null;
        if (snapshot) action(snapshot);
        onclose();
    }
</script>

{#if menu}
    <div class="scrim" role="presentation" onclick={onclose}></div>
    <div class="menu" role="menu" style:left="{menu.x}px" style:top="{menu.y}px">
        {#if menu.tabId}
            <button class="item" type="button" role="menuitem"
                onclick={() => run((m) => m.tabId && onmute(m.tabId))}>
                {menu.muted ? 'Unmute tab' : 'Mute tab'}
            </button>
            <button class="item" type="button" role="menuitem"
                onclick={() => run((m) => m.tabId && onduplicate(m.tabId))}>
                Duplicate tab
            </button>
            <button class="item" type="button" role="menuitem"
                onclick={() => run((m) => m.tabId && oncloseothers(m.tabId))}>
                Close other tabs
            </button>
            <div class="sep" role="separator"></div>
        {/if}
        {#if !menu.grouped}
            <button class="item" type="button" role="menuitem"
                onclick={() => run((m) => m.tabId && oncreategroup(m.tabId, m.x, m.y))}>
                Create a Group
            </button>
            {#each menu.groups ?? [] as group (group.id)}
                <button class="item" type="button" role="menuitem"
                    onclick={() => run((m) => m.tabId && onaddtogroup(m.tabId, group.id))}>
                    <span class="dot" style:background={group.color}></span>
                    Add to "{group.name}"
                </button>
            {/each}
        {:else}
            {#each (menu.groups ?? []).filter((g) => g.id !== menu?.tabGroupId) as group (group.id)}
                <button class="item" type="button" role="menuitem"
                    onclick={() => run((m) => m.tabId && onaddtogroup(m.tabId, group.id))}>
                    <span class="dot" style:background={group.color}></span>
                    Move to "{group.name}"
                </button>
            {/each}
            <button class="item" type="button" role="menuitem"
                onclick={() => run((m) => m.tabId && onremovefromgroup(m.tabId))}>
                <span class="dot muted"></span>
                Remove from Group
            </button>
        {/if}
    </div>
{/if}

<style>
    .scrim {
        position: fixed;
        inset: 0;
        z-index: 5;
        background: transparent;
    }

    .menu {
        position: fixed;
        z-index: 10;
        display: flex;
        flex-direction: column;
        min-width: 190px;
        padding: 6px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--bg-page);
        color: var(--text);
        box-shadow: 0 8px 24px var(--shadow);
        animation: menu-pop 0.15s cubic-bezier(0.32, 0.72, 0, 1);
        transform-origin: top left;
    }

    @keyframes menu-pop {
        from { opacity: 0; transform: scale(0.96) translateY(-4px); }
    }

    .item {
        display: flex;
        align-items: center;
        gap: 8px;
        height: 32px;
        padding: 0 10px;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        text-align: left;
        white-space: nowrap;
        cursor: pointer;
    }
    .item:hover { background: var(--tab-hover); }
    .sep { height: 1px; margin: 5px 4px; background: var(--border); }
    .dot { flex: 0 0 auto; width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
    .dot.muted { background: var(--text-muted); }
</style>
