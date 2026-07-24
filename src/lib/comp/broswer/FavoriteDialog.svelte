<script lang="ts">
    let {
        title: initialTitle = '',
        url: initialUrl = '',
        onsave,
        oncancel
    }: {
        title?: string;
        url?: string;
        onsave: (title: string, url: string) => void;
        oncancel: () => void;
    } = $props();

    let title = $state(initialTitle);
    let url = $state(initialUrl);

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') oncancel();
    }

    function submit(e: SubmitEvent) {
        e.preventDefault();
        const t = title.trim();
        let u = url.trim();
        if (!t || !u) return;
        if (!/^https?:\/\//i.test(u)) u = `https://${u}`;
        onsave(t, u);
    }
</script>

<svelte:window on:keydown={onkeydown} />

<div
    class="overlay"
    role="button"
    tabindex="0"
    aria-label="Close"
    onclick={oncancel}
    onkeydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            oncancel();
        }
    }}
>
    <div
        class="panel"
        role="dialog"
        tabindex="-1"
        aria-modal="true"
        aria-label="Edit favorite"
        onpointerdown={(e) => e.stopPropagation()}
    >
    <form onsubmit={submit}>
        <h2>{initialTitle ? 'Edit Favorite' : 'Add Favorite'}</h2>

        <label class="field">
            <span>Name</span>
            <input type="text" bind:value={title} placeholder="Name" maxlength="40" required />
        </label>

        <label class="field">
            <span>URL</span>
            <input type="text" bind:value={url} placeholder="https://example.com" required />
        </label>

        <div class="actions">
            <button type="button" class="btn ghost" onclick={oncancel}>Cancel</button>
            <button type="submit" class="btn primary">Save</button>
        </div>
    </form>
    </div>
</div>

<style>
    .overlay {
        position: fixed;
        inset: 0;
        z-index: 250;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 24px;
        background: rgba(74, 58, 46, 0.28);
    }

    .panel {
        width: min(360px, 100%);
        padding: 22px 24px 24px;
        background: var(--bg-page);
        border-radius: 14px;
        box-shadow: 0 18px 48px rgba(74, 58, 46, 0.24);
    }

    .panel form {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }

    h2 {
        margin: 0;
        font-size: 17px;
        font-weight: 600;
        color: var(--text);
    }

    .field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        font-size: 12px;
        color: var(--text-soft);
    }

    .field input {
        padding: 9px 12px;
        border: 1px solid rgba(74, 58, 46, 0.14);
        border-radius: 8px;
        background: var(--field);
        color: var(--text);
        font: inherit;
        font-size: 13px;
        outline: none;
    }

    .field input:focus {
        border-color: var(--accent);
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
        margin-top: 4px;
    }

    .btn {
        padding: 8px 16px;
        border: none;
        border-radius: 999px;
        font: inherit;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
    }

    .btn.ghost {
        background: var(--field);
        color: var(--text-soft);
    }

    .btn.primary {
        background: var(--accent);
        color: #fff;
    }
</style>
