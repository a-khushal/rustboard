<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { initWasm } from '$lib/wasm';
	import { wasmLoaded, editorApi, rectangles, ellipses, lines, arrows } from '$lib/stores/editor';
	import { loadStateFromLocalStorage, saveStateToLocalStorage } from '$lib/utils/storage';
	import { centerViewportOnShapes } from '$lib/utils/center-viewport';
	import Canvas from '$lib/components/Canvas.svelte';

	let unsubscribeRectangles: (() => void) | null = null;
	let unsubscribeEllipses: (() => void) | null = null;
	let unsubscribeLines: (() => void) | null = null;
	let unsubscribeArrows: (() => void) | null = null;

	onMount(async () => {
		try {
			const api = await initWasm();
			editorApi.set(api);
			wasmLoaded.set(true);
			
			const loaded = loadStateFromLocalStorage();
			if (!loaded) {
			rectangles.set(api.get_rectangles());
			ellipses.set(api.get_ellipses());
				lines.set((api as any).get_lines());
				arrows.set((api as any).get_arrows());
			}

			setTimeout(() => {
				centerViewportOnShapes();
			}, 0);

			unsubscribeRectangles = rectangles.subscribe(() => {
				saveStateToLocalStorage();
			});

			unsubscribeEllipses = ellipses.subscribe(() => {
				saveStateToLocalStorage();
			});

			unsubscribeLines = lines.subscribe(() => {
				saveStateToLocalStorage();
			});

			unsubscribeArrows = arrows.subscribe(() => {
				saveStateToLocalStorage();
			});
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
	});

	onDestroy(() => {
		if (unsubscribeRectangles) unsubscribeRectangles();
		if (unsubscribeEllipses) unsubscribeEllipses();
		if (unsubscribeLines) unsubscribeLines();
		if (unsubscribeArrows) unsubscribeArrows();
	});
</script>

<div class="fixed inset-0 bg-stone-50">
	{#if $wasmLoaded}	
		<Canvas />
	{:else}
		<div class="flex items-center justify-center h-full bg-stone-50">
			<p class="text-stone-600 text-sm font-sans">Loading...</p>
		</div>
	{/if}
</div>
