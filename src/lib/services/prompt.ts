import type { ContentPart, PageContext } from './ai';

export function contentToText(content: string | ContentPart[]): string {
    if (typeof content === 'string') return content;
    return content
        .filter((p) => p.type === 'text')
        .map((p) => (p as { text: string }).text)
        .join('\n');
}

export function escapeAttr(value: string): string {
    return value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');
}

/** Keeps page text from closing its own <page_content> fence and posing as instructions. */
export function fenceText(text: string): string {
    return text.replace(/<\s*\/?\s*page_content/gi, (tag) => tag.replace(/page_content/i, 'page-content'));
}

export function pageBlock(page: PageContext): string {
    const media = [
        page.images.length ? `Images on the page:\n${page.images.join('\n')}` : '',
        page.videos.length ? `Videos on the page:\n${page.videos.join('\n')}` : ''
    ].filter(Boolean).join('\n\n');
    const body = [fenceText(page.text.trim()) || '(the page has no readable text)', fenceText(media)].filter(Boolean).join('\n\n');
    const truncated = page.truncated ? ' truncated="true"' : '';
    return `[The page I am currently viewing in the browser]\n<page_content url="${escapeAttr(page.url)}" title="${escapeAttr(page.title)}"${truncated}>\n${body}\n</page_content>`;
}

export function attachPage(content: string | ContentPart[], page: PageContext): string | ContentPart[] {
    const block = pageBlock(page);
    if (typeof content === 'string') return `${block}\n\n[My message]\n${content}`;
    return [{ type: 'text', text: block }, ...content];
}

/** Text-only models reject image_url parts, so replace them with a note the model can relay. */
export function stripImages(content: string | ContentPart[], visionModel?: string): string | ContentPart[] {
    if (typeof content === 'string') return content;
    const images = content.filter((p) => p.type === 'image_url').length;
    if (!images) return content;
    const text = contentToText(content);
    const note = `[The user attached ${images} image${images === 1 ? '' : 's'}, but the selected model cannot view images. Tell them to switch to ${visionModel ?? 'a model that supports photos'} to analyse images.]`;
    return text ? `${text}\n\n${note}` : note;
}
