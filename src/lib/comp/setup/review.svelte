<script lang="ts">
    import ButtonArrow from '../ui/ButtonArrow.svelte';
    import StepShell from './StepShell.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import { PRESET_THEMES, SYSTEM_THEME, theme as themeStore } from '$lib/stores/theme.svelte';

    let { onfinish, busy = false }: { onfinish: () => void; busy?: boolean } = $props();

    let engine = $derived(setup.engine);
    let theme = $derived(
        setup.data.theme === 'system'
            ? SYSTEM_THEME
            : PRESET_THEMES.find((t) => t.id === setup.data.theme)
    );
    let themeName = $derived(theme?.name ?? 'Custom');
    let surface = $derived(
        setup.data.theme === 'system'
            ? themeStore.current.surface
            : (theme?.surface ?? setup.data.customSurface ?? '#ffffff')
    );
    let initial = $derived((setup.data.name.trim()[0] ?? '?').toUpperCase());

    let brokenLogos = $state<string[]>([]);
    let logoOk = $derived(!brokenLogos.includes(engine.id));
    function markLogoBroken() {
        if (!brokenLogos.includes(engine.id)) brokenLogos = [...brokenLogos, engine.id];
    }
</script>

<StepShell
    title="You're all set"
    subtitle="Here's your setup. Change anything before you dive in."
    width={640}
>
    <div class="grid">
        <section class="preview-card" style="--stagger:0ms">
            <div class="preview-head">
                <span class="tag">{themeName}</span>
                <button class="change" type="button" onclick={() => setup.goto('style')}>Change</button>
            </div>
            <div
                class="preview-body"
                style="background:{setup.data.background
                    ? `url(${setup.data.background}) center/cover`
                    : surface}"
            >
                <p class="greet">Good afternoon, {setup.data.name || 'there'}</p>
                <div class="searchbar">
                    <span class="badge sm" class:lettered={!logoOk} style={logoOk ? '' : `background:${engine.color}`}>
                        {#if logoOk}
                            <img src={engine.logo} alt="" onerror={markLogoBroken} />
                        {:else}
                            {engine.initial}
                        {/if}
                    </span>
                    <span class="ph">Search {engine.name}</span>
                </div>
            </div>
        </section>

        <div class="side">
            <section class="row-card" style="--stagger:60ms">
                <span class="badge" class:lettered={!logoOk} style={logoOk ? '' : `background:${engine.color}`}>
                    {#if logoOk}
                        <img src={engine.logo} alt="" onerror={markLogoBroken} />
                    {:else}
                        {engine.initial}
                    {/if}
                </span>
                <span class="meta">
                    <span class="label">Search engine</span>
                    <span class="value">{engine.name}</span>
                </span>
                <button class="change" type="button" onclick={() => setup.goto('search')}>Change</button>
            </section>

            <section class="row-card" style="--stagger:120ms">
                <span class="avatar">
                    {#if setup.data.avatar}
                        <img src={setup.data.avatar} alt="" />
                    {:else}
                        {initial}
                    {/if}
                </span>
                <span class="meta">
                    <span class="label">Profile</span>
                    <span class="value">{setup.data.name || 'there'}</span>
                </span>
                <button class="change" type="button" onclick={() => setup.goto('profile')}>Change</button>
            </section>
        </div>
    </div>

    {#snippet footer()}
        <div class="cta" style="--stagger:180ms">
            <ButtonArrow label={busy ? 'Saving…' : 'Explore'} disabled={busy} onclick={onfinish} />
        </div>
    {/snippet}
</StepShell>

<style>
    .grid {
        display: grid;
        grid-template-columns: 1.35fr 1fr;
        gap: 16px;
        width: 100%;
    }

    .preview-card,
    .row-card,
    .cta {
        animation: rise 250ms ease-in-out backwards;
        animation-delay: var(--stagger, 0ms);
    }

    @keyframes rise {
        from {
            opacity: 0;
            transform: translateY(8px);
        }
    }

    .preview-card {
        display: flex;
        flex-direction: column;
        border-radius: 16px;
        overflow: hidden;
        background: var(--bg-page, #fff);
        border: 1px solid var(--border);
        box-shadow: 0 8px 24px var(--shadow);
    }

    .preview-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        padding: 16px 16px 8px;
    }

    .tag {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--text-muted);
    }

    .preview-body {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 16px;
        flex: 1;
        min-height: 160px;
        padding: 24px 16px;
        background-size: cover;
        background-position: center;
    }

    .greet {
        margin: 0;
        font-size: 15px;
        font-weight: 600;
        color: var(--text);
        text-shadow: 0 1px 4px rgba(0, 0, 0, 0.18);
    }

    .searchbar {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        max-width: 260px;
        padding: 8px 16px;
        border-radius: 999px;
        background: var(--bg-page);
        border: 1px solid var(--border);
        backdrop-filter: blur(8px);
    }

    .ph {
        font-size: 12px;
        color: var(--text-soft);
    }

    .side {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .row-card {
        display: flex;
        align-items: center;
        gap: 16px;
        flex: 1;
        padding: 16px;
        border-radius: 16px;
        background: var(--field, #f7f1ec);
        border: 1px solid transparent;
        transition: border-color 200ms ease-in-out, background-color 200ms ease-in-out;
    }

    .row-card:hover {
        background: var(--bg-page, #fff);
        border-color: var(--border-strong);
    }

    .badge,
    .avatar {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 40px;
        height: 40px;
        color: #ffffff;
        font-weight: 700;
        overflow: hidden;
    }

    .badge {
        border-radius: 12px;
        font-size: 16px;
        background: var(--field, #f7f1ec);
        padding: 6px;
    }

    .badge.lettered {
        padding: 0;
    }

    .badge img {
        width: 100%;
        height: 100%;
        object-fit: contain;
        border-radius: 4px;
    }

    .badge.sm {
        width: 20px;
        height: 20px;
        border-radius: 6px;
        font-size: 10px;
        padding: 3px;
        background: var(--field, #f7f1ec);
    }

    .badge.sm.lettered {
        padding: 0;
    }

    .badge.sm img {
        border-radius: 2px;
    }

    .avatar {
        border-radius: 50%;
        background: var(--accent, #80A4D4);
        color: var(--accent-contrast, #1c1917);
        font-size: 16px;
    }

    .avatar img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .meta {
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex: 1;
        min-width: 0;
    }

    .label {
        font-size: 11px;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--text-muted);
        line-height: 1;
    }

    .value {
        font-size: 15px;
        font-weight: 600;
        color: var(--text);
        line-height: 1;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .change {
        flex: 0 0 auto;
        align-self: center;
        min-height: 32px;
        padding: 8px;
        border: none;
        background: none;
        color: var(--accent, #80A4D4);
        font: inherit;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        border-radius: 8px;
        transition: background-color 150ms ease-in-out;
    }

    .change:hover {
        background: color-mix(in srgb, var(--accent) 14%, transparent);
    }

    .change:focus-visible {
        outline: 2px solid var(--accent, #80A4D4);
        outline-offset: 2px;
    }

    @media (max-width: 640px) {
        .grid {
            grid-template-columns: 1fr;
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .preview-card,
        .row-card,
        .cta {
            animation: none;
        }
        .row-card,
        .change {
            transition: none;
        }
    }
</style>
