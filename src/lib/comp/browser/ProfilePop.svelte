<script lang="ts">
    import { emit, listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';

    let {
        onclose,
        onsettings,
        onupdate
    }: {
        onclose: () => void;
        onsettings: () => void;
        onupdate: (profile: { name: string; avatar: string | null }) => void;
    } = $props();

    let name = $state('');
    let avatar = $state<string | null>(null);
    let x = $state(8);
    let y = $state(48);
    let editing = $state(false);
    let draftName = $state('');
    let draftAvatar = $state<string | null>(null);
    let fileInput = $state<HTMLInputElement>();

    onMount(() => {
        emit('overlay-request-state', { kind: 'profile' });
        const unlisten = listen<{ name: string; avatar: string | null; x?: number; y?: number }>('overlay-profile-state', (e) => {
            name = e.payload?.name ?? '';
            avatar = e.payload?.avatar ?? null;
            if (!editing) {
                draftName = name;
                draftAvatar = avatar;
            }
            if (typeof e.payload?.x === 'number') x = e.payload.x;
            if (typeof e.payload?.y === 'number') y = e.payload.y;
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    function initialOf(text: string): string {
        return (text.trim()[0] ?? '?').toUpperCase();
    }

    function beginEdit() {
        draftName = name;
        draftAvatar = avatar;
        editing = true;
    }

    function pickAvatar(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file || !file.type.startsWith('image/')) return;
        const reader = new FileReader();
        reader.onload = () => {
            draftAvatar = typeof reader.result === 'string' ? reader.result : null;
        };
        reader.readAsDataURL(file);
    }

    function save() {
        const nextName = draftName.trim();
        if (!nextName) return;
        onupdate({ name: nextName.slice(0, 40), avatar: draftAvatar });
        onclose();
    }
</script>

<div class="scrim" role="presentation" onclick={onclose}></div>

<div class="panel" role={editing ? 'dialog' : 'menu'} aria-label={editing ? 'Edit profile' : 'Profile'} style:left="{x}px" style:top="{y}px">
    {#if editing}
        <div class="editor-head">
            <span>Edit profile</span>
            <button class="close" type="button" aria-label="Cancel" onclick={() => (editing = false)}>
                <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
            </button>
        </div>
        <div class="editor-avatar">
            <span class="avatar large">
                {#if draftAvatar}
                    <img src={draftAvatar} alt="" />
                {:else}
                    {initialOf(draftName || 'Profile')}
                {/if}
            </span>
            <button class="photo" type="button" onclick={() => fileInput?.click()}>Choose photo</button>
        </div>
        <label class="field">
            <span>Name</span>
            <input bind:value={draftName} maxlength="40" placeholder="Your name" onkeydown={(event) => event.key === 'Enter' && save()} />
        </label>
        <div class="editor-actions">
            <button class="secondary" type="button" onclick={() => (editing = false)}>Cancel</button>
            <button class="primary" type="button" disabled={!draftName.trim()} onclick={save}>Save</button>
        </div>
        <input bind:this={fileInput} class="file" type="file" accept="image/*" onchange={pickAvatar} />
    {:else}
        <div class="head">
            <span class="avatar">
                {#if avatar}
                    <img src={avatar} alt="" />
                {:else}
                    {initialOf(name || 'Profile')}
                {/if}
            </span>
            <span class="meta">
                <span class="name">{name.trim() || 'Your profile'}</span>
                <span class="sub">Stored on this device</span>
            </span>
        </div>
        <button class="item" type="button" role="menuitem" onclick={beginEdit}>
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 20h4l10.5-10.5a2.828 2.828 0 1 0-4-4L4 16v4" /></svg>
            <span>Edit profile</span>
        </button>
        <button class="item" type="button" role="menuitem" onclick={onsettings}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37c1 .608 2.296.07 2.572-1.065" />
                <path d="M9 12a3 3 0 1 0 6 0a3 3 0 0 0-6 0" />
            </svg>
            <span>Settings</span>
        </button>
    {/if}
</div>

<style>
    .scrim {
        position: fixed;
        inset: 0;
        background: transparent;
    }

    .panel {
        position: fixed;
        z-index: 10;
        min-width: 230px;
        padding: 6px;
        border: 1px solid var(--border);
        border-radius: 12px;
        background: var(--bg-page);
        color: var(--text);
        box-shadow: 0 14px 36px var(--shadow);
        animation: panel-in 0.16s cubic-bezier(0.32, 0.72, 0, 1);
        transform-origin: top left;
    }

    .editor-head { display: flex; align-items: center; justify-content: space-between; padding: 5px 5px 10px; font-size: 13px; font-weight: 700; }
    .close { display: grid; place-items: center; width: 24px; height: 24px; padding: 0; border: 0; border-radius: 7px; background: transparent; color: var(--text-muted); cursor: pointer; }
    .close:hover { background: var(--hover); color: var(--text); }
    .close svg { width: 13px; height: 13px; fill: none; stroke: currentColor; stroke-width: 2.2; stroke-linecap: round; }
    .editor-avatar { display: flex; align-items: center; gap: 10px; padding: 2px 6px 12px; }
    .avatar.large { width: 42px; height: 42px; font-size: 17px; }
    .photo, .secondary, .primary { border-radius: 8px; font: inherit; font-size: 12px; font-weight: 600; cursor: pointer; }
    .photo { padding: 7px 9px; border: 1px solid var(--border); background: var(--field); color: var(--text); }
    .photo:hover, .secondary:hover { background: var(--hover); }
    .field { display: flex; flex-direction: column; gap: 5px; padding: 0 6px; font-size: 11px; font-weight: 600; color: var(--text-soft); }
    .field input { width: 100%; height: 33px; padding: 0 9px; border: 1px solid var(--border-strong); border-radius: 8px; outline: none; background: var(--field); color: var(--text); font: inherit; font-size: 13px; }
    .field input:focus { border-color: var(--accent); box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 22%, transparent); }
    .editor-actions { display: flex; justify-content: flex-end; gap: 7px; padding: 14px 6px 4px; }
    .secondary { padding: 7px 11px; border: 1px solid var(--border); background: transparent; color: var(--text); }
    .primary { padding: 7px 12px; border: 1px solid var(--accent); background: var(--accent); color: var(--accent-contrast); }
    .primary:disabled { opacity: .5; cursor: default; }
    .file { display: none; }

    @keyframes panel-in {
        from { opacity: 0; transform: translateY(-5px) scale(0.97); }
    }

    .head {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 9px 10px;
        border-bottom: 1px solid var(--border);
        margin-bottom: 5px;
    }

    .avatar {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 34px;
        height: 34px;
        overflow: hidden;
        border-radius: 50%;
        background: var(--accent);
        color: var(--accent-contrast);
        font-size: 15px;
        font-weight: 800;
    }
    .avatar img { width: 100%; height: 100%; object-fit: cover; }

    .meta { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
    .name {
        font-size: 13px;
        font-weight: 600;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }
    .sub { font-size: 11px; color: var(--text-muted); }

    .item {
        display: flex;
        align-items: center;
        gap: 9px;
        width: 100%;
        height: 32px;
        padding: 0 9px;
        border: 0;
        border-radius: 8px;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12.5px;
        font-weight: 500;
        text-align: left;
        cursor: pointer;
        transition: background-color 0.13s ease;
    }
    .item:hover { background: var(--hover); }
    .item svg {
        width: 15px;
        height: 15px;
        fill: none;
        stroke: var(--text-soft);
        stroke-width: 1.8;
        stroke-linecap: round;
        stroke-linejoin: round;
    }
</style>
