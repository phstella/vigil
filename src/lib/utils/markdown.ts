/**
 * Lightweight markdown-to-HTML renderer.
 *
 * Converts a subset of markdown syntax to sanitised HTML for live preview.
 * Supports: headings, bold, italic, inline code, code blocks, links,
 * wikilinks ([[...]]), unordered/ordered lists, blockquotes, horizontal
 * rules, and paragraphs.
 *
 * This is intentionally dependency-free to keep bundle size small and
 * toggle latency within the 100ms performance budget.
 */

/**
 * Escape HTML special characters to prevent injection.
 */
function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

/**
 * Render inline markdown elements (bold, italic, code, links, wikilinks).
 */
function renderInline(text: string): string {
	let result = escapeHtml(text);

	// Inline code (backticks) -- must come before bold/italic to avoid conflicts
	result = result.replace(/`([^`]+)`/g, '<code class="md-inline-code">$1</code>');

	// Bold + italic (*** or ___)
	result = result.replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>');
	result = result.replace(/___(.+?)___/g, '<strong><em>$1</em></strong>');

	// Bold (** or __)
	result = result.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
	result = result.replace(/__(.+?)__/g, '<strong>$1</strong>');

	// Italic (* or _)
	result = result.replace(/\*(.+?)\*/g, '<em>$1</em>');
	result = result.replace(/\b_(.+?)_\b/g, '<em>$1</em>');

	// Wikilinks [[target]] or [[target|display]]
	result = result.replace(
		/\[\[([^\]|]+)\|([^\]]+)\]\]/g,
		'<a class="md-wikilink" data-wikilink="$1">$2</a>'
	);
	result = result.replace(
		/\[\[([^\]]+)\]\]/g,
		'<a class="md-wikilink" data-wikilink="$1">$1</a>'
	);

	// Standard markdown links [text](url)
	result = result.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a class="md-link" href="$2">$1</a>');

	// Strikethrough ~~text~~
	result = result.replace(/~~(.+?)~~/g, '<del>$1</del>');

	return result;
}

/**
 * Convert a full markdown string to HTML.
 */
/**
 * Represents a parsed wikilink extracted from markdown content.
 */
export interface WikilinkMatch {
	/** Raw target text inside the brackets (e.g., "other-note" from [[other-note]]). */
	target: string;
	/** Display text if provided via pipe syntax, otherwise same as target. */
	display: string;
	/** Start index of the full `[[...]]` match in the source string. */
	start: number;
	/** End index (exclusive) of the full `[[...]]` match in the source string. */
	end: number;
}

/**
 * Extract all `[[wikilink]]` references from markdown content.
 *
 * Handles `[[target]]` and `[[target|display text]]` syntax.
 * Does not skip code blocks -- caller should handle that if needed.
 */
export function extractWikilinks(content: string): WikilinkMatch[] {
	const matches: WikilinkMatch[] = [];
	const re = /\[\[([^\]\n]+?)\]\]/g;
	let m: RegExpExecArray | null;

	while ((m = re.exec(content)) !== null) {
		const inner = m[1];
		const pipeIdx = inner.indexOf('|');
		const target = pipeIdx >= 0 ? inner.slice(0, pipeIdx).trim() : inner.trim();
		const display = pipeIdx >= 0 ? inner.slice(pipeIdx + 1).trim() : inner.trim();

		if (target) {
			matches.push({
				target,
				display,
				start: m.index,
				end: m.index + m[0].length
			});
		}
	}

	return matches;
}

/**
 * Detect if the cursor is currently inside a `[[` trigger sequence.
 *
 * Returns the partial query text after `[[` if triggered, or null if not.
 * For example, if text is "See [[oth" and cursor is at end, returns "oth".
 */
export function detectWikilinkTrigger(text: string, cursorPos: number): string | null {
	// Look backwards from cursor for `[[` without a closing `]]`
	const before = text.slice(0, cursorPos);
	const lastOpen = before.lastIndexOf('[[');
	if (lastOpen < 0) return null;

	// Check there's no closing ]] between the [[ and cursor
	const afterOpen = before.slice(lastOpen + 2);
	if (afterOpen.includes(']]')) return null;

	// Check no newline between [[ and cursor
	if (afterOpen.includes('\n')) return null;

	return afterOpen;
}

/**
 * Insert a wikilink into text at the cursor position, replacing any partial trigger.
 *
 * Returns the new text and the new cursor position after the inserted link.
 */
export function insertWikilink(
	text: string,
	cursorPos: number,
	target: string
): { text: string; cursor: number } {
	const before = text.slice(0, cursorPos);
	const lastOpen = before.lastIndexOf('[[');
	if (lastOpen < 0) {
		// Fallback: insert at cursor
		const link = `[[${target}]]`;
		return {
			text: text.slice(0, cursorPos) + link + text.slice(cursorPos),
			cursor: cursorPos + link.length
		};
	}

	const link = `[[${target}]]`;
	const after = text.slice(cursorPos);
	return {
		text: text.slice(0, lastOpen) + link + after,
		cursor: lastOpen + link.length
	};
}

/**
 * Convert a full markdown string to HTML.
 */
export function markdownToHtml(markdown: string): string {
	const lines = markdown.split('\n');
	const outputBlocks: string[] = [];
	let i = 0;

	while (i < lines.length) {
		const line = lines[i];

		// Fenced code block (``` or ~~~)
		if (/^(`{3,}|~{3,})(.*)$/.test(line)) {
			const fence = line.match(/^(`{3,}|~{3,})/)?.[0] ?? '```';
			const lang = line.slice(fence.length).trim();
			const codeLines: string[] = [];
			i++;
			while (i < lines.length && !lines[i].startsWith(fence)) {
				codeLines.push(escapeHtml(lines[i]));
				i++;
			}
			i++; // skip closing fence
			const langAttr = lang ? ` data-language="${escapeHtml(lang)}"` : '';
			outputBlocks.push(
				`<pre class="md-code-block"${langAttr}><code>${codeLines.join('\n')}</code></pre>`
			);
			continue;
		}

		// Blank line -- skip
		if (line.trim() === '') {
			i++;
			continue;
		}

		// Horizontal rule
		if (/^(\*{3,}|-{3,}|_{3,})\s*$/.test(line.trim())) {
			outputBlocks.push('<hr class="md-hr" />');
			i++;
			continue;
		}

		// Heading (ATX style)
		const headingMatch = line.match(/^(#{1,6})\s+(.+)$/);
		if (headingMatch) {
			const level = headingMatch[1].length;
			const content = renderInline(headingMatch[2]);
			outputBlocks.push(`<h${level} class="md-h${level}">${content}</h${level}>`);
			i++;
			continue;
		}

		// Blockquote
		if (line.startsWith('>')) {
			const quoteLines: string[] = [];
			while (
				i < lines.length &&
				(lines[i].startsWith('>') || (lines[i].trim() !== '' && quoteLines.length > 0))
			) {
				if (lines[i].startsWith('>')) {
					quoteLines.push(lines[i].replace(/^>\s?/, ''));
				} else {
					break;
				}
				i++;
			}
			const innerHtml = markdownToHtml(quoteLines.join('\n'));
			outputBlocks.push(`<blockquote class="md-blockquote">${innerHtml}</blockquote>`);
			continue;
		}

		// Unordered list
		if (/^[\s]*[-*+]\s+/.test(line)) {
			const items: string[] = [];
			while (i < lines.length && /^[\s]*[-*+]\s+/.test(lines[i])) {
				items.push(renderInline(lines[i].replace(/^[\s]*[-*+]\s+/, '')));
				i++;
			}
			const lis = items.map((item) => `<li>${item}</li>`).join('');
			outputBlocks.push(`<ul class="md-ul">${lis}</ul>`);
			continue;
		}

		// Ordered list
		if (/^[\s]*\d+\.\s+/.test(line)) {
			const items: string[] = [];
			while (i < lines.length && /^[\s]*\d+\.\s+/.test(lines[i])) {
				items.push(renderInline(lines[i].replace(/^[\s]*\d+\.\s+/, '')));
				i++;
			}
			const lis = items.map((item) => `<li>${item}</li>`).join('');
			outputBlocks.push(`<ol class="md-ol">${lis}</ol>`);
			continue;
		}

		// Paragraph -- collect consecutive non-blank, non-special lines
		const paraLines: string[] = [];
		while (
			i < lines.length &&
			lines[i].trim() !== '' &&
			!/^(#{1,6}\s|>|[-*+]\s|\d+\.\s|`{3,}|~{3,}|(\*{3,}|-{3,}|_{3,})\s*$)/.test(lines[i])
		) {
			paraLines.push(lines[i]);
			i++;
		}
		if (paraLines.length > 0) {
			outputBlocks.push(`<p class="md-p">${renderInline(paraLines.join(' '))}</p>`);
		}
	}

	return outputBlocks.join('\n');
}
