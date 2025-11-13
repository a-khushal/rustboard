import { writable } from 'svelte/store';
// @ts-expect-error
import type { EditorApi } from '../../../pkg/rustboard_wasm';

export interface Rectangle {
	id: number;
	position: { x: number; y: number };
	width: number;
	height: number;
}

export const wasmLoaded = writable<boolean>(false);
export const editorApi = writable<EditorApi | null>(null);
export const rectangles = writable<Rectangle[]>([]);
export const selectedRectangles = writable<Rectangle[]>([]);
