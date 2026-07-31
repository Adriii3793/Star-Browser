<script lang="ts">
    import { setup } from '$lib/stores/setup.svelte';

    let { url = '', onnavigate, onchat, onback, onforward, onreload,
        canBack = false, canForward = false, mediaActive = false, onmedia
    }:
        {
            url?: string;
            onnavigate: (u: string) => void;
            onchat: () => void;
            onback: () => void;
            onforward: () => void;
            onreload: () => void;
            canBack?: boolean;
            canForward?: boolean;
            mediaActive?: boolean;
            onmedia?: () => void;
        } = $props();
    let value = $state('');
    let focused = $state(false);
    let lastSyncedUrl = $state('');

    $effect(() => {
        if (!focused && url !== lastSyncedUrl) {
            value = url;
            lastSyncedUrl = url;
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
            onblur={() => (focused = false)}
            onkeydown={(e) => e.key === 'Enter' && onnavigate(value)}
            placeholder="Search with {setup.engine.name}"
        />
        <button class="copy" type="button" aria-label="Copy link" title="Copy link" onclick={copyLink} disabled={!url}>
            <span aria-hidden="true">🔗</span>
        </button>
    </div>

    <div class="actions">
        {#if mediaActive}
        <button class="media" type="button" aria-label="Media playing in tabs" title="Media controls" onclick={onmedia}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M9 18V5l12-2v13" />
                <circle cx="6" cy="18" r="3" />
                <circle cx="18" cy="16" r="3" />
            </svg>
        </button>
        {/if}
        <button class="chat" type="button" aria-label="Open AI Chat" onclick={onchat}>
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
            background: var(--bg-chrome); border: 1px solid transparent; border-radius: 999px; padding:0 6px 0 14px;
            transition: border-color 150ms ease, box-shadow 150ms ease;
        }
        .field:focus-within {
            box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 18%,transparent);
        }
        .field input {
            flex: 1; border: none; background: transparent; outline:none;
            font-size: 13px; color: var(--text); font-family:inherit;
        }
        .field input::placeholder {color:var(--text-muted);}
        .field:focus-within {
            border-color: var(--border-strong);
            background: var(--bg-chrome);
        }
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
            background: var(--bg-page);
            color: var(--text-soft);
            font-size: 12px;
            cursor: pointer;
            transition: background-color .14s ease, color .14s ease, transform .14s ease;
        }
        .copy:hover:not(:disabled) { background: var(--tab-hover); color: var(--text); transform: translateY(-1px); }
        .copy:disabled { opacity: .45; cursor: default; }
        .media {
            display: inline-flex; align-items: center; justify-content: center;
            width: 32px; height: 32px; padding: 0;
            border: 1px solid var(--border); border-radius: 999px;
            background: var(--bg-page); color: var(--accent);
            cursor: pointer;
            transition: background-color .14s ease, border-color .14s ease;
        }
        .media:hover { background: var(--tab-hover); border-color: var(--border-strong); }

        .chat {
            display: flex; align-items: center; gap: 7px;
            height: 32px;
            background: var(--bg-page); border: 1px solid var(--border); border-radius: 999px;
            padding: 0 13px; font-size: 12px; font-weight:600;
            color: var(--text); cursor: pointer;
            transition: background-color .14s ease, border-color .14s ease;
        }
        .chat:hover { background: var(--tab-hover); border-color: var(--border-strong); }

        .actions {
            position: relative;
            display: flex;
            align-items: center;
            gap: 8px;
            flex: 0 0 auto;
        }

        @media (prefers-reduced-motion: reduce) {
            .copy, .chat { transition: none; }
        }
        
    </style>
