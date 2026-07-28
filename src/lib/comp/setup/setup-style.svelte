<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import {
        PRESET_THEMES,
        readableText,
        imageLuminance,
        parseTheme,
        type Theme
    } from '$lib/stores/theme.svelte';

    let { onnext }: { onnext: () => void } = $props();

    let bgFileEl = $state<HTMLInputElement>();
    let themeFileEl = $state<HTMLInputElement>();
    let textColor = $state('#1c1917');
    let error = $state<string | null>(null);

    let themes = $derived.by<Theme[]>(() => {
        const list = [...PRESET_THEMES];
        if (setup.data.customBg && setup.data.customSurface && setup.data.customAccent) {
            list.push({
                id: 'custom',
                name: 'Custom',
                bg: setup.data.customBg,
                surface: setup.data.customSurface,
                accent: setup.data.customAccent,
                image: setup.data.background
            });
        }
        return list;
    });

    let active = $derived(themes.find((t) => t.id === setup.data.theme) ?? themes[0]);
    let engine = $derived(setup.engine);

    $effect(() => {
        const bg = setup.data.background;
        if (bg) {
            imageLuminance(bg).then((l) => (textColor = l > 0.5 ? '#1c1917' : '#ffffff'));
        } else {
            textColor = readableText(active.surface);
        }
    });

    function selectTheme(t: Theme) {
        setup.data.theme = t.id;
        setup.data.background = t.image ?? null;
    }

    function pickBackground(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file?.type.startsWith('image/')) return;
        const r = new FileReader();
        r.onload = () => {
            setup.data.background = String(r.result);
            setup.data.theme = 'custom';
            setup.data.customBg ??= '#faf7f7';
            setup.data.customSurface ??= '#ffffff';
            setup.data.customAccent ??= '#e8734a';
        };
        r.readAsDataURL(file);
    }

    function importTheme(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file) return;
        const r = new FileReader();
        r.onload = () => {
            const t = parseTheme(String(r.result));
            if (!t) {
                error = 'That theme file is not valid.';
                return;
            }
            error = null;
            setup.data.customBg = t.bg;
            setup.data.customSurface = t.surface;
            setup.data.customAccent = t.accent;
            setup.data.background = t.image ?? null;
            setup.data.theme = 'custom';
        };
        r.readAsText(file);
    }
</script>

<div class="wrap">
    <h1>Style your browser</h1>
    <p class="sub">Pick a look. Fine-tune everything later.</p>

    <div
        class="preview"
        style="background-color:{active.surface};
               background-image:{setup.data.background ? `url(${setup.data.background})` : 'none'};
               color:{textColor}"
    >
        <p class="greet">Good afternoon, {setup.data.name || 'there'}</p>
        <div class="searchbar">
            <span class="badge" style="background:{engine.color}">{engine.initial}</span>
            <span class="ph">Search {engine.name} or type a URL</span>
        </div>
    </div>

    <p class="section">THEME</p>
    <div class="swatches">
        {#each themes as t (t.id)}
            <button
                class="sw"
                class:on={setup.data.theme === t.id}
                type="button"
                style="background:{t.bg}"
                aria-label={t.name}
                onclick={() => selectTheme(t)}
            >
                {#if setup.data.theme === t.id}<span class="tick">✓</span>{/if}
            </button>
        {/each}
        <button class="sw add" type="button" aria-label="Upload background" onclick={() => bgFileEl?.click()}>
            +
        </button>
    </div>

    <button class="link" type="button" onclick={() => themeFileEl?.click()}>Import a theme file…</button>

    {#if error}
        <p class="err">{error}</p>
    {/if}

    <Button3D label="Continue" onclick={onnext} />

    <input type="file" accept="image/*" bind:this={bgFileEl} onchange={pickBackground} style="display:none" />
    <input type="file" accept=".json" bind:this={themeFileEl} onchange={importTheme} style="display:none" />
</div>

<style>
    .wrap {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
    }

    h1 {
        margin: 0;
        font-size: 26px;
        font-weight: 600;
        color: var(--text);
    }

    .sub {
        margin: 0 0 6px;
        font-size: 13px;
        color: var(--text-muted);
    }

    .preview {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 14px;
        width: min(380px, 100%);
        height: 190px;
        padding: 20px;
        border-radius: 14px;
        background-size: cover;
        background-position: center;
        box-shadow: 0 8px 28px rgba(74, 58, 46, 0.14);
        transition: background-color 0.2s ease;
    }

    .greet {
        margin: 0;
        font-size: 16px;
        font-weight: 600;
        text-shadow: 0 1px 3px rgba(0, 0, 0, 0.28);
    }

    .searchbar {
        display: flex;
        align-items: center;
        gap: 9px;
        width: 100%;
        padding: 9px 13px;
        border-radius: 999px;
        background: rgba(255, 255, 255, 0.82);
        color: #4a3a2e;
    }

    .badge {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 18px;
        height: 18px;
        border-radius: 5px;
        color: #fff;
        font-size: 10px;
        font-weight: 700;
    }

    .ph {
        font-size: 12px;
        opacity: 0.75;
    }

    .section {
        margin: 6px 0 0;
        font-size: 11px;
        letter-spacing: 0.06em;
        color: var(--text-muted);
    }

    .swatches {
        display: flex;
        gap: 10px;
    }

    .sw {
        position: relative;
        width: 46px;
        height: 46px;
        padding: 0;
        border-radius: 12px;
        border: 2px solid transparent;
        cursor: pointer;
    }

    .sw.on {
        border-color: var(--accent);
    }

    .tick {
        position: absolute;
        inset: 0;
        display: grid;
        place-items: center;
        color: var(--accent);
        font-size: 15px;
        font-weight: 700;
    }

    .sw.add {
        display: grid;
        place-items: center;
        background: var(--field);
        color: var(--text-muted);
        font-size: 20px;
    }

    .link {
        border: none;
        background: none;
        color: var(--accent);
        font: inherit;
        font-size: 12px;
        cursor: pointer;
        text-decoration: underline;
    }

    .err {
        margin: 0;
        font-size: 12px;
        color: #c0392b;
    }

    @media (prefers-reduced-motion: reduce) {
        .preview {
            transition: none;
        }
    }
</style>
