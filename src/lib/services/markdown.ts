import { marked } from 'marked';
import DOMPurify from 'dompurify';

// Assistant replies arrive as Markdown. They are model output, so the HTML that
// marked produces is always sanitised before it reaches the DOM.
marked.setOptions({ gfm: true, breaks: true });

const ALLOWED_TAGS = [
	'p', 'br', 'hr', 'strong', 'em', 'del', 'code', 'pre', 'blockquote',
	'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
	'ul', 'ol', 'li', 'a', 'img',
	'table', 'thead', 'tbody', 'tr', 'th', 'td'
];

const ALLOWED_ATTR = ['href', 'title', 'alt', 'src', 'start', 'align'];

let hooked = false;

function installHook() {
	if (hooked || typeof window === 'undefined') return;
	hooked = true;
	// Anything clickable must not be able to reach back into this window.
	DOMPurify.addHook('afterSanitizeAttributes', (node) => {
		if (node.tagName === 'A') {
			node.setAttribute('target', '_blank');
			node.setAttribute('rel', 'noopener noreferrer nofollow');
		}
	});
}

/** Render Markdown to sanitised HTML that is safe to inject with {@html}. */
export function renderMarkdown(source: string): string {
	if (!source) return '';
	installHook();
	try {
		const raw = marked.parse(source, { async: false }) as string;
		return DOMPurify.sanitize(raw, {
			ALLOWED_TAGS,
			ALLOWED_ATTR,
			// Block javascript:/data: URLs in links and images.
			ALLOWED_URI_REGEXP: /^(?:https?:|mailto:|tel:|#|\/)/i
		});
	} catch {
		// Fall back to plain text rather than showing nothing.
		return DOMPurify.sanitize(`<p>${source}</p>`, { ALLOWED_TAGS: ['p'], ALLOWED_ATTR: [] });
	}
}
