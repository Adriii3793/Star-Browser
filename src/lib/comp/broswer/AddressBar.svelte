<script lang="ts">
    let { url = '', onnavigate, onchat, onback, onforward, onreload,
        onnewtab, onsettings, onhistory, canBack = false, canForward = false
    }:
        {
            url?: string;
            onnavigate: (u: string) => void;
            onchat: () => void;
            onback: () => void;
            onforward: () => void;
            onreload: () => void;
            onnewtab?: () => void;
            onsettings?: () => void;
            onhistory?: () => void;
            canBack?: boolean;
            canForward?: boolean;
        } = $props();
    let value = $state('');

    let focused = $state(false);
    $effect(() => {
        if (!focused) value = url;
    });
</script>

<div class="navbar">
    <button class="navbtn" aria-label="Back" onclick={onback} disabled={!canBack}>
        <svg class="nav" class:dim={!canBack} xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M15 6l-6 6l6 6" />
        </svg>
    </button>
    <button class="navbtn" aria-label="Forward" onclick={onforward} disabled={!canForward}>
        <svg class="nav" class:dim={!canForward} xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M9 6l6 6l-6 6" />
        </svg>
    </button>
    <button class="navbtn" aria-label="Reload" onclick={onreload}>
        <svg class="nav" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 11a8.1 8.1 0 0 0 -15.5 -2m-.5 -5v5h5" />
            <path d="M4 13a8.1 8.1 0 0 0 15.5 2m.5 5v-5h-5" />
        </svg>
    </button>

    <div class="field">
        <i class="ti ti-lock lock"></i>
        <input
            bind:value
            onfocus={() => (focused = true)}
            onblur={() => (focused = false)}
            onkeydown={(e) => e.key === 'Enter' && onnavigate(value)}
            placeholder="Search"
        />
    </div>

    <div class="actions">
        <button class="chat" onclick={onchat}>
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 17.75l-6.172 3.245l1.179 -6.873l-5 -4.867l6.9 -1l3.086 -6.253l3.086 6.253l6.9 1l-5 4.867l1.179 6.873z" />
            </svg>
            Chat 
        </button>
    </div>
</div>

    <style>
        .navbar {
            display:flex; align-items: center; gap: 14px;
            padding: 6px 16px 12px; background: var(--bg-chrome);

        }
        .nav {
            font-size: 18px; color: var(--text); cursor:pointer;
        }
        .nav.dim {
            color: rgba(74, 58, 46, 0.28);
        }
        .navbtn {
            display:flex;
            align-items: center;
            justify-content: center;
            width: 30px;
            height: 30px;
            padding: 0;
            border: none;
            border-radius: 8px;
            background: transparent;
            cursor: pointer;
            transition: background-color 0.14s ease;
        }
        .navbtn:hover:not(:disabled) {
            background: var(--field);
        }
        .navbtn:disabled {
            cursor: default;
        }
        .field {
            flex:1; display:flex;align-items: center;gap:8px;
            background: var(--field); border-radius: 999px; padding:8px 16px;

        }
        .lock {
            font-size: 13px;color:#7fa06b;
        }
        .field input {
            flex: 1; border: none; background: transparent; outline:none;
            font-size: 13px; color: var(--text); font-family:inherit;
        }
        .field input::placeholder {color:var(--text-muted);}
        .chat {
            display: flex; align-items: center; gap: 7px;
            background: #fff; border: none; border-radius: 999px;
            padding: 8px 15px; font-size: 13px; font-weight:500;
            color: var(--text); cursor: pointer;
        }

        .actions {
            position: relative; 
            display: flex;
            align-items: center;
            gap: 4px;
            flex: 0 0 auto;
        }
        
    </style>
