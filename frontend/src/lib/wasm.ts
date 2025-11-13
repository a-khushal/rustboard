import type { EditorApi } from './wasm-types';

let wasmModule: any = null;
let editorApi: EditorApi | null = null;

export async function initWasm(): Promise<EditorApi> {
	if (editorApi) {
		return editorApi;
	}

	try {
		// @ts-ignore
		wasmModule = await import('rustboard_wasm');
		await wasmModule.default();
		editorApi = new wasmModule.EditorApi();
		return editorApi as EditorApi;
	} catch (error) {
		console.error('Failed to load Wasm module:', error);
		console.error('Make sure to run: npm run build:wasm first');
		throw error;
	}
}

export function getEditorApi(): EditorApi | null {
	return editorApi;
}

