import { EditorView } from '@codemirror/view';
import type { Extension } from '@codemirror/state';

/**
 * mongit CodeMirror theme.
 * Uses CSS custom properties from app.css design tokens.
 * This ensures CM6 inherits the app's dark theme automatically.
 */
export const mongitTheme: Extension = EditorView.theme({
	'&': {
		backgroundColor: 'var(--color-bg-surface)',
		color: 'var(--color-text-primary)',
		fontFamily: 'var(--font-mono)',
		fontSize: '13px',
		lineHeight: '1.5',
	},
	'.cm-content': {
		caretColor: 'var(--color-accent)',
	},
	'.cm-cursor, .cm-dropCursor': {
		borderLeftColor: 'var(--color-accent)',
	},
	'&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
		backgroundColor: 'var(--color-accent-muted)',
	},
	'.cm-gutters': {
		backgroundColor: 'var(--color-bg)',
		color: 'var(--color-text-muted)',
		borderRight: '1px solid var(--color-border-subtle)',
	},
	'.cm-activeLineGutter': {
		backgroundColor: 'var(--color-bg-hover)',
	},
	'.cm-activeLine': {
		backgroundColor: 'var(--color-bg-hover)',
	},
	/* Merge/diff specific */
	'.cm-changedLine': {
		backgroundColor: 'var(--color-diff-modified-bg)',
	},
	'.cm-changedText': {
		backgroundColor: 'var(--color-diff-modified-bg)',
	},
	'.cm-deletedChunk': {
		backgroundColor: 'var(--color-diff-removed-bg)',
	},
});

/** Read-only theme variant (slightly dimmed) */
export const mongitReadOnlyTheme: Extension = EditorView.theme({
	'&': {
		backgroundColor: 'var(--color-bg)',
		opacity: '0.95',
	},
});
