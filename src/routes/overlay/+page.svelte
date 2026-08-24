<script lang="ts">
    import { onMount } from 'svelte';
    import Settings from '$lib/comp/browser/Settings.svelte';
    import History from '$lib/comp/browser/History.svelte';
    import Downloads from '$lib/comp/browser/Downloads.svelte';
    import MiniPlayer from '$lib/comp/browser/MiniPlayer.svelte';
    import ProfilePop from '$lib/comp/browser/ProfilePop.svelte';
    import TabMenu from '$lib/comp/browser/TabMenu.svelte';
    import GroupEdit from '$lib/comp/browser/GroupEdit.svelte';
    import { emit, listen } from '@tauri-apps/api/event';

    type Kind = 'settings' | 'history' | 'downloads' | 'media' | 'profile' | 'tabmenu' | 'groupedit' | null;
    let kind = $state<Kind>(null);
    let settings = $state<{ themeId: string; searchEngine: string; background: string | null }>({
        themeId: 'light',
        searchEngine: 'google',
        background: null
    });

    function close() {
        kind = null;
        emit('overlay-close', {});
    }

    function openUrl(url: string) {
        kind = null;
        emit('overlay-navigate', { url });
    }

    onMount(() => {
        const unlistenShow = listen<{ kind: Kind; themeId?: string; searchEngine?: string; background?: string | null }>('overlay-show', (e) => {
            kind = e.payload?.kind ?? null;
            if (!e.payload) return;
            settings = {
                themeId: e.payload.themeId ?? settings.themeId,
                searchEngine: e.payload.searchEngine ?? settings.searchEngine,
                background: 'background' in e.payload ? e.payload.background ?? null : settings.background
            };
        });
        const unlistenTheme = listen<Record<string, string>>('overlay-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });

        Promise.all([unlistenShow, unlistenTheme]).then(() => emit('overlay-ready', {}));
        return () => {
            unlistenShow.then((off) => off());
            unlistenTheme.then((off) => off());
        };
    });
</script>

{#if kind === 'settings'}
    <Settings onclose={close} themeId={settings.themeId} searchEngine={settings.searchEngine} background={settings.background} />
{:else if kind === 'history'}
    <History onclose={close} onopen={openUrl} />
{:else if kind === 'downloads'}
    <Downloads onclose={close} />
{:else if kind === 'media'}
    <MiniPlayer
        onclose={close}
        ongoto={(tabId) => {
            close();
            emit('overlay-goto-tab', { tabId });
        }}
        ontoggle={(tabId) => emit('overlay-media-toggle', { tabId })}
        onmute={(tabId) => emit('overlay-media-mute', { tabId })}
    />
{:else if kind === 'profile'}
    <ProfilePop
        onclose={close}
        onsettings={() => {
            close();
            emit('overlay-open-settings', {});
        }}
        onupdate={(profile) => emit('overlay-profile-updated', profile)}
    />
{:else if kind === 'tabmenu'}
    <TabMenu
        onclose={close}
        onmute={(id) => emit('overlay-tab-action', { action: 'mute', tabId: id })}
        onduplicate={(id) => emit('overlay-tab-action', { action: 'duplicate', tabId: id })}
        oncloseothers={(id) => emit('overlay-tab-action', { action: 'closeothers', tabId: id })}
        oncreategroup={(id, x, y) => emit('overlay-tab-action', { action: 'creategroup', tabId: id, x, y })}
        onaddtogroup={(id, groupId) => emit('overlay-tab-action', { action: 'addtogroup', tabId: id, groupId })}
        onremovefromgroup={(id) => emit('overlay-tab-action', { action: 'removefromgroup', tabId: id })}
    />
{:else if kind === 'groupedit'}
    <GroupEdit onclose={close} onsave={(payload) => emit('overlay-group-save', payload)} />
{/if}

<style>
    :global(*) {
        box-sizing: border-box;
    }

    :global(:root) {
        --bg-page: #ffffff;
        --bg-chrome: #faf7f7;
        --text: #4a3a2e;
        --text-soft: #8a6b57;
        --text-muted: #ac8064;
        --field: #f7f1ec;
        --tab-hover: #fbf6f2;
        --accent: #80a4d4;
        --accent-contrast: #1c1917;
        --border: rgba(74, 58, 46, 0.14);
        --border-strong: rgba(74, 58, 46, 0.24);
        --hover: rgba(0, 0, 0, 0.06);
        --success: #27875a;
        --danger: #c0392b;
        --overlay: rgba(74, 58, 46, 0.28);
        --shadow: rgba(74, 58, 46, 0.16);
    }

    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        height: 100%;
        background: transparent !important;
        font-family: Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Segoe UI', Roboto,
            sans-serif;
        color: var(--text);
    }

    :global(body) {
        -webkit-user-select: none;
        user-select: none;
        cursor: default;
    }

    :global(input),
    :global(textarea) {
        -webkit-user-select: text;
        user-select: text;
    }

    :global(.overlay),
    :global(.scrim) {
        border-radius: var(--win-radius, 0px);
    }
</style>
