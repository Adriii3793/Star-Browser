<script lang="ts">
	import { ai } from '$lib/stores/ai.svelte';
	import type { ChatMessage, ContentPart } from '$lib/services/ai';
	import Loading from '../ui/Loading.svelte';
	import {memory} from '$lib/stores/memory.svelte';
	import { fetchPageContext } from '$lib/services/ai';
	import { renderMarkdown } from '$lib/services/markdown';
	import { cubicOut } from 'svelte/easing';

	/**
	 * Slide the panel in from the right edge and back out again. A CSS animation could
	 * only ever run on open, because the parent's {#if} rips the element straight out
	 * of the DOM on close; a Svelte transition holds it there long enough to animate.
	 */
	function slidePanel(node: HTMLElement, { duration = 260 } = {}) {
		const reduced =
			typeof matchMedia !== 'undefined' &&
			matchMedia('(prefers-reduced-motion: reduce)').matches;
		const width = node.offsetWidth;
		return {
			duration: reduced ? 0 : duration,
			easing: cubicOut,
			css: (t: number) => `
				opacity: ${t};
				transform: translateX(${(1 - t) * (width * 0.35 + 24)}px);
			`
		};
	}
    import { page } from '$app/state';
	let showMemory = $state(false);
	let memoryDraft = $state('');
	let memoryFileEl = $state<HTMLInputElement>();
	let toast = $state<string | null>(null);
	let toastTimer: ReturnType<typeof setTimeout> | undefined;
	let { username = 'there', pageUrl = null, onclose }: { username?: string; pageUrl?: string | null; onclose: () => void } = $props();

	let draft = $state('');
	let listEl = $state<HTMLElement>();
	let fileInputEl = $state<HTMLInputElement>();
	let dragActive = $state(false);
	let dragDepth = 0;
	let attachments = $state<Array<{ type: 'image' | 'video' | 'text'; data: string; name: string }>>([]);
	const PANEL_MIN = 300;
	const PANEL_MAX = 720;
	const WIDTH_KEY = 'ai_panel_width';
	let panelWidth = $state(loadPanelWidth());
	let resizing = $state(false);
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

	function showToast(msg: string) {
		toast = msg;
		clearTimeout(toastTimer);
		toastTimer = setTimeout(() => (toast = null), 6000);
	}

	function loadPanelWidth(): number {
		const v = Number(localStorage.getItem(WIDTH_KEY));
		return Number.isFinite(v) && v>= PANEL_MIN && v <= PANEL_MAX?v: 340;
		
	}

	function startResize(e: PointerEvent) {
		e.preventDefault();
		resizing = true;
		const startX = e.clientX;
		const startW = panelWidth;
		const onMove = (ev: PointerEvent) => {
			panelWidth = Math.max(PANEL_MIN, Math.min(PANEL_MAX, startW + (startX - ev.clientX)));

		};
		const onUp = () => {
			resizing = false;
			localStorage.setItem(WIDTH_KEY, String(panelWidth));
			window.removeEventListener('pointermove', onMove);
			window.removeEventListener('pointerup', onUp);
		};
		window.addEventListener('pointermove', onMove);
		window.addEventListener('pointerup', onUp);
	}

	function toggleMemory() { 
		showMemory =!showMemory;
		if (showMemory) showHistory = false;
	}
	function addMemory() {
		if (memory.add(memoryDraft)) memoryDraft = '';
		else if (memory.lastError) showToast(memory.lastError);
	}
	function exportMemory() {
		const blob = new Blob([memory.export()], {type: 'text/markdown'});
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'ai-memory.md';
		document.body.appendChild(a);
		a.click();
		a.remove();
		setTimeout(() => URL.revokeObjectURL(url),10_000);
		showToast('Saved ai-memory.md to your Downloads folder');
	}
	function importMemory(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		const reader = new FileReader();
		reader.onload = () => {
			const r = memory.import(String(reader.result ?? ''), 'merge');
			const bits = [`Imported ${r.added}`];
			if (r.duplicates) bits.push(`${r.duplicates} duplicate(s) skipped`);
			if (r.rejected.length) bits.push(`${r.rejected.length} blocked`);
			showToast(bits.join(' · '));
			if (r.rejected.length) console.warn('Blocked memory entries:', r.rejected);
		};
		reader.readAsText(file);
	}

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


	let cachedPage: {url:string;data:any} | null = null;

	async function currentPage() {
		if (!pageUrl) {console.warn('currentPage: pageUrl is null'); return null;}
		if (cachedPage?.url === pageUrl) return cachedPage.data
		try {
			const data = await fetchPageContext(pageUrl);
			cachedPage = { url: pageUrl, data};
			return data;
		} catch (e) {
			console.error('fetch_page_context failed:', e);
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
			await ai.send(text, await currentPage());
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
			await ai.send(parts, await currentPage());
		}
		saveCurrentChat();
	}

	async function sendPrompt(prompt: string) {
		let page = null;
		if (pageUrl) {
			try { page = await fetchPageContext(pageUrl); } catch {}
		}
		await ai.send(prompt,page);
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

	/* ---- message actions ------------------------------------------------ */

	let editingIndex = $state<number | null>(null);
	let editDraft = $state('');

	async function copyMessage(message: ChatMessage) {
		const text = messageText(message);
		if (!text) return;
		try {
			await navigator.clipboard.writeText(text);
			showToast('Copied to clipboard');
		} catch {
			showToast('Could not access the clipboard');
		}
	}

	async function retry(index: number) {
		await ai.regenerate(index, await currentPage());
		saveCurrentChat();
	}

	function startEdit(index: number, message: ChatMessage) {
		editingIndex = index;
		editDraft = messageText(message);
	}

	function cancelEdit() {
		editingIndex = null;
		editDraft = '';
	}

	async function confirmEdit(index: number) {
		const text = editDraft.trim();
		if (!text) return;
		cancelEdit();
		await ai.editAndResend(index, text, await currentPage());
		saveCurrentChat();
	}

	function exportConversation() {
		if (ai.messages.length === 0) return;
		const stamp = new Date().toISOString();
		const body = ai.messages
			.map((m, i) => {
				const who = m.role === 'user' ? 'You' : 'Assistant';
				const variants = ai.alternatives[i];
				let block = `## ${who}\n\n${messageText(m)}\n`;
				// Export every answer that was generated, not just the visible one.
				if (m.role === 'assistant' && variants?.length > 1) {
					const active = ai.activeAlt[i] ?? variants.length - 1;
					block += variants
						.map((v, n) =>
							n === active ? '' : `\n<details><summary>Alternative ${n + 1}</summary>\n\n${v}\n</details>\n`
						)
						.join('');
				}
				return block;
			})
			.join('\n');

		const md = `---\ntitle: star chat export\nexported: ${stamp}\nmessages: ${ai.messages.length}\n---\n\n${body}`;
		const blob = new Blob([md], { type: 'text/markdown' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'star-chat.md';
		document.body.appendChild(a);
		a.click();
		a.remove();
		setTimeout(() => URL.revokeObjectURL(url), 10_000);
		showToast('Saved star-chat.md to your Downloads folder');
	}
</script>

<aside
	class="panel"
	style="width:{panelWidth}px"
	transition:slidePanel
	class:resizing
	class:drag-active={dragActive}
	ondragenter={handleDragEnter}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
>
<div class="resize-handle" role="separator" aria-orientation="vertical" aria-label="Resize panel" onpointerdown={startResize}></div>
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
			<button class="icon" aria-label="AI Memory" type="button" onclick={toggleMemory}>
    <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M9 3a3 3 0 0 0 -3 3v0a3 3 0 0 0 -3 3a3 3 0 0 0 1.5 2.6M9 3a3 3 0 0 1 6 0M9 3v18" />
        <path d="M15 3a3 3 0 0 1 3 3a3 3 0 0 1 3 3a3 3 0 0 1 -1.5 2.6M15 21v-9" />
    </svg>
</button>
			<button
				class="icon"
				aria-label="Export conversation"
				title="Export conversation"
				type="button"
				disabled={ai.messages.length === 0}
				onclick={exportConversation}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 3v12" />
					<path d="M8 11l4 4l4 -4" />
					<path d="M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2 -2v-2" />
				</svg>
			</button>
			<button class="icon" aria-label="Close" type="button" onclick={onclose}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
			</button>
		</div>
	</header>

	{#if showMemory} 
	<div class="history-panel">
		<div class="history-header">
			<h3>AI memory</h3>
			<div class="group">
				<button class="clear-btn" type="button" title="Import" onclick={() => memoryFileEl?.click()}>⤓</button>
                <button class="clear-btn" type="button" title="Export" onclick={exportMemory}>⤒</button>
                <button class="clear-btn" type="button" title="Clear all" onclick={() => memory.clear()}>
                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M10 11v6M14 11v6M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3" /></svg>
                </button>
			</div>
		</div>
		<div class="history-list">
			{#if memory.items.length === 0}
			<p class="empty-history">Nothing here..</p>
			{:else}
			{#each memory.items as m (m.id)}
				<div class="history-item" style="flex-direction: row; align-items: center; justify-content:space-between;">
					<span class="history-title">{m.text}</span>
					<button class="clear-btn" type="button" title="Forget" onclick={() => memory.remove(m.id)}>
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12" /></svg>
                    </button>
				</div>
				{/each}
			{/if}
		</div>
	</div>
	  <div style="display:flex;gap:6px;padding:8px;">
            <input style="flex:1;border:1px solid var(--border-strong);border-radius:8px;padding:6px 8px;font:inherit;font-size:12px;"
                placeholder="Add something to remember…" bind:value={memoryDraft}
                onkeydown={(e) => e.key === 'Enter' && addMemory()} />
            <button class="tool send" type="button" onclick={addMemory} aria-label="Save memory">＋</button>
        </div>
    <input type="file" accept=".md,.markdown,.json,.txt" bind:this={memoryFileEl} onchange={importMemory} style="display:none" />
{/if}
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
				<h1>Hi {username}, let's get into it</h1>
			</div>
			<div class="prompts">
				{#each prompts as prompt (prompt)}
					<button class="chip" type="button" onclick={() => sendPrompt(prompt)}>{prompt}</button>
				{/each}
			</div>
		{:else}
			<ul class="messages">
				{#each ai.messages as message, i (i)}
					<li class="row {message.role}">
						<div class="msg {message.role}">
							{#each messageImages(message) as img (img)}
								<img class="msg-image" src={img} alt="attachment" />
							{/each}

							{#if editingIndex === i}
								<div class="edit-box">
									<textarea
										rows="3"
										bind:value={editDraft}
										aria-label="Edit message"
										onkeydown={(e) => {
											if (e.key === 'Enter' && !e.shiftKey) {
												e.preventDefault();
												confirmEdit(i);
											} else if (e.key === 'Escape') {
												e.preventDefault();
												cancelEdit();
											}
										}}
									></textarea>
									<div class="edit-actions">
										<button type="button" class="mini ghost" onclick={cancelEdit}>Cancel</button>
										<button type="button" class="mini solid" onclick={() => confirmEdit(i)}>
											Send
										</button>
									</div>
								</div>
							{:else if messageText(message)}
								{#if message.role === 'assistant'}
									<!-- Model output: rendered from Markdown, sanitised in renderMarkdown(). -->
									<div class="msg-text md">{@html renderMarkdown(messageText(message))}</div>
								{:else}
									<p class="msg-text">{messageText(message)}</p>
								{/if}
							{/if}
						</div>

						{#if editingIndex !== i && messageText(message)}
							<div class="actions">
								<button
									type="button"
									class="act"
									title="Copy"
									aria-label="Copy message"
									onclick={() => copyMessage(message)}
								>
									<svg viewBox="0 0 24 24" aria-hidden="true">
										<rect x="9" y="9" width="11" height="11" rx="2" />
										<path d="M5 15V5a2 2 0 0 1 2-2h10" />
									</svg>
								</button>

								{#if message.role === 'assistant'}
									<button
										type="button"
										class="act"
										title="Try again"
										aria-label="Try again"
										disabled={ai.sending}
										onclick={() => retry(i)}
									>
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<path d="M20 11a8.1 8.1 0 0 0-15.5-2m-.5-5v5h5" />
											<path d="M4 13a8.1 8.1 0 0 0 15.5 2m.5 5v-5h-5" />
										</svg>
									</button>

									{#if (ai.alternatives[i]?.length ?? 0) > 1}
										{@const list = ai.alternatives[i]}
										{@const active = ai.activeAlt[i] ?? list.length - 1}
										<span class="variants" aria-label="Switch between answers">
											<button
												type="button"
												class="act tiny"
												aria-label="Previous answer"
												disabled={active === 0}
												onclick={() => ai.selectAlternative(i, active - 1)}
											>
												<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 6l-6 6l6 6" /></svg>
											</button>
											<span class="count">{active + 1}/{list.length}</span>
											<button
												type="button"
												class="act tiny"
												aria-label="Next answer"
												disabled={active === list.length - 1}
												onclick={() => ai.selectAlternative(i, active + 1)}
											>
												<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M9 6l6 6l-6 6" /></svg>
											</button>
										</span>
									{/if}
								{:else}
									<button
										type="button"
										class="act"
										title="Edit"
										aria-label="Edit message"
										disabled={ai.sending}
										onclick={() => startEdit(i, message)}
									>
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<path d="M4 20h4l10.5-10.5a2.828 2.828 0 1 0-4-4L4 16v4" />
										</svg>
									</button>
								{/if}
							</div>
						{/if}
					</li>
				{/each}
				{#if ai.sending}
					<li class="row assistant">
						<div class="msg assistant thinking">
							<div class="loading-wrapper">
								<Loading size={24} showText={true} />
							</div>
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
		<button class="tool" aria-label="Attach File" type="button" onclick={() => fileInputEl?.click()}>
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
		</button>
		<textarea
			rows="1"
			placeholder="Ask star"
			bind:value={draft}
			onkeydown={handleKeydown}
			onpaste={handlePaste}
		></textarea>
		<span class="model" title="Requests used today">{ai.used}/{ai.limit}</span>
		<button class="tool send" aria-label="Send" type="button" onclick={submit}>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
			</svg>
		</button>
	</div>

	<input
		type="file"
		multiple
		accept="image/*,video/*,.txt,.json"
		bind:this={fileInputEl}
		onchange={handleFileSelect}
		style="display: none"
	/>

	{#if toast}
		<div class="toast" role="status">{toast}</div>
	{/if}
	{#if ai.lastMemoryNote}
		<div class="toast subtle" role="status">{ai.lastMemoryNote}</div>
	{/if}
</aside>

<style>

	.toast {
		margin: 0 8px 8px;
		padding: 8px 12px;
		border-radius: 10px;
		background: var(--text, #4a3a2e);
		color: var(--accent-contrast);
		font-size: 12px;
		line-height: 1.35;
	}
	.toast.subtle {
		background: var(--field, #f7f1ec);
		color: var(--text-soft, #8a6b57);
	}
	.panel {
		position: relative;
		flex: 0 0 auto;
		display: flex;
		flex-direction: column;
		margin: 0 8px 8px 0;
		border: 1px solid var(--hover);
		border-radius: 16px;
		background: var(--bg-page, #fff);
		box-shadow: 0 8px 32px var(--hover);
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

	.resize-handle {
		position: absolute;
		top: 0;
		left: -5px;
		width: 12px;
		height: 100%;
		z-index: 5;
		cursor: ew-resize;
		touch-action: none;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	/* Always-visible grip pill: makes it obvious the panel can be dragged wider or
	   narrower. It grows and picks up the accent colour on hover and while dragging. */
	.resize-handle::before {
		content: '';
		width: 4px;
		height: 34px;
		border-radius: 999px;
		background: var(--text-muted, #ac8064);
		opacity: 0.3;
		transition:
			opacity 0.16s ease,
			height 0.16s ease,
			background-color 0.16s ease;
	}

	.resize-handle:hover::before {
		height: 56px;
		opacity: 0.8;
		background: var(--accent, #80A4D4);
	}

	.panel.resizing .resize-handle::before {
		height: 72px;
		opacity: 1;
		background: var(--accent, #80A4D4);
	}
	.panel.resizing {
		user-select: none;
	}

	.panel.drag-active {
		border-color: var(--accent, #80A4D4);
		background: rgba(128, 164, 212, 0.02);
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
	/* Circular header controls, as in the reference. */
	.icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 50%;
		background: var(--field);
		color: var(--text-soft, #8a6b57);
		cursor: pointer;
		transition: background-color 150ms ease-in-out, color 150ms ease-in-out;
	}

	.icon:disabled {
		opacity: 0.45;
		cursor: default;
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

	/* Gemini-style centred greeting: gradient spark above, light large heading. */
	.hero {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		min-height: 180px;
		text-align: center;
	}

	.spark {
		width: 34px;
		height: 34px;
		margin-bottom: 14px;
	}

	.hero h1 {
		margin: 0;
		font-size: 25px;
		font-weight: 500;
		line-height: 1.25;
		letter-spacing: -0.01em;
		color: var(--text, #4a3a2e);
		text-align: center;
		text-wrap: balance;
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
		border: 1px solid var(--hover);
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

	/* Each row stacks the bubble and its action bar, and decides which side they sit on. */
	.row {
		display: flex;
		flex-direction: column;
		max-width: 100%;
		min-width: 0;
	}
	.row.user {
		align-items: flex-end;
	}
	.row.assistant {
		align-items: flex-start;
	}

	.msg {
		max-width: 85%;
		min-width: 0;
		padding: 8px 16px;
		border-radius: 16px;
		font-size: 14px;
		line-height: 1.4;
		word-break: break-word;
	}
	.msg.user {
		background: var(--accent, #80A4D4);
		color: var(--accent-contrast);
	}
	.msg.assistant {
		background: var(--field, #f7f1ec);
		color: var(--text, #4a3a2e);
	}
	.msg-text {
		margin: 0;
		white-space: pre-wrap;
		word-break: break-word;
	}

	/* Markdown output must not keep the pre-wrap used for plain user text. */
	.msg-text.md {
		white-space: normal;
	}

	/* ---- rendered Markdown ----
	   Injected via {@html}, so Svelte's scoping does not reach it: these need :global. */

	.msg-text.md :global(> *:first-child) {
		margin-top: 0;
	}
	.msg-text.md :global(> *:last-child) {
		margin-bottom: 0;
	}
	.msg-text.md :global(p) {
		margin: 0 0 8px;
	}
	.msg-text.md :global(h1),
	.msg-text.md :global(h2),
	.msg-text.md :global(h3),
	.msg-text.md :global(h4),
	.msg-text.md :global(h5),
	.msg-text.md :global(h6) {
		margin: 14px 0 6px;
		font-weight: 600;
		line-height: 1.25;
	}
	.msg-text.md :global(h1) {
		font-size: 18px;
	}
	.msg-text.md :global(h2) {
		font-size: 16px;
	}
	.msg-text.md :global(h3) {
		font-size: 15px;
	}
	.msg-text.md :global(h4),
	.msg-text.md :global(h5),
	.msg-text.md :global(h6) {
		font-size: 14px;
	}
	.msg-text.md :global(ul),
	.msg-text.md :global(ol) {
		margin: 0 0 8px;
		padding-left: 20px;
	}
	.msg-text.md :global(li) {
		margin: 2px 0;
	}
	.msg-text.md :global(li > p) {
		margin: 0;
	}
	.msg-text.md :global(a) {
		color: var(--accent, #80A4D4);
		text-decoration: underline;
		text-underline-offset: 2px;
	}
	.msg-text.md :global(strong) {
		font-weight: 650;
	}
	.msg-text.md :global(code) {
		padding: 1.5px 5px;
		border-radius: 5px;
		background: var(--hover);
		font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
		font-size: 0.88em;
	}
	.msg-text.md :global(pre) {
		margin: 0 0 8px;
		padding: 10px 12px;
		border-radius: 10px;
		background: var(--hover);
		/* Long lines scroll inside the block instead of stretching the panel. */
		overflow-x: auto;
	}
	.msg-text.md :global(pre code) {
		display: block;
		padding: 0;
		background: none;
		font-size: 12.5px;
		line-height: 1.5;
		white-space: pre;
	}
	.msg-text.md :global(blockquote) {
		margin: 0 0 8px;
		padding: 2px 0 2px 10px;
		border-left: 3px solid var(--border-strong);
		color: var(--text-soft, #8a6b57);
	}
	.msg-text.md :global(hr) {
		margin: 12px 0;
		border: 0;
		border-top: 1px solid var(--border-strong);
	}
	.msg-text.md :global(table) {
		display: block;
		width: 100%;
		margin: 0 0 8px;
		border-collapse: collapse;
		overflow-x: auto;
		font-size: 13px;
	}
	.msg-text.md :global(th),
	.msg-text.md :global(td) {
		padding: 5px 9px;
		border: 1px solid var(--border-strong);
		text-align: left;
	}
	.msg-text.md :global(th) {
		background: var(--hover);
		font-weight: 600;
	}
	.msg-text.md :global(img) {
		max-width: 100%;
		border-radius: 8px;
	}

	/* ---- per-message actions ---- */

	.actions {
		display: flex;
		align-items: center;
		gap: 2px;
		margin: 2px 4px 0;
		opacity: 0;
		transition: opacity 0.14s ease;
	}
	.row:hover .actions,
	.actions:focus-within {
		opacity: 1;
	}

	.act {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		padding: 0;
		border: 0;
		border-radius: 7px;
		background: transparent;
		color: var(--text-muted, #ac8064);
		cursor: pointer;
		transition: background-color 0.14s ease, color 0.14s ease;
	}
	.act:hover:not(:disabled) {
		background: var(--field, #f7f1ec);
		color: var(--text, #4a3a2e);
	}
	.act:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.act svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.9;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
	.act.tiny {
		width: 20px;
		height: 20px;
	}
	.act.tiny svg {
		width: 12px;
		height: 12px;
	}

	.variants {
		display: inline-flex;
		align-items: center;
		gap: 1px;
		margin-left: 2px;
	}
	.count {
		font-size: 11px;
		color: var(--text-muted, #ac8064);
		font-variant-numeric: tabular-nums;
	}

	/* ---- inline edit ---- */

	.edit-box {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 220px;
	}
	.edit-box textarea {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--border-strong);
		border-radius: 10px;
		background: var(--bg-page, #fff);
		color: var(--text, #4a3a2e);
		font: inherit;
		font-size: 13px;
		resize: vertical;
	}
	.edit-box textarea:focus {
		outline: 2px solid var(--accent, #80A4D4);
		outline-offset: -1px;
	}
	.edit-actions {
		display: flex;
		justify-content: flex-end;
		gap: 6px;
	}
	.mini {
		padding: 5px 12px;
		border: 0;
		border-radius: 999px;
		font: inherit;
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
	}
	.mini.ghost {
		background: var(--field, #f7f1ec);
		color: var(--text-soft, #8a6b57);
	}
	.mini.solid {
		background: var(--accent, #80A4D4);
		color: var(--accent-contrast);
	}

	@media (prefers-reduced-motion: reduce) {
		.actions,
		.act {
			transition: none;
		}
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
		border-bottom: 1px solid var(--hover);
		background: var(--field, #f7f1ec);
		max-height: 200px;
		overflow: hidden;
	}

	.history-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--hover);
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
		background: var(--hover);
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
		border-bottom: 1px solid var(--hover);
	}

	.history-item:hover {
		background: var(--hover);
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
		border-bottom: 1px solid var(--hover);
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
		border: 1px solid var(--hover);
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
		background: var(--accent, #80A4D4);
		color: var(--accent-contrast);
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

	/* Single rounded pill: attach on the left, input in the middle, send on the right. */
	.composer {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 10px;
		padding: 5px 5px 5px 6px;
		border: 1px solid var(--border);
		border-radius: 999px;
		background: var(--field);
		transition: border-color 0.2s ease, background-color 0.2s ease;
	}

	.composer:focus-within {
		border-color: var(--accent, #80A4D4);
	}

	.composer.drag-over {
		border-color: var(--accent, #80A4D4);
		background: var(--field-strong, var(--field));
	}
	textarea {
		flex: 1;
		min-width: 0;
		max-height: 120px;
		padding: 8px 2px;
		border: 0;
		background: transparent;
		color: var(--text, #4a3a2e);
		font: inherit;
		font-size: 14px;
		line-height: 1.35;
		resize: none;
		box-sizing: border-box;
	}
	textarea::placeholder {
		color: var(--text-muted);
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
		flex: 0 0 auto;
		width: 34px;
		height: 34px;
		border: 0;
		border-radius: 50%;
		background: transparent;
		color: var(--text-soft, #8a6b57);
		cursor: pointer;
		transition: background-color 150ms ease-in-out;
	}
	.tool:hover {
		background: var(--hover);
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
		background: var(--accent, #80A4D4);
		color: var(--accent-contrast);
	}

	.tool.send:hover {
		background: var(--accent-hover, #6B8FC4);
	}

	.model {
		flex: 0 0 auto;
		font-size: 11px;
		color: var(--text-muted, #ac8064);
		font-variant-numeric: tabular-nums;
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