<script lang="ts">
    import { emit, listen } from '@tauri-apps/api/event';
    import { onMount } from 'svelte';

    interface EditState {
        groupId: string;
        name: string;
        color: string;
        colors: string[];
        x: number;
        y: number;
    }

    let {
        onclose,
        onsave
    }: {
        onclose: () => void;
        onsave: (payload: { groupId: string; name: string; color: string }) => void;
    } = $props();

    let edit = $state<EditState | null>(null);
    let name = $state('');
    let color = $state('');
    let input = $state<HTMLInputElement>();

    onMount(() => {
        emit('overlay-request-state', { kind: 'groupedit' });
        const unlisten = listen<EditState>('overlay-groupedit-state', (e) => {
            if (!e.payload) {
                onclose();
                return;
            }
            edit = e.payload;
            name = e.payload.name;
            color = e.payload.color;
            queueMicrotask(() => {
                input?.focus();
                input?.select();
            });
        });
        return () => {
            unlisten.then((off) => off());
        };
    });

    function save() {
        if (!edit) return;
        const trimmed = name.trim();
        onsave({ groupId: edit.groupId, name: trimmed || edit.name, color });
        onclose();
    }

    function keydown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            e.preventDefault();
            save();
        } else if (e.key === 'Escape') {
            e.preventDefault();
            onclose();
        }
    }
</script>

<svelte:window onkeydown={keydown} />

{#if edit}
    <div class="scrim" role="presentation" onclick={onclose}></div>
    <div class="pop" role="dialog" aria-label="Edit tab group" style:left="{edit.x}px" style:top="{edit.y}px">
        <input
            bind:this={input}
            bind:value={name}
            type="text"
            maxlength="30"
            placeholder="Group name"
            aria-label="Group name"
        />

        <div class="swatches" role="radiogroup" aria-label="Group color">
            {#each edit.colors as option (option)}
                <button
                    type="button"
                    class="swatch"
                    class:picked={color === option}
                    style:background={option}
                    role="radio"
                    aria-checked={color === option}
                    aria-label="Group color {option}"
                    onclick={() => (color = option)}
                ></button>
            {/each}
            <label class="custom" title="Custom color">
                <input type="color" value={color} oninput={(e) => (color = e.currentTarget.value)} />
                {#if !edit.colors.includes(color)}
                    <span class="custom-dot" style:background={color}></span>
                {/if}
            </label>
        </div>

        <div class="actions">
            <button type="button" class="cancel" onclick={onclose}>Cancel</button>
            <button type="button" class="save" onclick={save}>Save</button>
        </div>
    </div>
{/if}

<style>
    .scrim {
        position: fixed;
        inset: 0;
        z-index: 5;
        background: transparent;
    }

    .pop {
        position: fixed;
        z-index: 10;
        display: flex;
        flex-direction: column;
        gap: 10px;
        width: 250px;
        padding: 10px;
        border: 1px solid var(--border);
        border-radius: 12px;
        background: var(--bg-page);
        color: var(--text);
        box-shadow: 0 14px 36px var(--shadow);
        animation: pop-in 0.15s cubic-bezier(0.32, 0.72, 0, 1);
        transform-origin: top left;
    }

    @keyframes pop-in {
        from { opacity: 0; transform: scale(0.96) translateY(-4px); }
    }

    .pop > input {
        width: 100%;
        height: 32px;
        padding: 0 10px;
        border: 1px solid var(--border-strong);
        border-radius: 8px;
        background: var(--field);
        color: var(--text);
        font: inherit;
        font-size: 13px;
        outline: none;
    }
    .pop > input:focus {
        border-color: var(--accent);
        box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 22%, transparent);
    }

    .swatches {
        display: flex;
        align-items: center;
        gap: 7px;
    }

    .swatch {
        width: 19px;
        height: 19px;
        padding: 0;
        border: 2px solid transparent;
        border-radius: 50%;
        cursor: pointer;
        transition: transform 0.12s ease, border-color 0.12s ease;
    }
    .swatch:hover { transform: scale(1.15); }
    .swatch.picked {
        border-color: var(--text);
        transform: scale(1.15);
    }

    .custom {
        position: relative;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 19px;
        height: 19px;
        border-radius: 50%;
        cursor: pointer;
    }
    .custom input {
        position: absolute;
        inset: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        cursor: pointer;
    }
    .custom::before {
        content: '';
        position: absolute;
        inset: 0;
        border-radius: 50%;
        border: 1px solid var(--border-strong);
        background: conic-gradient(#ff544d, #ffbd2e, #28c93f, #56aeb2, #7b9eea, #a78bcb, #ff544d);
    }
    .custom-dot {
        position: absolute;
        inset: 3px;
        border-radius: 50%;
        z-index: 1;
    }

    .actions {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 7px;
    }
    .actions button {
        height: 28px;
        padding: 0 13px;
        border: none;
        border-radius: 999px;
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
    }
    .cancel {
        background: var(--field);
        color: var(--text-soft);
    }
    .cancel:hover { background: var(--tab-hover); }
    .save {
        background: var(--accent);
        color: var(--accent-contrast);
    }
    .save:hover { filter: brightness(1.08); }

    @media (prefers-reduced-motion: reduce) {
        .pop { animation: none; }
        .swatch { transition: none; }
    }
</style>
