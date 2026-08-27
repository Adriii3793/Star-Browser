<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        title,
        subtitle = '',
        width = 380,
        children,
        footer
    }: {
        title: string;
        subtitle?: string;
        width?: number;
        children?: Snippet;
        footer?: Snippet;
    } = $props();
</script>

<section class="step" style="--step-width:{width}px">
    <header>
        <h1>{title}</h1>
        {#if subtitle}<p class="sub">{subtitle}</p>{/if}
    </header>

    {#if children}
        <div class="body">{@render children()}</div>
    {/if}

    {#if footer}
        <div class="footer">{@render footer()}</div>
    {/if}
</section>

<style>
    .step {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 100%;
        max-width: var(--step-width, 380px);
    }

    header {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
        text-align: center;
    }

    h1 {
        margin: 0;
        font-size: 26px;
        font-weight: 600;
        letter-spacing: -0.01em;
        line-height: 1.2;
        color: var(--text);
    }

    .sub {
        margin: 0;
        font-size: 13.5px;
        line-height: 1.4;
        color: var(--text-muted);
    }

    .body {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 16px;
        width: 100%;
        margin-top: 24px;
    }

    .footer {
        display: flex;
        justify-content: center;
        margin-top: 28px;
    }
</style>
