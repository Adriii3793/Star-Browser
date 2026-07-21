<script lang="ts">
	import { ai } from '$lib/stores/ai.svelte';

	let { username = 'there', onclose }: { username?: string; onclose: () => void } = $props();

	let draft = $state('');
	let listEl = $state<HTMLElement>();

	ai.init();

	const prompts = ['Summarise this page', 'Learn more about this topic'];

	$effect(() => {
		void ai.messages.length;
		listEl?.scrollTo({ top: listEl.scrollHeight, behavior: 'smooth' });
	});

	function submit() {
		if (!draft.trim()) return;
		ai.send(draft);
		draft = '';
	}

	function keydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			submit();
		}
	}
</script>

<aside class="panel">
	<header class="head">
		<div class="group">
			<button class="icon" aria-label="AI Panel" type="button">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z" />
				</svg>
			</button>
			<button class="icon" aria-label="New Conversation" type="button" onclick={() => ai.reset()}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
			</button>
		</div>
		<div class="group">
			<button class="icon" aria-label="More" type="button">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<circle cx="5" cy="12" r="1.4" /><circle cx="12" cy="12" r="1.4" /><circle cx="19" cy="12" r="1.4" />
				</svg>
			</button>
			<button class="icon" aria-label="Close" type="button" onclick={onclose}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
			</button>
		</div>
	</header>

	<div class="body" bind:this={listEl}>
		{#if ai.messages.length === 0}
			<div class="hero">
				<h1>Hello {username}, what's on your mind?</h1>
			</div>

			<button class="identity" type="button">
				<span>OpenRouter</span>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
			</button>

			<div class="prompts">
				{#each prompts as prompt (prompt)}
					<button class="chip" type="button" onclick={() => ai.send(prompt)}>{prompt}</button>
				{/each}
			</div>
		{:else}
			<ul class="messages">
				{#each ai.messages as message, i (i)}
					<li class="msg {message.role}">{message.content}</li>
				{/each}
				{#if ai.sending}
					<li class="msg assistant thinking">…</li>
				{/if}
			</ul>
		{/if}

		{#if ai.error}
			<p class="err">{ai.error}</p>
		{/if}
	</div>

	<div class="composer">
		<textarea
			rows="1"
			placeholder="Send a message or @ tag a tab"
			bind:value={draft}
			onkeydown={keydown}
		></textarea>
		<div class="toolbar">
			<div class="group">
				<button class="tool" aria-label="Attach File" type="button">
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
				</button>
				<span class="model">{ai.used}/{ai.limit}</span>
			</div>
			<button class="tool mic" aria-label="Dictation" type="button">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 4v16M8 8v8M16 8v8M4 11v2M20 11v2" />
				</svg>
			</button>
		</div>
	</div>
</aside>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		width: 340px;
		margin: 0 8px 8px 0;
		border: 1px solid rgba(0, 0, 0, 0.05);
		border-radius: 16px;
		background: var(--bg-page, #fff);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.06);
		overflow: hidden;
		font-family:
			Inter,
			-apple-system,
			BlinkMacSystemFont,
			'SF Pro Text',
			'Segoe UI',
			sans-serif;
	}

	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px;
	}
	.group {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--text-muted, #ac8064);
		cursor: default;
		transition: background-color 150ms ease-in-out;
	}
	.icon:hover {
		background: var(--field, #f7f1ec);
	}
	.icon svg {
		width: 18px;
		height: 18px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.body {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		padding: 16px;
		overflow-y: auto;
	}

	.hero h1 {
		margin: 24px 0;
		font-size: 26px;
		font-weight: 600;
		line-height: 1.2;
		color: var(--text, #4a3a2e);
		text-align: left;
	}

	.identity {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		align-self: flex-start;
		padding: 8px 16px;
		border: 0;
		border-radius: 16px;
		background: var(--field, #f7f1ec);
		color: var(--text-soft, #8a6b57);
		font: inherit;
		font-size: 13px;
		font-weight: 500;
		cursor: default;
	}
	.identity svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.prompts {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 24px;
	}
	.chip {
		align-self: flex-start;
		padding: 8px 16px;
		border: 1px solid rgba(0, 0, 0, 0.08);
		border-radius: 16px;
		background: var(--bg-page, #fff);
		color: var(--text-soft, #8a6b57);
		font: inherit;
		font-size: 13px;
		text-align: left;
		cursor: default;
		transition: background-color 150ms ease-in-out;
	}
	.chip:hover {
		background: var(--field, #f7f1ec);
	}

	.messages {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.msg {
		max-width: 85%;
		padding: 8px 16px;
		border-radius: 16px;
		font-size: 14px;
		line-height: 1.4;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.msg.user {
		align-self: flex-end;
		background: var(--accent, #e8734a);
		color: #fff;
	}
	.msg.assistant {
		align-self: flex-start;
		background: var(--field, #f7f1ec);
		color: var(--text, #4a3a2e);
	}
	.msg.thinking {
		opacity: 0.6;
	}

	.err {
		margin-top: 8px;
		font-size: 12px;
		color: #c0392b;
	}

	.composer {
		margin: 8px;
		border: 1px solid rgba(0, 0, 0, 0.08);
		border-radius: 16px;
		background: var(--bg-page, #fff);
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
	}
	textarea {
		width: 100%;
		max-height: 120px;
		padding: 16px 16px 8px;
		border: 0;
		background: transparent;
		color: var(--text, #4a3a2e);
		font: inherit;
		font-size: 14px;
		resize: none;
		box-sizing: border-box;
	}
	textarea:focus {
		outline: none;
	}
	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px;
	}
	.tool {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 8px;
		background: transparent;
		color: var(--text-muted, #ac8064);
		cursor: default;
		transition: background-color 150ms ease-in-out;
	}
	.tool:hover {
		background: var(--field, #f7f1ec);
	}
	.tool svg {
		width: 18px;
		height: 18px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.model {
		font-size: 12px;
		color: var(--text-muted, #ac8064);
	}

	@media (prefers-reduced-motion: reduce) {
		.icon,
		.chip,
		.tool {
			transition: none;
		}
	}
</style>