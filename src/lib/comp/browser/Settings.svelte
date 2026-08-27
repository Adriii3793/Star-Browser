<script lang="ts">
    import { prefs } from '$lib/stores/prefs.svelte';
    import { emit } from '@tauri-apps/api/event';
    import {
        PRESET_THEMES,
        SYSTEM_THEME,
        theme,
        applyThemeVars,
        luminance,
        mix
    } from '$lib/stores/theme.svelte';
    import type { Theme } from '$lib/stores/theme.svelte';
    import { SEARCH_ENGINES } from '$lib/stores/setup.svelte';
    import { AI_PROVIDERS } from '$lib/stores/prefs.svelte';
    import CloseButton from '../ui/CloseButton.svelte';
    let {
        onclose,
        themeId = theme.preference,
        searchEngine = 'google',
        background = null,
        customBg = null,
        customSurface = null,
        customAccent = null
    }: {
        onclose: () => void;
        themeId?: string;
        searchEngine?: string;
        background?: string | null;
        customBg?: string | null;
        customSurface?: string | null;
        customAccent?: string | null;
    } = $props();

    const DEFAULT_CUSTOM_BG = '#faf7f7';
    const DEFAULT_CUSTOM_ACCENT = '#80a4d4';
    const HEX = /^#[0-9a-fA-F]{6}$/;

    function surfaceFor(bg: string): string {
        return mix(bg, '#ffffff', luminance(bg) < 0.5 ? 0.07 : 0.6);
    }

    let customOverride = $state<{ bg: string; surface: string; accent: string } | null>(null);
    let custom = $derived(
        customOverride ?? {
            bg: customBg ?? DEFAULT_CUSTOM_BG,
            surface: customSurface ?? surfaceFor(customBg ?? DEFAULT_CUSTOM_BG),
            accent: customAccent ?? DEFAULT_CUSTOM_ACCENT
        }
    );

    let customTheme = $derived<Theme>({
        id: 'custom',
        name: 'Custom',
        bg: custom.bg,
        surface: custom.surface,
        accent: custom.accent,
        image: background
    });
    let themes = $derived<Theme[]>([SYSTEM_THEME, ...PRESET_THEMES, customTheme]);

    let themeOverride = $state<string | null>(null);
    let engineOverride = $state<string | null>(null);
    let activeThemeId = $derived(themeOverride ?? themeId);
    let selectedSearchEngine = $derived(engineOverride ?? searchEngine);

    function selectTheme(t: Theme) {
        if (t.id === 'custom') {
            applyCustom(custom.bg, custom.accent);
            return;
        }
        themeOverride = t.id;
        theme.set(t.id);
        void emit('settings-changed', { theme: t.id });
    }

    function applyCustom(bg: string, accent: string) {
        const surface = surfaceFor(bg);
        customOverride = { bg, surface, accent };
        themeOverride = 'custom';
        applyThemeVars({ id: 'custom', name: 'Custom', bg, surface, accent, image: background });
        void emit('settings-changed', {
            theme: 'custom',
            customBg: bg,
            customSurface: surface,
            customAccent: accent
        });
    }

    function applyHex(value: string, which: 'bg' | 'accent') {
        const next = value.startsWith('#') ? value : `#${value}`;
        if (!HEX.test(next)) return;
        if (which === 'bg') applyCustom(next, custom.accent);
        else applyCustom(custom.bg, next);
    }

    function selectSearchEngine(id: string) {
        engineOverride = id;
        void emit('settings-changed', { searchEngine: id });
    }

    const MAX_BG_EDGE = 1920;
    let backgroundOverride = $state<string | null | undefined>(undefined);
    let wallpaper = $derived(backgroundOverride === undefined ? background : backgroundOverride);
    let bgFileEl = $state<HTMLInputElement>();

    function downscale(source: string): Promise<string> {
        return new Promise((resolve) => {
            const img = new Image();
            img.onload = () => {
                const scale = Math.min(1, MAX_BG_EDGE / Math.max(img.width, img.height));
                const canvas = document.createElement('canvas');
                canvas.width = Math.round(img.width * scale);
                canvas.height = Math.round(img.height * scale);
                const ctx = canvas.getContext('2d');
                if (!ctx) return resolve(source);
                ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
                resolve(canvas.toDataURL('image/jpeg', 0.82));
            };
            img.onerror = () => resolve(source);
            img.src = source;
        });
    }

    function pickBackground(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const file = input.files?.[0];
        input.value = '';
        if (!file?.type.startsWith('image/')) return;
        const reader = new FileReader();
        reader.onload = async () => {
            const data = await downscale(String(reader.result));
            backgroundOverride = data;
            void emit('settings-changed', { background: data });
        };
        reader.readAsDataURL(file);
    }

    function clearBackground() {
        backgroundOverride = null;
        void emit('settings-changed', { background: null });
    }

    function onkeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') onclose();
    }

    let adblockOn = $state(localStorage.getItem('star.adblock') !== 'off');
    function toggleAdblock(enabled: boolean) {
        adblockOn = enabled;
        try {
            localStorage.setItem('star.adblock', enabled ? 'on' : 'off');
        } catch {}
        void emit('settings-changed', { adblock: enabled });
    }

    prefs.init();

    function toggleUngrouped(enabled: boolean) {
        prefs.setSkipUngroupedTabs(enabled);
        void emit('settings-changed', { skipUngroupedTabs: enabled });
    }

    function toggleFavorites(enabled: boolean) {
        prefs.setFavorites(enabled);
        void emit('settings-changed', { showFavorites: enabled });
    }

    function toggleRecent(enabled: boolean) {
        prefs.setRecent(enabled);
        void emit('settings-changed', { showRecent: enabled });
    }
</script>

<svelte:window {onkeydown} />

<div
    class="overlay"
    role="button"
    tabindex="0"
    aria-label="Close settings"
    onclick={(e) => {
        if (e.target === e.currentTarget) onclose();
    }}
    onkeydown={(e) => {
        if ((e.key === 'Enter' || e.key === ' ') && e.target === e.currentTarget) {
            e.preventDefault();
            onclose();
        }
    }}
>
<div
    class="panel"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-label="Settings"
>
        <header>
            <h2>Settings</h2>
            <CloseButton label="Close settings" onclick={onclose} />
        </header>
        <div class="panel-scroll">
        <section>
            <h3>Home Page</h3>
            <p class="hint">Choose which sections appear on the start page.</p>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Show Favorites</span>
                    <span class="rowsub">Quick access tiles at the top of the start page.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={prefs.showFavorites}
                    onchange={(e) => toggleFavorites(e.currentTarget.checked)}
                />
            </label>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Show Recently Visited</span>
                    <span class="rowsub">Pages and searches you opened recently.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={prefs.showRecent}
                    onchange={(e) => toggleRecent(e.currentTarget.checked)}
                />
            </label>
        </section>
        

        <section>
            <h3>Theme</h3>
            <p class="hint">Pick a theme for the browser chrome. System follows your OS setting.</p>

            <div class="theme-swatches">
                {#each themes as t (t.id)}
                    <button
                        class="theme-dot"
                        class:active={activeThemeId === t.id}
                        type="button"
                        style="--dot-bg:{t.id === 'system' ? 'linear-gradient(135deg, #f8fafc 0 50%, #292524 50% 100%)' : t.bg}; --dot-accent:{t.accent}"
                        title={t.name}
                        aria-label={t.name}
                        aria-pressed={activeThemeId === t.id}
                        onclick={() => selectTheme(t)}
                    >
                        <span class="dot-fill"></span>
                        {#if activeThemeId === t.id}
                            <span class="dot-ring"></span>
                        {/if}
                    </button>
                {/each}
            </div>

            <div class="customrow">
                <div class="colorfield">
                    <span class="colorlabel">Custom colour</span>
                    <div class="colorpick">
                        <input
                            class="swatch"
                            type="color"
                            aria-label="Custom base colour"
                            value={custom.bg}
                            oninput={(e) => applyCustom(e.currentTarget.value, custom.accent)}
                        />
                        <input
                            class="hex"
                            type="text"
                            spellcheck="false"
                            autocomplete="off"
                            maxlength="7"
                            aria-label="Custom base colour hex code"
                            value={custom.bg}
                            oninput={(e) => applyHex(e.currentTarget.value, 'bg')}
                        />
                    </div>
                </div>

                <div class="colorfield">
                    <span class="colorlabel">Accent</span>
                    <div class="colorpick">
                        <input
                            class="swatch"
                            type="color"
                            aria-label="Accent colour"
                            value={custom.accent}
                            oninput={(e) => applyCustom(custom.bg, e.currentTarget.value)}
                        />
                        <input
                            class="hex"
                            type="text"
                            spellcheck="false"
                            autocomplete="off"
                            maxlength="7"
                            aria-label="Accent colour hex code"
                            value={custom.accent}
                            oninput={(e) => applyHex(e.currentTarget.value, 'accent')}
                        />
                    </div>
                </div>
            </div>
            <p class="hint subtle">The accent colour drives buttons, switches and selection marks.</p>

            <div class="wallpaper">
                <div class="wallpaper-preview" class:empty={!wallpaper} aria-hidden="true">
                    {#if wallpaper}
                        <img src={wallpaper} alt="" />
                    {:else}
                        <svg viewBox="0 0 24 24"><path d="M4 5h16v14H4z" /><path d="M4 15l4.5-4.5 3.5 3.5 3-3L20 16" /><circle cx="9" cy="9" r="1.4" /></svg>
                    {/if}
                </div>
                <div class="wallpaper-text">
                    <span class="rowtitle">Start page background</span>
                    <span class="rowsub">{wallpaper ? 'A custom image is in use. It stays when you switch themes.' : 'Use a photo behind the start page.'}</span>
                </div>
                <div class="wallpaper-actions">
                    <button type="button" class="wallpaper-btn" onclick={() => bgFileEl?.click()}>
                        {wallpaper ? 'Replace' : 'Choose'}
                    </button>
                    {#if wallpaper}
                        <button type="button" class="wallpaper-btn subtle" onclick={clearBackground}>Remove</button>
                    {/if}
                </div>
                <input class="hidden-file" type="file" accept="image/*" bind:this={bgFileEl} onchange={pickBackground} />
            </div>
        </section>

        <section>
            <h3>Search engine</h3>
            <p class="hint">Used by the address bar for text that is not a web address.</p>

            <div class="engine-list" role="radiogroup" aria-label="Search engine">
                {#each SEARCH_ENGINES as engine (engine.id)}
                    <button
                        class="engine-row"
                        class:active={selectedSearchEngine === engine.id}
                        type="button"
                        role="radio"
                        aria-checked={selectedSearchEngine === engine.id}
                        onclick={() => selectSearchEngine(engine.id)}
                    >
                        <img class="engine-logo" src={engine.logo} alt="" />
                        <span>{engine.name}</span>
                        {#if selectedSearchEngine === engine.id}
                            <svg class="engine-check" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 7" /></svg>
                        {/if}
                    </button>
                {/each}
            </div>
        </section>

        <section>
            <h3>Assistant</h3>
            <p class="hint">{AI_PROVIDERS.length} AI models are available for Star chat - pick the one to use.</p>
            <div class="assistant-list" role="radiogroup" aria-label="AI provider">
                {#each AI_PROVIDERS as provider (provider.id)}
                    <button
                        class="assistant-row"
                        class:active={prefs.aiProvider === provider.id}
                        type="button"
                        role="radio"
                        aria-checked={prefs.aiProvider === provider.id}
                        onclick={() => {
                            prefs.aiProvider = provider.id;
                            prefs.selectProvider(provider.id);
                            void emit('settings-changed', { aiProvider: provider.id });
                        }}
                    >
                        <div class="assistant-main">
                            <span class="assistant-name">{provider.name}</span>
                            <span class="assistant-meta">{provider.vendor} · {provider.modalities}</span>
                            {#if provider.disclosure}
                                <span class="assistant-note">{provider.disclosure}</span>
                            {/if}
                        </div>
                        {#if prefs.aiProvider === provider.id}
                            <svg class="assistant-check" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 7" /></svg>
                        {/if}
                    </button>
                {/each}
            </div>
        </section>

        <section>
            <h3>Tabs</h3>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Don't save ungrouped tabs</span>
                    <span class="rowsub">Only tabs inside a group are restored on next launch.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={prefs.skipUngroupedTabs}
                    onchange={(e) => toggleUngrouped(e.currentTarget.checked)}
                />
            </label>
        </section>

        <section>
            <h3>Privacy</h3>

            <label class="row">
                <span class="rowtext">
                    <span class="rowtitle">Block ads</span>
                    <span class="rowsub">Blocks known ad and tracker networks on Windows, and hides ad slots on every platform. Reload open tabs to apply.</span>
                </span>
                <input
                    type="checkbox"
                    class="switch"
                    checked={adblockOn}
                    onchange={(e) => toggleAdblock(e.currentTarget.checked)}
                />
            </label>
        </section>
        </div>
    </div>
</div>


<style>
    .overlay {
        position: fixed;
        inset: 0;
        z-index: 200;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 24px;
        background: var(--overlay);
    }

    .panel {
        width: min(520px, 100%);
        max-height: min(640px, 100%);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        padding: 22px 24px 0;
        background: var(--bg-page);
        border-radius: 14px;
        box-shadow: 0 18px 48px var(--overlay);
    }
    .panel-scroll {
        overflow-y: auto;
        padding-bottom: 26px;
    }

    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 20px;
    }

    h2 {
        margin: 0;
        font-size: 19px;
        font-weight: 600;
        color: var(--text);
    }

    h3 {
        margin: 0 0 4px;
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.02em;
        text-transform: uppercase;
        color: var(--text-soft);
    }

    .hint {
        margin: 0 0 12px;
        font-size: 12px;
        color: var(--text-muted);
    }

    .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 16px;
        padding: 12px 14px;
        margin-bottom: 8px;
        background: var(--field);
        border-radius: 10px;
        cursor: pointer;
    }


    .rowtext {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .rowtitle {
        font-size: 13px;
        font-weight: 500;
        color: var(--text);
    }

    .rowsub {
        font-size: 11.5px;
        color: var(--text-muted);
    }

    .switch {
        appearance: none;
        -webkit-appearance: none;
        position: relative;
        flex: 0 0 auto;
        width: 40px;
        height: 22px;
        margin: 0;
        border-radius: 999px;
        background: var(--border-strong);
        cursor: pointer;
        transition: background 0.16s ease;
    }

    .switch::after {
        content: '';
        position: absolute;
        top: 2px;
        left: 2px;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: var(--bg-page);
        box-shadow: 0 1px 3px var(--overlay);
        transition: transform 0.16s ease;
    }

    .switch:checked {
        background: var(--accent);
    }

    .switch:checked::after {
        transform: translateX(18px);
    }

    section + section {
        margin-top: 24px;
        padding-top: 20px;
        border-top: 1px solid var(--border);
    }

    .theme-swatches {
        display: flex;
        flex-wrap: wrap;
        gap: 14px;
        margin-top: 10px;
    }

    .theme-dot {
        position: relative;
        width: 34px;
        height: 34px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: transparent;
        cursor: pointer;
    }

    .dot-fill {
        position: absolute;
        inset: 0;
        border-radius: 50%;
        background: var(--dot-bg);
        box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
        transition: transform 0.14s ease;
    }

    .theme-dot:hover .dot-fill {
        transform: scale(1.08);
    }

    .dot-ring {
        position: absolute;
        inset: -4px;
        border-radius: 50%;
        border: 2px solid var(--dot-accent);
        pointer-events: none;
    }

    .customrow {
        display: flex;
        flex-wrap: wrap;
        gap: 10px;
        margin-top: 14px;
    }

    .colorfield {
        display: flex;
        flex-direction: column;
        gap: 6px;
        flex: 1 1 150px;
        min-width: 0;
    }

    .colorlabel {
        font-size: 11.5px;
        color: var(--text-muted);
    }

    .colorpick {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 6px 10px 6px 6px;
        border: 1px solid var(--border);
        border-radius: 999px;
        background: var(--field);
        transition: border-color 0.15s ease;
    }

    .colorpick:focus-within {
        border-color: var(--accent);
    }

    .swatch {
        flex: 0 0 auto;
        width: 26px;
        height: 26px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: none;
        cursor: pointer;
    }
    .swatch::-webkit-color-swatch-wrapper { padding: 0; }
    .swatch::-webkit-color-swatch { border: 1px solid var(--border-strong); border-radius: 50%; }

    .hex {
        min-width: 0;
        flex: 1;
        padding: 0;
        border: none;
        background: transparent;
        color: var(--text);
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        letter-spacing: 0.02em;
        text-transform: uppercase;
        outline: none;
    }

    .hint.subtle {
        margin: 8px 0 0;
        font-size: 11.5px;
    }

    .engine-list {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 8px;
    }

    .engine-row {
        display: flex;
        align-items: center;
        gap: 8px;
        min-width: 0;
        height: 40px;
        padding: 0 10px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--field);
        color: var(--text);
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        text-align: left;
    }
    .engine-row:hover { background: var(--tab-hover); }
    .engine-row.active { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
    .engine-logo {
        flex: 0 0 auto;
        width: 20px;
        height: 20px;
        border-radius: 5px;
        object-fit: contain;
        background: var(--bg-page);
    }
    .engine-check { width: 15px; height: 15px; margin-left: auto; fill: none; stroke: var(--accent); stroke-width: 2.6; stroke-linecap: round; stroke-linejoin: round; }

    .assistant-list {
        display: grid;
        gap: 8px;
        margin-top: 10px;
    }

    .assistant-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        width: 100%;
        padding: 10px 12px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--field);
        color: var(--text);
        text-align: left;
        cursor: pointer;
    }
    .assistant-row:hover { background: var(--tab-hover); }
    .assistant-row.active { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }

    .assistant-main {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .assistant-name {
        font-size: 13px;
        font-weight: 600;
        color: var(--text);
    }

    .assistant-meta {
        font-size: 11.5px;
        color: var(--text-muted);
    }

    .assistant-note {
        font-size: 11px;
        color: var(--text-muted);
        opacity: .85;
    }

    .assistant-check {
        flex: 0 0 auto;
        width: 16px;
        height: 16px;
        fill: none;
        stroke: var(--accent);
        stroke-width: 2.4;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .wallpaper {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-top: 16px;
        padding: 10px 12px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--field);
    }

    .wallpaper-preview {
        display: grid;
        place-items: center;
        flex: 0 0 auto;
        width: 56px;
        height: 38px;
        overflow: hidden;
        border: 1px solid var(--border);
        border-radius: 7px;
        background: var(--bg-page);
        color: var(--text-muted);
    }
    .wallpaper-preview img { width: 100%; height: 100%; object-fit: cover; }
    .wallpaper-preview svg {
        width: 18px;
        height: 18px;
        fill: none;
        stroke: currentColor;
        stroke-width: 1.6;
        stroke-linecap: round;
        stroke-linejoin: round;
    }

    .wallpaper-text {
        display: flex;
        flex-direction: column;
        gap: 2px;
        flex: 1;
        min-width: 0;
    }

    .wallpaper-actions {
        display: flex;
        flex: 0 0 auto;
        gap: 6px;
    }

    .wallpaper-btn {
        padding: 7px 12px;
        border: 1px solid var(--border-strong);
        border-radius: 999px;
        background: var(--bg-page);
        color: var(--text);
        font: inherit;
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
    }
    .wallpaper-btn:hover { background: var(--tab-hover); }
    .wallpaper-btn.subtle { border-color: transparent; background: transparent; color: var(--text-muted); }
    .wallpaper-btn.subtle:hover { color: var(--danger); background: transparent; }

    .hidden-file { display: none; }

    @media (max-width: 440px) {
        .engine-list { grid-template-columns: 1fr; }
    }
</style>
