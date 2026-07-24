<script lang="ts">
    interface Props {
        size?: number;
        duration?: string;
        showText?: boolean;
    }

    let { size= 180, duration = '1.6s', showText = false }: Props = $props();

    const uid = crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
    const gradientId = `sparkle-gold-${uid}`;
    const shapeId = `sparkle-shape-${uid}`;

    const thinkingMessages = [
        'Understanding context...',
        'Analyzing information...',
        'Processing request...',
        'Reasoning through response...',
        'Formulating answer...',
        'Considering details...',
        'Loading output...',
        'Generating response...',
        'Evaluating options...',
        'Synthesizing results...',
        'Preparing reply...',
        'Almost ready...'
    ];

    let currentMessageIndex = $state(0);

    $effect(() => {
        if (!showText) return;
        const interval = setInterval(() => {
            currentMessageIndex = (currentMessageIndex + 1) % thinkingMessages.length;
        }, 1200);
        return () => clearInterval(interval);
    });
</script>
<div class="sparkle-wrapper" style="--loader-size: {size}px">
    <div class="loader-container">
        <svg
            class="sparkle-loader"
            viewBox="0 0 300 300"
            xmlns="http://www.w3.org/2000/svg"
            role="img"
            aria-label="Loading"
        >
            <defs>
                <linearGradient id={gradientId} x1="0" y1="1" x2="1" y2="0">
                    <stop offset="0"    stop-color="var(--gold-deep)" />
                    <stop offset="0.55" stop-color="var(--gold-core)" />
                    <stop offset="1"    stop-color="var(--gold-light)" />
                </linearGradient>
            
                <path
                    id={shapeId}
                    d="M 50 2 Q 56.36 43.64 98 50 Q 56.36 56.36 50 98 Q 43.64 56.36 2 50 Q 43.64 43.64 50 2 Z" 
                />
            </defs>

            <g transform="translate(10 95) scale(1.9)">
                <use href="#{shapeId}" class="star star--sm" fill="url(#{gradientId})" />
            </g>
            <g transform="translate(157.5 32.5) scale(1.15)">
                <use href="#{shapeId}"  class="star star--md" fill="url(#{gradientId})" />
            </g>
            <g transform="translate(112.5 32.5) scale(0.75)">
                <use href="#{shapeId}" class="star star--sm" fill="url(#{gradientId})" />
            </g>
        </svg>
        {#if showText}
            <div class="thinking-text">
                <p class="message">{thinkingMessages[currentMessageIndex]}</p>
            </div>
        {/if}
    </div>
</div>


<style>
    .sparkle-wrapper {
        --loader-size:  180px;
        --gold-deep:    #8a5411;
        --gold-core:    #dfae44;
        --gold-light:   #f8e7ad;
        --glow:         rgba(224, 173, 74, 0.40);
        --bg:           transparent;

        --twinkle-duration: 1.6s;
        --twinkle-ease: ease-in-out;
        --scale-min:    0.40;
        --opacity-min:  0.18;

        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--bg);
        width: 100%;
        height: 100%;
    }

    .loader-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 16px;
    }

    .sparkle-loader {
        width: var(--loader-size);
        height: var(--loader-size);
    }

    .sparkle-loader .star {
        transform-box: fill-box;
        transform-origin: center;
        filter: drop-shadow(0 0 5px var(--glow));
        animation: twinkle var(--twinkle-duration) var(--twinkle-ease) infinite;
    }

    .sparkle-loader .star--md { 
        animation-delay: -0.55s; 
    }

    .sparkle-loader .star--sm { 
        animation-delay: -1.05s 
    }

    .thinking-text {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 24px;
    }

    .message {
        margin: 0;
        font-size: 14px;
        color: var(--text-soft, #8a6b57);
        font-weight: 500;
        animation: fadeInOut 1.2s ease-in-out;
    }

    @keyframes fadeInOut {
        0% {
            opacity: 0;
        }
        10% {
            opacity: 1;
        }
        90% {
            opacity: 1;
        }
        100% {
            opacity: 0;
        }
    }

    @keyframes twinkle{ 
        0%, 
        100% { 
            transform: scale(var(--scale-min)); 
            opacity: var(--opacity-min); 
        }
        50% { 
            transform: scale(1); 
            opacity: 1; 
        }
    }

    @media (prefers-reduced-motion: reduce) {
        .sparkle-loader .star { 
            animation: none; 
            opacity: 1; 
        }
        .message {
            animation: none;
        }
    }
</style>
