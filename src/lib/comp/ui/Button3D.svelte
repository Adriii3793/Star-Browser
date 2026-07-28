<script lang="ts">
    import type { Snippet } from 'svelte';

    let {
        label = 'Continue',
        type = 'button',
        disabled = false,
        onclick,
        children,
    }: {
        label?: string
        type?: 'button' | 'submit'
        disabled?: boolean
        onclick?: () => void
        children?: Snippet
    } =$props()
    
</script>

<button class="pushable" { type } { disabled } {onclick} >
    <span class = "shadow"></span>
    <span class = "edge"></span>
    <span class = "front">
        {@render children?.()}
        {#if !children}{label}{/if}
    </span>
</button>
<style>
    .pushable {
        position: relative;
        background: transparent;
        padding: 0;
        border: none;
        cursor: pointer;
        outline-offset: 4px;
        transition: filter 250ms;
        -webkit-tap-highlight-color: #00000000;

    }
    .pushable:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .shadow {
        position: absolute;
        top: 0;
        left: 0;
        height:100%;
        width:100%;
        background: #00000066;
        border-radius: 12px;
        filter: blur(4px);
        will-change: transform;
        transform: translateY(2px);
        transition: transform 600ms cubic-bezier(.3, .7, .4, 1);

    }

    .edge {
        position: absolute;
        top: 0;
        left: 0;
        height:100%;
        width:100%;
        border-radius: 12px;
        background: color-mix(in srgb, var(--accent, #37373f) 60%, black);
    }
    .front {
        display: block;
        position: relative;
        border-radius: 12px;
        background: var(--accent, #37373f);
        padding: 14px 32px;
        color: white;
        font-weight: 600;
        text-transform:uppercase;
        letter-spacing: 1.5px;
        font-size: 0.95rem;
        transform: translateY(-4px);
        transition: transform 600ms cubic-bezier(.3, .7, .4, 1);
    }
    .pushable:hover:not(:disabled) {
        filter: brightness(110%);

    }
    .pushable:hover:not(:disabled) .front {
        transform: translateY(-6px);
        transition: transform 250ms cubic-bezier(.3, .7, .4, 1.5);
    }

    .pushable:active:not(:disabled) .front {
        transform: translateY(-2px);
        transition: transform 34ms;


    }

    .pushable:hover:not(:disabled) .shadow {
        transform: translateY(4px);
        transition: transform 250ms cubic-bezier(.3, .7, .4, 1.5);
    }
    .pushable:active:not(:disabled) .shadow {
        transform: translateY(1px);
        transition:transform 34ms;
    }
    .pushable:focus:not(:focus-visible) {
        outline: none;
    }
    
</style>