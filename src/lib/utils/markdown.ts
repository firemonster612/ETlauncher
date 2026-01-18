import DOMPurify from 'dompurify';
import { marked } from 'marked';

/**
 * Renders markdown content to sanitized HTML.
 * Handles both markdown and raw HTML input safely.
 */
export function renderMarkdown(body?: string | null, fallback?: string): string {
	const source = (body && body.trim()) || fallback || '';
	if (!source) return '';

	const trimmed = source.trim();
	const looksLikeHtml = /^</.test(trimmed) && /<\/?[a-z][\s\S]*>/i.test(trimmed);

	try {
		const html = looksLikeHtml ? trimmed : (marked.parse(trimmed) as string);
		return DOMPurify.sanitize(html, {
			ALLOWED_TAGS: [
				'h1',
				'h2',
				'h3',
				'h4',
				'h5',
				'h6',
				'p',
				'br',
				'hr',
				'ul',
				'ol',
				'li',
				'blockquote',
				'pre',
				'code',
				'a',
				'strong',
				'em',
				'del',
				's',
				'img',
				'table',
				'thead',
				'tbody',
				'tr',
				'th',
				'td',
				'details',
				'summary',
				'span',
				'div',
			],
			ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'target', 'rel', 'class'],
			ALLOW_DATA_ATTR: false,
			ADD_ATTR: ['target'],
			FORBID_TAGS: ['script', 'style', 'iframe', 'form', 'input', 'button', 'object', 'embed'],
			FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover', 'onfocus', 'onblur'],
		});
	} catch (e) {
		console.error('Failed to render description', e);
		return DOMPurify.sanitize(trimmed.replace(/\n/g, '<br/>'));
	}
}
