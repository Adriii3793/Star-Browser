<script lang="ts">
    import { setup } from '$lib/stores/setup.svelte';

    let { url = '', onnavigate, onchat, chatOpen = false, onback, onforward, onreload,
        canBack = false, canForward = false, mediaActive = false, mediaOpen = false, onmedia,
        favoriteActive = false, onfavorite
    }:
        {
            url?: string;
            onnavigate: (u: string) => void;
            onchat: () => void;
            chatOpen?: boolean;
            onback: () => void;
            onforward: () => void;
            onreload: () => void;
            canBack?: boolean;
            canForward?: boolean;
            mediaActive?: boolean;
            mediaOpen?: boolean;
            onmedia?: () => void;
            favoriteActive?: boolean;
            onfavorite?: () => void;
        } = $props();
    let value = $state('');
    let focused = $state(false);
    let dirty = $state(false);
    let lastSyncedUrl = $state('');
    let copied = $state(false);
    let copiedTimer: ReturnType<typeof setTimeout> | undefined;

    $effect(() => {
        if (url !== lastSyncedUrl) {
            lastSyncedUrl = url;
            if (!focused || !dirty) {
                value = url;
                dirty = false;
            }
        }
    });

    async function copyLink() {
        if (!url) return;
        try {
            await navigator.clipboard.writeText(url);
        } catch {
            const textarea = document.createElement('textarea');
            textarea.value = url;
            textarea.style.position = 'fixed';
            textarea.style.opacity = '0';
            document.body.append(textarea);
            textarea.select();
            document.execCommand('copy');
            textarea.remove();
        }
        copied = true;
        clearTimeout(copiedTimer);
        copiedTimer = setTimeout(() => (copied = false), 2000);
    }
</script>

<svelte:window onblur={() => (focused = false)} />

	<div class="navbar">

    <div class="nav-controls">
        <button class="nav-btn" type="button" aria-label="Back" onclick={onback} disabled={!canBack}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6l-6 6l6 6" /></svg>
        </button>
        <button class="nav-btn" type="button" aria-label="Forward" onclick={onforward} disabled={!canForward}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6l-6 6" /></svg>
        </button>
        <button class="nav-btn" type="button" aria-label="Reload" onclick={onreload}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 4v5h5" /><path d="M4.5 9a8 8 0 1 1 1.5 8" /></svg>
        </button>
    </div>

    <div class="field">
        <input
            bind:value
            onfocus={() => (focused = true)}
            onblur={() => { focused = false; dirty = false; }}
            oninput={() => (dirty = true)}
            onkeydown={(e) => {
                if (e.key === 'Enter') {
                    onnavigate(value);
                    e.currentTarget.blur();
                }
            }}
            placeholder="Search with {setup.engine.name}"
        />
        <button class="favorite-toggle" class:active={favoriteActive} type="button" aria-label={favoriteActive ? 'Remove from favorites' : 'Add to favorites'} title={favoriteActive ? 'Remove from favorites' : 'Add to favorites'} onclick={onfavorite}>
            <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill={favoriteActive ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 17.3l-5.6 3.2 1.1-6.3L2 9.1l6.3-.9L12 2.5l3.7 5.7 6.3.9-5.5 4.1 1.1 6.3z" />
            </svg>
        </button>
        <button class="copy" class:copied type="button" aria-label={copied ? 'Link copied' : 'Copy link'} title={copied ? 'Copied!' : 'Copy link'} onclick={copyLink} disabled={!url}>
            {#if copied}
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12l4 4L19 7" /></svg>
            {:else}
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 17H7A5 5 0 0 1 7 7h2" />
                    <path d="M15 7h2a5 5 0 1 1 0 10h-2" />
                    <path d="M8 12h8" />
                </svg>
            {/if}
        </button>
    </div>

    <div class="actions">
        {#if mediaActive}
        <button class="media" class:active={mediaOpen} type="button" aria-label="Media playing in tabs" aria-pressed={mediaOpen} title={mediaOpen ? 'Hide media controls' : 'Media controls'} onclick={onmedia}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11 5L6 9H2v6h4l5 4V5z" />
                <path d="M15.5 8.5a5 5 0 0 1 0 7" />
                <path d="M18.5 5.5a9.5 9.5 0 0 1 0 13" />
            </svg>
        </button>
        {/if}
        <button class="chat" class:active={chatOpen} type="button" aria-label={chatOpen ? 'Close AI Chat' : 'Open AI Chat'} aria-pressed={chatOpen} onclick={onchat}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 17.75l-6.172 3.245l1.179 -6.873l-5 -4.867l6.9 -1l3.086 -6.253l3.086 6.253l6.9 1l-5 4.867l1.179 6.873z" />
            </svg>
            <span>AI Chat</span>
        </button>
    </div>
</div>

    <style>
      
        .navbar {
            display:flex; align-items: center; gap: 4px;
            min-width: 0;
            height: 42px;
            padding: 4px 10px 6px; background: var(--bg-chrome);

        }
        .nav-controls {
            display: flex; align-items: center; gap: 2px; flex: 0 0 auto;
        }
        .nav-btn {
            display: inline-flex; align-items: center; justify-content: center;
            width: 26px; height: 26px; padding: 0; border: 0; border-radius: 50%;
            background: transparent; color: var(--text-soft); cursor: pointer;
            transition: background-color .14s ease, color .14s ease;
        }
        .nav-btn:hover:not(:disabled) { background: var(--tab-hover); color: var(--text); }
        .nav-btn:disabled { opacity: .35; cursor: default; }
        .field {
            flex:1 1 auto; min-width: 120px; display:flex; align-items: center; gap:8px;
            height: 32px;
            background: var(--field); border: 1px solid var(--border); border-radius: 999px; padding:0 6px 0 14px;
            transition: background-color 150ms ease, border-color 150ms ease, box-shadow 150ms ease;
        }
        .field:hover { background: var(--field-strong); }
        .field:focus-within {
            border-color: var(--accent);
            background: var(--bg-page);
            box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 18%,transparent);
        }
        .field input {
            flex: 1; border: none; background: transparent; outline:none;
            font-size: 13px; color: var(--text); font-family:inherit;
        }
        .field input::placeholder {color:var(--text-muted);}
        .copy {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            flex: 0 0 auto;
            width: 26px;
            height: 26px;
            padding: 0;
            border: 0;
            border-radius: 50%;
            background: transparent;
            color: var(--text-soft);
            cursor: pointer;
            transition: background-color .14s ease, color .14s ease;
        }
        .copy:hover:not(:disabled) { background: var(--hover); color: var(--text); }
        .copy.copied { color: var(--success); }
        .copy svg { display: block; }
        .copy:disabled { opacity: .45; cursor: default; }
        .favorite-toggle {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 28px;
            height: 28px;
            padding: 0;
            border: 0;
            background: transparent;
            color: var(--text-soft);
            cursor: pointer;
            transition: color .14s ease, transform .14s ease;
        }
        .favorite-toggle:hover { color: var(--text); }
        .favorite-toggle.active { color: var(--accent); }
        .favorite-toggle.active:hover { color: var(--accent-hover, var(--accent)); }
        .favorite-toggle:active { transform: scale(.9); }
        .favorite-toggle svg { display:block; }
        .media {
            display: inline-flex; align-items: center; justify-content: center;
            width: 32px; height: 32px; padding: 0;
            border: 1px solid var(--border); border-radius: 999px;
            background: var(--bg-page); color: var(--accent);
            cursor: pointer;
            transition: background-color .14s ease, border-color .14s ease;
        }
        .media:hover { background: var(--tab-hover); border-color: var(--border-strong); }
        .media.active {
            background: color-mix(in srgb, var(--accent) 16%, var(--bg-page));
            border-color: var(--accent);
        }

        .chat {
            display: flex; align-items: center; gap: 7px;
            height: 32px;
            background: var(--bg-page); border: 1px solid var(--border); border-radius: 999px;
            padding: 0 13px; font-size: 12px; font-weight:600;
            color: var(--text); cursor: pointer;
            transition: background-color .14s ease, border-color .14s ease;
        }
        .chat:hover { background: var(--tab-hover); border-color: var(--border-strong); }
        .chat.active {
            background: color-mix(in srgb, var(--accent) 16%, var(--bg-page));
            border-color: var(--accent);
            color: var(--accent);
        }

        .actions {
            position: relative;
            display: flex;
            align-items: center;
            gap: 8px;
            flex: 0 0 auto;
        }

        @media (prefers-reduced-motion: reduce) {
            .copy, .chat, .favorite-toggle { transition: none; }
        }
        
    </style>
