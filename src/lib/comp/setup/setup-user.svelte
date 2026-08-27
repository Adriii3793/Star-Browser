<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import StepShell from './StepShell.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import { cropAvatar } from '$lib/services/avatar';

    let { onnext }: { onnext: () => void } = $props();
    let fileEl = $state<HTMLInputElement>();

    let initial = $derived((setup.data.name.trim()[0] ?? '?').toUpperCase());

    async function pickAvatar(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;
        const cropped = await cropAvatar(file);
        if (cropped) setup.data.avatar = cropped;
    }
</script>

<StepShell title="Create your profile" subtitle="Only stored on this device" width={340}>
    <div class="avatar-wrap">
        <div class="avatar">
            {#if setup.data.avatar}
                <img src={setup.data.avatar} alt="Your profile" />
            {:else if setup.data.name.trim()}
                <span>{initial}</span>
            {:else}
                <img src="/avatars/default.svg" alt="" />
            {/if}
        </div>
        <button
            class="upload"
            type="button"
            aria-label="Change photo"
            title="Change photo"
            onclick={() => fileEl?.click()}
        >
            <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M4 20h4L18.5 9.5a2.828 2.828 0 1 0-4-4L4 16v4" />
            </svg>
        </button>
    </div>

    <label class="field">
        <span>Your name</span>
        <input type="text" bind:value={setup.data.name} placeholder="Name" maxlength="40" />
    </label>

    <input type="file" accept="image/*" bind:this={fileEl} onchange={pickAvatar} hidden />

    {#snippet footer()}
        <Button3D label="Continue" disabled={!setup.data.name.trim()} onclick={onnext} />
    {/snippet}
</StepShell>

<style>
    .avatar-wrap {
        position: relative;
    }

    .avatar {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 96px;
        height: 96px;
        border-radius: 50%;
        overflow: hidden;
        background: var(--field);
        color: var(--accent);
        font-size: 38px;
        font-weight: 600;
    }

    .avatar img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .upload {
        position: absolute;
        right: -2px;
        bottom: -2px;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        padding: 0;
        border: 3px solid var(--bg-page, #fff);
        border-radius: 50%;
        background: var(--accent);
        color: var(--accent-contrast, #1c1917);
        cursor: pointer;
        transition: transform 0.15s ease;
    }

    .upload:hover {
        transform: scale(1.08);
    }

    .upload:focus-visible {
        outline: 2px solid var(--accent);
        outline-offset: 3px;
    }

    .upload svg {
        width: 14px;
        height: 14px;
        fill: none;
        stroke: currentColor;
        stroke-width: 2.5px;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        width: 100%;
    }

    .field span {
        font-size: 11px;
        letter-spacing: 0.06em;
        color: var(--text-muted);
    }

    .field input {
        padding: 11px 14px;
        border: 1px solid var(--border-strong);
        border-radius: 10px;
        background: var(--bg-page);
        color: var(--text);
        font: inherit;
        font-size: 14px;
        outline: none;
        transition: border-color 0.15s ease;
    }

    .field input:focus {
        border-color: var(--accent);
    }

    @media (prefers-reduced-motion: reduce) {
        .upload,
        .field input {
            transition: none;
        }
    }
</style>
