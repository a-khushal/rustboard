<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { initWasm } from '$lib/wasm';
	import { wasmLoaded, editorApi, rectangles, ellipses, lines, arrows, diamonds, zoom, viewportOffset } from '$lib/stores/editor';
	import { loadStateFromLocalStorage, saveStateToLocalStorage, loadZoomFromLocalStorage, saveZoomToLocalStorage, loadViewportOffsetFromLocalStorage, saveViewportOffsetToLocalStorage } from '$lib/utils/storage';
	import { centerViewportOnShapes } from '$lib/utils/center-viewport';
	import { initSelectionHistory, resetSelectionHistory, disposeSelectionHistory } from '$lib/utils/selection-history';
	import Canvas from '$lib/components/Canvas.svelte';

	let unsubscribeRectangles: (() => void) | null = null;
	let unsubscribeEllipses: (() => void) | null = null;
	let unsubscribeLines: (() => void) | null = null;
	let unsubscribeArrows: (() => void) | null = null;
	let unsubscribeDiamonds: (() => void) | null = null;
	let unsubscribeZoom: (() => void) | null = null;
	let unsubscribeViewportOffset: (() => void) | null = null;

	onMount(async () => {
		try {
			initSelectionHistory();
			const api = await initWasm();
			editorApi.set(api);
			wasmLoaded.set(true);
			
		const loaded = loadStateFromLocalStorage();
		if (!loaded) {
		rectangles.set(api.get_rectangles());
		ellipses.set(api.get_ellipses());
			lines.set(api.get_lines());
			arrows.set(api.get_arrows());
			diamonds.set(api.get_diamonds());
		}
		resetSelectionHistory();

		loadZoomFromLocalStorage();
		const hasSavedOffset = loadViewportOffsetFromLocalStorage();

		if (!hasSavedOffset) {
			setTimeout(() => {
				centerViewportOnShapes();
			}, 0);
		}

			const saveState = () => saveStateToLocalStorage();
			unsubscribeRectangles = rectangles.subscribe(saveState);
			unsubscribeEllipses = ellipses.subscribe(saveState);
			unsubscribeLines = lines.subscribe(saveState);
			unsubscribeArrows = arrows.subscribe(saveState);
			unsubscribeDiamonds = diamonds.subscribe(saveState);

		unsubscribeZoom = zoom.subscribe(() => {
			saveZoomToLocalStorage();
		});

		unsubscribeViewportOffset = viewportOffset.subscribe(() => {
			saveViewportOffsetToLocalStorage();
		});
	} catch (error) {
		console.error('Failed to initialize Wasm:', error);
	}
});

	onDestroy(() => {
		disposeSelectionHistory();
		if (unsubscribeRectangles) unsubscribeRectangles();
		if (unsubscribeEllipses) unsubscribeEllipses();
		if (unsubscribeLines) unsubscribeLines();
		if (unsubscribeArrows) unsubscribeArrows();
		if (unsubscribeDiamonds) unsubscribeDiamonds();
		if (unsubscribeZoom) unsubscribeZoom();
		if (unsubscribeViewportOffset) unsubscribeViewportOffset();
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
