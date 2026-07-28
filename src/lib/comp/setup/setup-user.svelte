<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import {setup} from '$lib/stores/setup.svelte'

    let { onnext }: {onnext: () => void} = $props();
    let fileEl = $state<HTMLInputElement>();

    function initial() { return(setup.data.name.trim()[0] ?? '?').toUpperCase();}
    function pickAvatar(e: Event) {
        const file = (e.target as HTMLInputElement).files?.[0];
        (e.target as HTMLInputElement).value = '';
        if (!file || !file.type.startsWith('image/')) return;
        const reader = new FileReader();
        reader.onload = () => {
            const img = new Image();
            img.onload = () => {
                const c = document.createElement('canvas');
                const s = Math.min(img.width, img.height);
                c.width = c.height = 256;
                const ctx = c.getContext('2d');
                if(!ctx) return;
                ctx.drawImage(img, (img.width - s) /2, (img.height - s)/2, s, s, 0, 0, 256, 256);
                setup.data.avatar = c.toDataURL('image/jpeg', 0.85);
            };
            img.src = String(reader.result);
        };
        reader.readAsDataURL(file);
    }
</script>

<div class="wrap">
    <h1>Create your profile</h1>
    <p class="sub">Only stored on this device</p>

    <div class="avatar-wrap">
        <div class="avatar">
            {#if setup.data.avatar} 
                <img src={setup.data.avatar} alt="Your profile" />
            {:else if setup.data.name.trim()}
                <span>{initial()}</span>
            {:else}
                <img src="/avatars/default.svg" alt="" />
            {/if}
        </div>
        <button class="upload" type="button" aria-label="Upload Photo" onclick={() => fileEl?.click()}>
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 19V5M5 12l7-7 7 7" /></svg>
        </button>
    </div>
    <label class="field">
        <span>Your name</span>
        <input type="text" bind:value={setup.data.name} placeholder="Name" maxlength="40" />
    </label>
    <Button3D label="Continue" disabled={!setup.data.name.trim()} onclick={onnext} />

    <input type="file" accept="image/*" bind:this={fileEl} onchange={pickAvatar} style="display: none" />
</div>

<style>
    .wrap {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 14px;
    }
    h1 {
        margin: 0;
        font-size: 28px;
        font-weight: 600;
        color: var(--text);
    }
    .sub {
        margin: 0 0 8px;
        font-size: 13px;
        color: var(--text-muted);
    }
    .avatar-wrap { position: relative}
    .avatar {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 96px;
        height: 96px;
        border-radius: 50%;
        overflow: hidden;
        background: var(--field);
        color:var(--accent);
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
        color: #fff;
        cursor: pointer;
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
        width: min(340px, 100%);
    }
    .field span {
        font-size: 11px;
        letter-spacing: .06em;
        color: var(--text-muted);
    }
    .field input {
        padding: 11px 14px;
        border:1px solid rgba(74, 58, 46, .16);
        border-radius: 10px;
        background: var(--bg-page);
        color: var(--text);
        font:inherit;
        font-size: 14px;
        outline: none;
    }
    .field input:focus {border-color: var(--accent);}
</style>