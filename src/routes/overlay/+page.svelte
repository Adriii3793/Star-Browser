<script lang="ts">
    import '../../app.css';
    import { onMount } from 'svelte';
    import Settings from '$lib/comp/browser/Settings.svelte';
    import History from '$lib/comp/browser/History.svelte';
    import Downloads from '$lib/comp/browser/Downloads.svelte';
    import MiniPlayer from '$lib/comp/browser/MiniPlayer.svelte';
    import ProfilePop from '$lib/comp/browser/ProfilePop.svelte';
    import TabMenu from '$lib/comp/browser/TabMenu.svelte';
    import GroupEdit from '$lib/comp/browser/GroupEdit.svelte';
    import { emit, listen } from '@tauri-apps/api/event';
    import { clippedSurface, watchSurfaceClip } from '$lib/services/surface';
    import type { AiKeyStatus } from '$lib/services/ai';
    import { shouldDismissOnBlur, shouldDismissOnPointer } from '$lib/services/popup';

    type Kind = 'settings' | 'history' | 'downloads' | 'media' | 'profile' | 'tabmenu' | 'groupedit' | null;
    let kind = $state<Kind>(null);
    let rootEl = $state<HTMLElement>();
    let shownAt = 0;
    interface SettingsState {
        aiKey: AiKeyStatus | null;
        themeId: string;
        searchEngine: string;
        background: string | null;
        customBg: string | null;
        customSurface: string | null;
        customAccent: string | null;
    }
    let settings = $state<SettingsState>({
        aiKey: null,
        themeId: 'light',
        searchEngine: 'google',
        background: null,
        customBg: null,
        customSurface: null,
        customAccent: null
    });

    let showSeq = 0;

    function close(reason?: 'blur' | 'backdrop') {
        if (
            reason === 'backdrop' &&
            !shouldDismissOnPointer({ shown: kind !== null, shownAt, now: Date.now() })
        ) {
            return;
        }
        kind = null;
        emit('overlay-close', { seq: showSeq, reason });
    }


    function openUrl(url: string) {
        kind = null;
        emit('overlay-navigate', { url });
    }

    const clip = watchSurfaceClip('overlay', () => rootEl);
    $effect(() => {
        void kind;
        void settings;
        clip.sync();
    });

    onMount(() => {
        let unlistenBlur: (() => void) | undefined;
        if (clippedSurface()) {
            const onBlur = () => {
                if (
                    !shouldDismissOnBlur({
                        shown: kind !== null,
                        visible: document.visibilityState === 'visible',
                        shownAt,
                        now: Date.now(),
                        activeElement: document.activeElement
                    })
                )
                    return;
                close('blur');
            };
            window.addEventListener('blur', onBlur);
            unlistenBlur = () => window.removeEventListener('blur', onBlur);
        }
        const unlistenShow = listen<{
            kind: Kind;
            seq?: number;
            themeId?: string;
            searchEngine?: string;
            background?: string | null;
            customBg?: string | null;
            customSurface?: string | null;
            customAccent?: string | null;
            aiKey?: AiKeyStatus | null;
        }>('overlay-show', (e) => {
            kind = e.payload?.kind ?? null;
            showSeq = e.payload?.seq ?? showSeq;
            shownAt = Date.now();
            clip.invalidate();
            if (!e.payload) return;
            settings = {
                aiKey: 'aiKey' in e.payload ? e.payload.aiKey ?? null : settings.aiKey,
                themeId: e.payload.themeId ?? settings.themeId,
                searchEngine: e.payload.searchEngine ?? settings.searchEngine,
                background: 'background' in e.payload ? e.payload.background ?? null : settings.background,
                customBg: 'customBg' in e.payload ? e.payload.customBg ?? null : settings.customBg,
                customSurface:
                    'customSurface' in e.payload ? e.payload.customSurface ?? null : settings.customSurface,
                customAccent:
                    'customAccent' in e.payload ? e.payload.customAccent ?? null : settings.customAccent
            };
        });
        const unlistenAiKey = listen<AiKeyStatus | null>('overlay-ai-key', (e) => {
            settings = { ...settings, aiKey: e.payload ?? null };
        });
        const unlistenTheme = listen<Record<string, string>>('overlay-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });

        Promise.all([unlistenShow, unlistenTheme]).then(() => emit('overlay-ready', {}));
        return () => {
            unlistenBlur?.();
            clip.destroy();
            unlistenShow.then((off) => off());
            unlistenAiKey.then((off) => off());
            unlistenTheme.then((off) => off());
        };
    });
</script>

<div class="overlay" bind:this={rootEl}>
{#if kind === 'settings'}
    <Settings
        onclose={close}
        themeId={settings.themeId}
        searchEngine={settings.searchEngine}
        background={settings.background}
        customBg={settings.customBg}
        customSurface={settings.customSurface}
        customAccent={settings.customAccent}
        aiKey={settings.aiKey}
    />
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
</div>

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
        --field-strong: #efe6de;
        --tab-active: #ffffff;
        --accent-hover: #6b8fc4;
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
        font-family: var(--font-ui);
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
        cursor: auto;
    }

    :global(.overlay),
    :global(.scrim) {
        border-radius: var(--win-radius, 0px);
    }

    .overlay {
        position: fixed;
        inset: 0;
        background: transparent;
    }
    
    :global([data-star-clipped-card]) {
        outline: 1px solid var(--border-strong, rgba(74, 58, 46, 0.24));
        outline-offset: -1px;
    }
</style>
