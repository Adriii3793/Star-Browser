<script lang="ts">
    import { prefs, AI_PROVIDERS, type AiProvider, type AiProviderId } from '$lib/stores/prefs.svelte';
    import { emit } from '@tauri-apps/api/event';
    import { PRESET_THEMES, SYSTEM_THEME, theme } from '$lib/stores/theme.svelte';
    import type { Theme } from '$lib/stores/theme.svelte';
    import { SEARCH_ENGINES } from '$lib/stores/setup.svelte';
    let {
        onclose,
        themeId = theme.preference,
        searchEngine = 'google'
    }: {
        onclose: () => void;
        themeId?: string;
        searchEngine?: string;
    } = $props();

    const themes = [SYSTEM_THEME, ...PRESET_THEMES];
    let activeThemeId = $state(theme.preference);
    let selectedSearchEngine = $state('Google');

    $effect(() => {
        activeThemeId = themeId;
        selectedSearchEngine = searchEngine;
    });

    function selectTheme(t: Theme) {
        activeThemeId = t.id;
        theme.set(t.id);
        void emit('settings-changed', { theme: t.id });
    }

    function selectSearchEngine(id: string) {
        selectedSearchEngine = id;
        void emit('settings-changed', { searchEngine: id });
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

    let disclosureFor = $state<AiProvider | null>(null);

    function selectProvider(p: AiProvider) {
        if (prefs.aiProvider === p.id) return;
        const firstTime = prefs.selectProvider(p.id);
        if (firstTime) disclosureFor = p;
        void emit('settings-changed', { aiProvider: p.id });
    }

    function showDisclosure(p: AiProvider, e: MouseEvent) {
        e.stopPropagation();
        disclosureFor = p;
    }

    function toggleUngrouped(enabled: boolean) {
        prefs.setSkipUngroupedTabs(enabled);
        void emit('settings-changed', { skipUngroupedTabs: enabled });
    }
</script>

<svelte:window on:keydown={onkeydown} />

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
            <button type="button" class="close" aria-label="Close settings" onclick={onclose}>
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M18 6l-12 12" />
                    <path d="M6 6l12 12" />
                </svg>
            </button>
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
                    onchange={(e) => prefs.setFavorites(e.currentTarget.checked)}
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
                    onchange={(e) => prefs.setRecent(e.currentTarget.checked)}
                />
            </label>
        </section>
        

        <section>
            <h3>Theme</h3>
            <p class="hint">Pick a theme for the browser chrome. System follows Windows automatically.</p>

            <div class="theme-swatches">
                {#each themes as t (t.id)}
                    <button
                        class="theme-swatch"
                        class:active={activeThemeId === t.id}
                        type="button"
                        style="background:{t.id === 'system' ? 'linear-gradient(135deg, #f8fafc 0 50%, #292524 50% 100%)' : t.bg}"
                        title={t.name}
                        aria-label={t.name}
                        onclick={() => selectTheme(t)}
                    >
                        <span class="swatch-dot" style="background:{t.accent}"></span>
                        {#if activeThemeId === t.id}
                            <span class="swatch-check">✓</span>
                        {/if}
                    </button>
                {/each}
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
                        <span class="engine-badge" style="background:{engine.color}">{engine.initial}</span>
                        <span>{engine.name}</span>
                        {#if selectedSearchEngine === engine.id}
                            <svg class="engine-check" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 7" /></svg>
                        {/if}
                    </button>
                {/each}
            </div>
        </section>

        <section>
            <h3>AI model</h3>
            <p class="hint">Used by the star assistant. Both run through OpenRouter.</p>

            <div class="provider-list" role="radiogroup" aria-label="AI model">
                {#each AI_PROVIDERS as p (p.id)}
                    <button
                        class="provider-row"
                        class:active={prefs.aiProvider === p.id}
                        type="button"
                        role="radio"
                        aria-checked={prefs.aiProvider === p.id}
                        onclick={() => selectProvider(p)}
                    >
                        <span class="provider-text">
                            <span class="provider-name">
                                {p.name}
                                <span
                                    class="info"
                                    role="button"
                                    tabindex="0"
                                    aria-label="Data disclosure for {p.name}"
                                    title="Data disclosure"
                                    onclick={(e) => showDisclosure(p, e)}
                                    onkeydown={(e) => {
                                        if (e.key === 'Enter' || e.key === ' ') {
                                            e.preventDefault();
                                            e.stopPropagation();
                                            disclosureFor = p;
                                        }
                                    }}
                                >i</span>
                            </span>
                            <span class="provider-sub">{p.modalities}</span>
                        </span>
                        {#if prefs.aiProvider === p.id}
                            <svg class="engine-check" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 7" /></svg>
                        {/if}
                    </button>
                {/each}
            </div>

            {#if disclosureFor}
                <div class="disclosure" role="status">
                    <strong>{disclosureFor.vendor}</strong>
                    <span>{disclosureFor.disclosure}</span>
                    <button type="button" class="disclosure-ok" onclick={() => (disclosureFor = null)}>Got it</button>
                </div>
            {/if}
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
                    <span class="rowsub">Stops requests to known advertising networks. Reload open tabs to apply.</span>
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

    .close {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 30px;
        height: 30px;
        padding: 0;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--text-soft);
        cursor: pointer;
    }

    .close:hover {
        background: var(--field);
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
        gap: 10px;
        margin-top: 8px;
    }

    .theme-swatch {
        position: relative;
        width: 48px;
        height: 48px;
        padding: 0;
        border: 2px solid var(--border);
        border-radius: 12px;
        cursor: pointer;
        transition: border-color 0.16s ease, transform 0.14s ease, box-shadow 0.14s ease;
        overflow: hidden;
    }

    .theme-swatch:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px var(--shadow);
    }

    .theme-swatch.active {
        border-color: var(--accent);
        box-shadow: 0 0 0 2px var(--bg-page), 0 0 0 4px var(--accent);
    }

    .swatch-dot {
        position: absolute;
        bottom: 6px;
        right: 6px;
        width: 10px;
        height: 10px;
        border-radius: 50%;
        box-shadow: 0 1px 2px rgba(0,0,0,0.2);
    }

    .swatch-check {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        font-size: 18px;
        font-weight: 700;
        color: #fff;
        text-shadow: 0 1px 3px rgba(0,0,0,0.5);
        pointer-events: none;
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
    .engine-badge { display: grid; place-items: center; flex: 0 0 auto; width: 21px; height: 21px; border-radius: 6px; color: #fff; font-size: 10px; font-weight: 800; }
    .engine-check { width: 15px; height: 15px; margin-left: auto; fill: none; stroke: var(--accent); stroke-width: 2.6; stroke-linecap: round; stroke-linejoin: round; }

    .provider-list { display: flex; flex-direction: column; gap: 8px; }
    .provider-row {
        display: flex;
        align-items: center;
        gap: 10px;
        width: 100%;
        padding: 11px 13px;
        border: 1px solid var(--border);
        border-radius: 10px;
        background: var(--field);
        color: var(--text);
        font: inherit;
        text-align: left;
        cursor: pointer;
    }
    .provider-row:hover { background: var(--tab-hover); }
    .provider-row.active { border-color: var(--accent); box-shadow: 0 0 0 1px var(--accent); }
    .provider-text { display: flex; flex-direction: column; gap: 3px; flex: 1; min-width: 0; }
    .provider-name {
        display: flex;
        align-items: center;
        gap: 7px;
        font-size: 13px;
        font-weight: 600;
        color: var(--text);
    }
    .provider-sub { font-size: 11.5px; color: var(--text-muted); }
    .info {
        display: inline-grid;
        place-items: center;
        width: 15px;
        height: 15px;
        border-radius: 50%;
        background: var(--border-strong);
        color: var(--bg-page);
        font-size: 10px;
        font-weight: 700;
        font-style: italic;
        cursor: help;
    }
    .info:hover { background: var(--accent); }
    .disclosure {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-top: 10px;
        padding: 10px 12px;
        border-radius: 10px;
        background: var(--field-strong, var(--field));
        font-size: 12px;
        color: var(--text);
    }
    .disclosure span { flex: 1; min-width: 0; color: var(--text-muted); }
    .disclosure-ok {
        flex: 0 0 auto;
        padding: 4px 11px;
        border: 0;
        border-radius: 999px;
        background: var(--accent);
        color: var(--accent-contrast);
        font: inherit;
        font-size: 11.5px;
        font-weight: 600;
        cursor: pointer;
    }

    @media (max-width: 440px) {
        .engine-list { grid-template-columns: 1fr; }
    }
</style>
