<script lang="ts">
    import Button3D from '../ui/Button3D.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import {
        PRESET_THEMES,
        SYSTEM_THEME,
        readableText,
        imageLuminance,
        luminance,
        mix,
        theme as themeStore,
        type Theme
    } from '$lib/stores/theme.svelte';

    let { onnext }: { onnext: () => void } = $props();

    let bgFileEl = $state<HTMLInputElement>();
    let textColor = $state('#1c1917');

    let themes = $derived.by<Theme[]>(() => {
        const list = [SYSTEM_THEME, ...PRESET_THEMES];
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

    let active = $derived(
        setup.data.theme === 'system'
            ? themeStore.current
            : themes.find((t) => t.id === setup.data.theme) ?? themes[1]
    );
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
        themeStore.set(t.id === 'system' ? 'system' : t);
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
            setup.data.customAccent ??= '#80A4D4';
            themeStore.set({
                id: 'custom',
                name: 'Custom',
                bg: setup.data.customBg,
                surface: setup.data.customSurface,
                accent: setup.data.customAccent,
                image: setup.data.background
            });
        };
        r.readAsDataURL(file);
    }

    function applyCustomColors(bg: string, accent: string) {
        const surface = mix(bg, '#ffffff', luminance(bg) < 0.5 ? 0.07 : 0.6);
        setup.data.customBg = bg;
        setup.data.customSurface = surface;
        setup.data.customAccent = accent;
        setup.data.theme = 'custom';
        themeStore.set({
            id: 'custom',
            name: 'Custom',
            bg,
            surface,
            accent,
            image: setup.data.background
        });
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
                style="background:{t.id === 'system' ? 'linear-gradient(135deg, #f8fafc 0 50%, #292524 50% 100%)' : t.bg}; --tick:{t.id === 'system' ? '#1c1917' : readableText(t.bg)}"
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

    <div class="customrow">
        <label class="colorpick">
            <input
                type="color"
                value={setup.data.customBg ?? '#faf7f7'}
                oninput={(e) => applyCustomColors(e.currentTarget.value, setup.data.customAccent ?? '#80A4D4')}
            />
            <span>Custom color</span>
        </label>
        <label class="colorpick">
            <input
                type="color"
                value={setup.data.customAccent ?? '#80A4D4'}
                oninput={(e) => applyCustomColors(setup.data.customBg ?? '#faf7f7', e.currentTarget.value)}
            />
            <span>Accent</span>
        </label>
    </div>

    <Button3D label="Continue" onclick={onnext} />

    <input type="file" accept="image/*" bind:this={bgFileEl} onchange={pickBackground} style="display:none" />
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
        box-shadow: 0 8px 28px var(--shadow);
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
        color: var(--tick, var(--accent));
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

    .customrow {
        display: flex;
        gap: 14px;
    }

    .colorpick {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 7px 12px 7px 8px;
        border: 1px solid var(--border);
        border-radius: 999px;
        background: var(--field);
        font-size: 12px;
        font-weight: 500;
        color: var(--text-soft);
        cursor: pointer;
        transition: border-color 0.15s ease, background-color 0.15s ease;
    }

    .colorpick:hover {
        border-color: var(--border-strong);
        background: var(--tab-hover, var(--field));
    }

    .colorpick input {
        width: 26px;
        height: 26px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: none;
        cursor: pointer;
    }
    .colorpick input::-webkit-color-swatch-wrapper { padding: 0; }
    .colorpick input::-webkit-color-swatch { border: 1px solid rgba(0, 0, 0, 0.15); border-radius: 50%; }

    @media (prefers-reduced-motion: reduce) {
        .preview {
            transition: none;
        }
    }
</style>
