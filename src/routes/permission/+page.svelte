<script lang="ts">
    import '../../app.css';
    import { onMount } from 'svelte';
    import { Camera, Mic, MapPin, Bell } from '@lucide/svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { emit, listen } from '@tauri-apps/api/event';

    import { getDecision, saveDecision, originOf } from '$lib/services/sitePermissions';

    interface PermissionRequest {
        tabId: string;
        requestId: string;
        kind: string;
        uri: string;
    }

    const LABELS: Record<string, string> = {
        camera: 'use your camera',
        microphone: 'use your microphone',
        geolocation: 'know your location',
        notifications: 'send you notifications'
    };

    let request = $state<PermissionRequest | null>(null);
    let busy = $state(false);

    let siteName = $derived.by(() => {
        if (!request?.uri) return 'This site';
        try {
            return new URL(request.uri).hostname.replace(/^www\./, '') || 'This site';
        } catch {
            return 'This site';
        }
    });

    async function handleRequest(req: PermissionRequest | null) {
        while (req) {
            const origin = req.uri ? originOf(req.uri) : null;
            const saved = origin ? getDecision(origin, req.kind) : null;
            if (!saved) {
                request = req;
                return;
            }
            await invoke('pending_permission', {
                requestId: req.requestId,
                granted: saved === 'allow'
            }).catch(() => {});
            req = await invoke<PermissionRequest | null>('current_permission').catch(() => null);
        }
        request = null;
    }

    async function respond(granted: boolean) {
        if (!request || busy) return;
        busy = true;
        const { requestId, kind, uri } = request;
        request = null;
        try {
            const origin = uri ? originOf(uri) : null;
            if (origin) saveDecision(origin, kind, granted ? 'allow' : 'deny');
            await invoke('pending_permission', { requestId, granted });
            const next = await invoke<PermissionRequest | null>('current_permission');
            await handleRequest(next);
        } finally {
            busy = false;
        }
    }

    onMount(() => {
        const unlistenRequest = listen<PermissionRequest>('permission-requested', (event) => {
            void handleRequest(event.payload);
        });
        const unlistenTheme = listen<Record<string, string>>('permission-theme', (e) => {
            const root = document.documentElement;
            for (const [key, value] of Object.entries(e.payload ?? {})) {
                root.style.setProperty(key, value);
            }
        });

        invoke<PermissionRequest | null>('current_permission').then((pending) => {
            if (!request && pending) void handleRequest(pending);
        });

        Promise.all([unlistenRequest, unlistenTheme]).then(() => emit('permission-ready', {}));
        return () => {
            unlistenRequest.then((off) => off());
            unlistenTheme.then((off) => off());
        };
    });
</script>

{#if request}
    <div class="overlay">
        <div class="card">
            <div class="row">
                <span class="kind-icon" aria-hidden="true">
                    {#if request.kind === 'camera'}
                        <Camera />
                    {:else if request.kind === 'microphone'}
                        <Mic />
                    {:else if request.kind === 'geolocation'}
                        <MapPin />
                    {:else if request.kind === 'notifications'}
                        <Bell />
                    {/if}
                </span>
                <p class="message"><strong>{siteName}</strong> wants to {LABELS[request.kind] ?? request.kind}. Your choice is remembered for this site.</p>
            </div>
            <div class="actions">
                <button class="deny" onclick={() => respond(false)} disabled={busy}>Block</button>
                <button class="allow" onclick={() => respond(true)} disabled={busy}>Allow</button>
            </div>
        </div>
    </div>
{/if}

<style>
    :global(*) {
        box-sizing: border-box;
    }

    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        background: transparent !important;
        font-family: var(--font-ui);
    }
    
    .overlay {
        position: fixed;
        inset: 0;
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 96px;
        background: var(--overlay, rgba(74, 58, 46, 0.28));
    }

    .card {
        width: min(420px, calc(100vw - 24px));
        padding: 16px 18px;
        background: var(--bg-page, #ffffff);
        color: var(--text, #4a3a2e);
        border: 1px solid var(--border, rgba(74, 58, 46, 0.08));
        border-radius: 12px;
        box-shadow: 0 12px 32px var(--shadow, rgba(74, 58, 46, 0.16));
        animation: drop-in 0.18s cubic-bezier(0.32, 0.72, 0, 1);
    }

    @keyframes drop-in {
        from {
            opacity: 0;
            transform: translateY(-8px) scale(0.98);
        }
        to {
            opacity: 1;
            transform: translateY(0) scale(1);
        }
    }

    .row {
        display: flex;
        align-items: flex-start;
        gap: 10px;
        margin-bottom: 14px;
    }

    .kind-icon {
        flex: 0 0 auto;
        width: 20px;
        height: 20px;
        margin-top: 1px;
        color: var(--accent, #80a4d4);
    }

    .kind-icon :global(svg) {
        width: 100%;
        height: 100%;
    }

    .message {
        margin: 0;
        font-size: 14px;
        line-height: 1.5;
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }

    button {
        font: inherit;
        font-size: 13px;
        padding: 7px 16px;
        border-radius: 8px;
        border: none;
        cursor: pointer;
    }

    button:disabled {
        opacity: 0.6;
        cursor: default;
    }

    .deny {
        background: var(--field, #f7f1ec);
        color: var(--text, #4a3a2e);
    }

    .allow {
        background: var(--accent, #80a4d4);
        color: var(--accent-contrast, #ffffff);
        font-weight: 500;
    }
</style>
