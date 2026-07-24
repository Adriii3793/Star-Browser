<script lang="ts">
	import { ai } from '$lib/stores/ai.svelte';
	import type { ChatMessage, ContentPart } from '$lib/services/ai';
	import Loading from '../ui/Loading.svelte';

	let { username = 'there', onclose }: { username?: string; onclose: () => void } = $props();

	let draft = $state('');
	let listEl = $state<HTMLElement>();
	let fileInputEl = $state<HTMLInputElement>();
	let dragActive = $state(false);
	let dragDepth = 0;
	let attachments = $state<Array<{ type: 'image' | 'video' | 'text'; data: string; name: string }>>([]);
	let showHistory = $state(false);
	const HISTORY_KEY = 'ai_chat_history';
	type ChatEntry = { id: string; title: string; messages: typeof ai.messages; timestamp: number };
	let chatHistory = $state<ChatEntry[]>(loadHistoryFromStorage());
	let activeChatId = $state<string | null>(null);

	ai.init();

	const prompts = ['Summarise this page', 'Learn more about this topic'];

	$effect(() => {
		void ai.messages.length;
		listEl?.scrollTo({ top: listEl.scrollHeight, behavior: 'smooth' });
	});

	function loadHistoryFromStorage(): ChatEntry[] {
		try {
			const saved = localStorage.getItem(HISTORY_KEY);
			return saved ? JSON.parse(saved) : [];
		} catch (e) {
			console.error('Failed to load chat history:', e);
			return [];
		}
	}

	function persistHistory(list: ChatEntry[]) {
		try {
			localStorage.setItem(HISTORY_KEY, JSON.stringify(list));
		} catch (e) {
			console.error('Failed to save chat history:', e);
		}
	}

	function messageText(message: ChatMessage): string {
		if (typeof message.content === 'string') return message.content.trim();
		return message.content
			.filter((p) => p.type === 'text')
			.map((p) => p.text)
			.join('\n')
			.trim();
	}

	function messageImages(message: ChatMessage): string[] {
		if (typeof message.content === 'string') return [];
		return message.content.filter((p) => p.type === 'image_url').map((p) => p.image_url.url);
	}

	function saveCurrentChat() {
		if (ai.messages.length === 0) return;
		const title = messageText(ai.messages[0]).substring(0, 50) || 'New Chat';

		if (activeChatId) {
			const idx = chatHistory.findIndex((c) => c.id === activeChatId);
			if (idx !== -1) {
				const updated = [...chatHistory];
				updated[idx] = { ...updated[idx], messages: ai.messages, title };
				chatHistory = updated;
				persistHistory(updated);
				return;
			}
		}

		activeChatId = crypto.randomUUID();
		const updated = [...chatHistory, { id: activeChatId, title, messages: ai.messages, timestamp: Date.now() }];
		chatHistory = updated;
		persistHistory(updated);
	}

	function decodeTextAttachment(dataUrl: string): string | null {
		try {
			return atob(dataUrl.split(',')[1] ?? '');
		} catch {
			return null;
		}
	}

	async function submit() {
		const text = draft.trim();
		if (!text && attachments.length === 0) return;
		const files = attachments;
		draft = '';
		attachments = [];

		if (files.length === 0) {
			await ai.send(text);
		} else {
			const parts: ContentPart[] = [];
			if (text) parts.push({ type: 'text', text });
			for (const file of files) {
				if (file.type === 'image') {
					parts.push({ type: 'image_url', image_url: { url: file.data } });
				} else if (file.type === 'text') {
					const decoded = decodeTextAttachment(file.data);
					parts.push({
						type: 'text',
						text: decoded ? `File "${file.name}":\n${decoded}` : `[attached file: ${file.name}]`
					});
				} else {
					parts.push({ type: 'text', text: `[attached video: ${file.name}]` });
				}
			}
			await ai.send(parts);
		}
		saveCurrentChat();
	}

	async function sendPrompt(prompt: string) {
		await ai.send(prompt);
		saveCurrentChat();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			e.stopPropagation();
			submit();
		}
	}

	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragDepth += 1;
		dragActive = true;
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragDepth = Math.max(0, dragDepth - 1);
		if (dragDepth === 0) dragActive = false;
	}

	function attachFile(file: File) {
		const isImage = file.type.startsWith('image/');
		const isVideo = file.type.startsWith('video/');
		const isText = file.type.startsWith('text/') || file.type === 'application/json';
		if (!isImage && !isVideo && !isText) return;

		const reader = new FileReader();
		reader.onload = (event) => {
			const data = event.target?.result as string;
			attachments = [
				...attachments,
				{
					type: isImage ? 'image' : isVideo ? 'video' : 'text',
					data,
					name: file.name || (isImage ? 'pasted-image.png' : 'attachment')
				}
			];
		};
		reader.readAsDataURL(file);
	}

	function collectFiles(dt: DataTransfer | null): File[] {
		if (!dt) return [];
		if (dt.files && dt.files.length > 0) return Array.from(dt.files);
		// Some sources (browser images, screenshots) expose data only via items.
		const files: File[] = [];
		for (const item of Array.from(dt.items ?? [])) {
			if (item.kind === 'file') {
				const file = item.getAsFile();
				if (file) files.push(file);
			}
		}
		return files;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		dragDepth = 0;
		dragActive = false;
		for (const file of collectFiles(e.dataTransfer)) attachFile(file);
	}

	function handlePaste(e: ClipboardEvent) {
		const files = collectFiles(e.clipboardData);
		if (files.length === 0) return;
		// Only intercept when there is an actual file/image; keep normal text paste.
		e.preventDefault();
		for (const file of files) attachFile(file);
	}

	function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		if (input.files) for (const file of Array.from(input.files)) attachFile(file);
		input.value = '';
	}

	function removeAttachment(index: number) {
		attachments = attachments.filter((_, i) => i !== index);
	}

	function loadChatHistory(chat: ChatEntry) {
		ai.setMessages([...chat.messages]);
		activeChatId = chat.id;
		showHistory = false;
	}

	function clearChatHistory() {
		chatHistory = [];
		activeChatId = null;
		localStorage.removeItem(HISTORY_KEY);
	}

	function toggleHistory() {
		showHistory = !showHistory;
	}

	function newConversation() {
		ai.reset();
		activeChatId = null;
		showHistory = false;
	}
</script>

<aside
	class="panel"
	class:drag-active={dragActive}
	ondragenter={handleDragEnter}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
>
	<header class="head">
		<div class="group">
			<button class="icon" aria-label="AI Panel" type="button">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z" />
				</svg>
			</button>
			<button class="icon" aria-label="New Conversation" type="button" onclick={newConversation}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
			</button>
		</div>
		<div class="group">
			<button class="icon" aria-label="Chat History" type="button" onclick={toggleHistory}>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 8l0 4l2 2" />
					<path d="M3.05 11a9 9 0 1 1 .5 4m-.5 5v-5h5" />
				</svg>
			</button>
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

	{#if showHistory}
		<div class="history-panel">
			<div class="history-header">
				<h3>Chat History</h3>
				<button class="clear-btn" type="button" onclick={clearChatHistory} title="Clear all history">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M4 7l16 0" />
						<path d="M10 11l0 6" />
						<path d="M14 11l0 6" />
						<path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12" />
						<path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" />
					</svg>
				</button>
			</div>
			<div class="history-list">
				{#if chatHistory.length === 0}
					<p class="empty-history">No chat history yet</p>
				{:else}
					{#each chatHistory as chat (chat.id)}
						<button class="history-item" type="button" onclick={() => loadChatHistory(chat)}>
							<span class="history-title">{chat.title}</span>
							<span class="history-time">{new Date(chat.timestamp).toLocaleDateString()}</span>
						</button>
					{/each}
				{/if}
			</div>
		</div>
	{/if}

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
					<button class="chip" type="button" onclick={() => sendPrompt(prompt)}>{prompt}</button>
				{/each}
			</div>
		{:else}
			<ul class="messages">
				{#each ai.messages as message, i (i)}
					<li class="msg {message.role}">
						{#each messageImages(message) as img (img)}
							<img class="msg-image" src={img} alt="attachment" />
						{/each}
						{#if messageText(message)}
							<p class="msg-text">{messageText(message)}</p>
						{/if}
					</li>
				{/each}
				{#if ai.sending}
					<li class="msg assistant thinking">
						<div class="loading-wrapper">
							<Loading size={24} showText={true} />
						</div>
					</li>
				{/if}
			</ul>
		{/if}

		{#if ai.error}
			<p class="err">{ai.error}</p>
		{/if}
	</div>

	{#if attachments.length > 0}
		<div class="attachments">
			{#each attachments as attachment, i (i)}
				<div class="attachment-item">
					{#if attachment.type === 'image'}
						<img src={attachment.data} alt={attachment.name} />
					{:else if attachment.type === 'video'}
						<video controls>
							<source src={attachment.data} />
						</video>
					{:else}
						<div class="text-file">
							<svg viewBox="0 0 24 24" aria-hidden="true">
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
								<polyline points="14 2 14 8 20 8" />
							</svg>
							<span>{attachment.name}</span>
						</div>
					{/if}
					<button class="remove-attachment" type="button" onclick={() => removeAttachment(i)} title="Remove">
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
					</button>
				</div>
			{/each}
		</div>
	{/if}

	<div class="composer" class:drag-over={dragActive}>
		<textarea
			rows="1"
			placeholder="Send a message or drag files here"
			bind:value={draft}
			onkeydown={handleKeydown}
			onpaste={handlePaste}
		></textarea>
		<div class="toolbar">
			<div class="group">
				<button class="tool" aria-label="Attach File" type="button" onclick={() => fileInputEl?.click()}>
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
				</button>
				<span class="model">{ai.used}/{ai.limit}</span>
			</div>
			<button class="tool send" aria-label="Send" type="button" onclick={submit}>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
				</svg>
			</button>
		</div>
	</div>

	<input
		type="file"
		multiple
		accept="image/*,video/*,.txt,.json"
		bind:this={fileInputEl}
		onchange={handleFileSelect}
		style="display: none"
	/>
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
		transition: border-color 0.2s ease;
	}

	.panel.drag-active {
		border-color: var(--accent, #e8734a);
		background: rgba(232, 115, 74, 0.02);
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
		cursor: pointer;
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
		cursor: pointer;
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
	.msg-text {
		margin: 0;
		white-space: pre-wrap;
		word-break: break-word;
	}
	.msg-image {
		display: block;
		max-width: 100%;
		max-height: 220px;
		border-radius: 10px;
		object-fit: contain;
	}
	.msg-image + .msg-text {
		margin-top: 8px;
	}
	.msg.thinking {
		opacity: 0.7;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 12px;
	}

	.loading-wrapper {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
	}

	.loading-wrapper :global(.sparkle-wrapper) {
		--loader-size: 24px;
	}

	.loading-wrapper :global(.sparkle-loader) {
		width: 24px;
		height: 24px;
	}

	.loading-wrapper :global(.message) {
		font-size: 11px;
	}

	.err {
		margin-top: 8px;
		font-size: 12px;
		color: #c0392b;
	}

	.history-panel {
		display: flex;
		flex-direction: column;
		border-bottom: 1px solid rgba(0, 0, 0, 0.05);
		background: var(--field, #f7f1ec);
		max-height: 200px;
		overflow: hidden;
	}

	.history-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.05);
	}

	.history-header h3 {
		margin: 0;
		font-size: 13px;
		font-weight: 600;
		color: var(--text, #4a3a2e);
	}

	.clear-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--text-muted, #ac8064);
		cursor: pointer;
		transition: background-color 150ms ease-in-out;
	}

	.clear-btn:hover {
		background: rgba(0, 0, 0, 0.06);
	}

	.clear-btn svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.history-list {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.empty-history {
		margin: 0;
		padding: 16px;
		font-size: 12px;
		color: var(--text-muted, #ac8064);
		text-align: center;
	}

	.history-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 10px 12px;
		border: none;
		background: transparent;
		text-align: left;
		cursor: pointer;
		transition: background-color 150ms ease-in-out;
		border-bottom: 1px solid rgba(0, 0, 0, 0.03);
	}

	.history-item:hover {
		background: rgba(0, 0, 0, 0.04);
	}

	.history-title {
		font-size: 12px;
		color: var(--text, #4a3a2e);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.history-time {
		font-size: 10px;
		color: var(--text-muted, #ac8064);
	}

	.attachments {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		padding: 8px;
		border-bottom: 1px solid rgba(0, 0, 0, 0.05);
		background: var(--field, #f7f1ec);
	}

	.attachment-item {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 60px;
		height: 60px;
		border-radius: 8px;
		background: var(--bg-page, #fff);
		border: 1px solid rgba(0, 0, 0, 0.08);
		overflow: hidden;
	}

	.attachment-item img,
	.attachment-item video {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.text-file {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 4px;
		width: 100%;
		height: 100%;
		padding: 4px;
	}

	.text-file svg {
		width: 20px;
		height: 20px;
		fill: none;
		stroke: var(--text-soft, #8a6b57);
		stroke-width: 1.5;
	}

	.text-file span {
		font-size: 8px;
		color: var(--text-soft, #8a6b57);
		text-align: center;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.remove-attachment {
		position: absolute;
		top: -8px;
		right: -8px;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		padding: 0;
		border: none;
		border-radius: 50%;
		background: var(--accent, #e8734a);
		color: #fff;
		cursor: pointer;
		opacity: 0;
		transition: opacity 150ms ease-in-out;
	}

	.attachment-item:hover .remove-attachment {
		opacity: 1;
	}

	.remove-attachment svg {
		width: 12px;
		height: 12px;
		fill: none;
		stroke: currentColor;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.composer {
		margin: 8px;
		border: 1px solid rgba(0, 0, 0, 0.08);
		border-radius: 16px;
		background: var(--bg-page, #fff);
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.04);
		transition: border-color 0.2s ease, background-color 0.2s ease;
	}

	.composer.drag-over {
		border-color: var(--accent, #e8734a);
		background: rgba(232, 115, 74, 0.02);
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
		cursor: pointer;
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

	.tool.send {
		background: var(--accent, #e8734a);
		color: #fff;
	}

	.tool.send:hover {
		background: var(--accent-hover, #d85a2f);
	}

	.model {
		font-size: 12px;
		color: var(--text-muted, #ac8064);
	}

	@media (prefers-reduced-motion: reduce) {
		.icon,
		.chip,
		.tool,
		.panel,
		.composer {
			transition: none;
		}
	}
</style>