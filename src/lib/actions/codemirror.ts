import { EditorView } from '@codemirror/view';
import { Compartment, type Extension } from '@codemirror/state';
import type { Action } from 'svelte/action';

export interface CodemirrorOptions {
	doc?: string;
	extensions?: Extension[];
	readonly?: boolean;
}

const readOnlyCompartment = new Compartment();

export const codemirror: Action<HTMLElement, CodemirrorOptions> = (node, options = {}) => {
	const { doc = '', extensions = [], readonly = false } = options;

	const view = new EditorView({
		doc,
		extensions: [...extensions, readOnlyCompartment.of(EditorView.editable.of(!readonly))],
		parent: node,
	});

	return {
		update(newOptions = {}) {
			const effects: ReturnType<typeof readOnlyCompartment.reconfigure>[] = [];

			if (newOptions.readonly !== undefined) {
				effects.push(readOnlyCompartment.reconfigure(EditorView.editable.of(!newOptions.readonly)));
			}

			if (newOptions.doc !== undefined && newOptions.doc !== view.state.doc.toString()) {
				view.dispatch({
					changes: { from: 0, to: view.state.doc.length, insert: newOptions.doc },
					effects,
				});
			} else if (effects.length > 0) {
				view.dispatch({ effects });
			}
		},
		destroy() {
			view.destroy();
		},
	};
};
