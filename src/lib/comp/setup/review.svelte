<script lang="ts">
    import ButtonArrow from '../ui/ButtonArrow.svelte';
    import { setup } from '$lib/stores/setup.svelte';
    import { PRESET_THEMES } from '$lib/stores/theme.svelte';

    let { onfinish, busy = false }: { onfinish: () => void; busy?: boolean } = $props();

    let engine = $derived(setup.engine);
    let theme = $derived(PRESET_THEMES.find((t) => t.id === setup.data.theme));
    let themeName = $derived(theme?.name ?? 'Custom');
    let surface = $derived(theme?.surface ?? setup.data.customSurface ?? '#ffffff');
    let initial = $derived((setup.data.name.trim()[0] ?? '?').toUpperCase());
</script>

<div class="wrap">
    <header class="head">
        <h1>You're all set</h1>
        <p class="sub">Here's your setup. Change anything before you dive in.</p>
    </header>

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
                    <span class="badge sm" style="background:{engine.color}">{engine.initial}</span>
                    <span class="ph">Search {engine.name}</span>
                </div>
            </div>
        </section>

        <div class="side">
            <section class="row-card" style="--stagger:60ms">
                <span class="badge" style="background:{engine.color}">{engine.initial}</span>
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

    <div class="cta" style="--stagger:180ms">
        <ButtonArrow label={busy ? 'Saving…' : 'Explore'} disabled={busy} onclick={onfinish} />
    </div>
</div>

<style>
    .wrap {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 32px;
        width: 100%;
        max-width: 640px;
    }

    .head {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 8px;
        text-align: center;
    }

    h1 {
        margin: 0;
        font-size: 28px;
        font-weight: 600;
        letter-spacing: -0.01em;
        color: var(--text);
    }

    .sub {
        margin: 0;
        font-size: 14px;
        color: var(--text-muted);
    }

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
        border: 1px solid rgba(74, 58, 46, 0.08);
        box-shadow: 0 8px 24px rgba(74, 58, 46, 0.08);
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
        background: rgba(255, 255, 255, 0.9);
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
        border-color: rgba(74, 58, 46, 0.12);
    }

    .badge,
    .avatar {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 0 0 auto;
        width: 40px;
        height: 40px;
        color: #fff;
        font-weight: 700;
        overflow: hidden;
    }

    .badge {
        border-radius: 12px;
        font-size: 16px;
    }

    .badge.sm {
        width: 20px;
        height: 20px;
        border-radius: 6px;
        font-size: 10px;
    }

    .avatar {
        border-radius: 50%;
        background: var(--accent, #e8734a);
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
        color: var(--accent, #e8734a);
        font: inherit;
        font-size: 13px;
        font-weight: 600;
        cursor: pointer;
        border-radius: 8px;
        transition: background-color 150ms ease-in-out;
    }

    .change:hover {
        background: rgba(232, 115, 74, 0.1);
    }

    .change:focus-visible {
        outline: 2px solid var(--accent, #e8734a);
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
