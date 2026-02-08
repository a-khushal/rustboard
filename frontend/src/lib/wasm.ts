import type { EditorApi } from './wasm/pkg/rustboard_wasm';

type RustboardWasmModule = typeof import('./wasm/pkg/rustboard_wasm');

let wasmModule: RustboardWasmModule | null = null;
let editorApi: EditorApi | null = null;

export async function initWasm(): Promise<EditorApi> {
	if (editorApi) {
		return editorApi;
	}

	try {
		// @ts-expect-error -- resolved by bundler at runtime; no type declarations available
		const dynamicModule = await import('rustboard_wasm');
		wasmModule = dynamicModule as RustboardWasmModule;
		await wasmModule.default();
		editorApi = new wasmModule.EditorApi();
		return editorApi!;
	} catch (error) {
		console.error('Failed to load Wasm module:', error);
		console.error('Make sure to run: npm run build:wasm first');
		throw error;
	}
}

export function getEditorApi(): EditorApi | null {
	return editorApi;
}
