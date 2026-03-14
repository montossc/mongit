import { keymap } from '@codemirror/view';
import { defaultKeymap } from '@codemirror/commands';
import { defaultHighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { javascript } from '@codemirror/lang-javascript';
import { rust } from '@codemirror/lang-rust';
import type { Extension } from '@codemirror/state';

/** Base extensions shared across all CM6 instances */
export function baseExtensions(): Extension[] {
	return [keymap.of(defaultKeymap), syntaxHighlighting(defaultHighlightStyle)];
}

/** Language extension by file name */
export function languageExtension(filename: string): Extension[] {
	const ext = filename.split('.').pop()?.toLowerCase();
	switch (ext) {
		case 'js':
		case 'jsx':
		case 'mjs':
			return [javascript()];
		case 'ts':
		case 'tsx':
		case 'mts':
			return [javascript({ typescript: true })];
		case 'rs':
			return [rust()];
		default:
			return [];
	}
}
