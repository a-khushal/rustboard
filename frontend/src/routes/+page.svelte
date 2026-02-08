<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { initWasm } from '$lib/wasm';
	import { wasmLoaded, editorApi, rectangles, ellipses, lines, arrows, diamonds, groups, zoom, viewportOffset, images, paths, texts } from '$lib/stores/editor';
	import { loadStateFromLocalStorage, saveStateToLocalStorage, loadZoomFromLocalStorage, saveZoomToLocalStorage, loadViewportOffsetFromLocalStorage, saveViewportOffsetToLocalStorage } from '$lib/utils/storage';
	import { ensureDefaultBoard } from '$lib/utils/boards';
	import { centerViewportOnShapes } from '$lib/utils/center-viewport';
	import { initSelectionHistory, resetSelectionHistory, disposeSelectionHistory } from '$lib/utils/selection-history';
	import { collaborationState } from '$lib/stores/collaboration';
	import { requestFullSync } from '$lib/utils/collaboration';
	import Canvas from '$lib/components/Canvas.svelte';

	let unsubscribeRectangles: (() => void) | null = null;
	let unsubscribeEllipses: (() => void) | null = null;
	let unsubscribeLines: (() => void) | null = null;
	let unsubscribeArrows: (() => void) | null = null;
	let unsubscribeDiamonds: (() => void) | null = null;
	let unsubscribeTexts: (() => void) | null = null;
	let unsubscribeImages: (() => void) | null = null;
	let unsubscribePaths: (() => void) | null = null;
	let unsubscribeGroups: (() => void) | null = null;
	let unsubscribeZoom: (() => void) | null = null;
	let unsubscribeViewportOffset: (() => void) | null = null;
	let saveStateTimer: ReturnType<typeof setTimeout> | null = null;

	onMount(async () => {
		try {
			initSelectionHistory();
			const api = await initWasm();
			editorApi.set(api);
			wasmLoaded.set(true);
		ensureDefaultBoard();
			
		const loaded = loadStateFromLocalStorage();
		if (!loaded) {
			rectangles.set(api.get_rectangles());
			ellipses.set(api.get_ellipses());
			lines.set(api.get_lines());
			arrows.set(api.get_arrows());
			diamonds.set(api.get_diamonds());
			groups.set(api.get_groups());
			texts.set(api.get_texts());
			images.set(api.get_images());
			paths.set(api.get_paths());
		} else {
			groups.set(api.get_groups());
			texts.set(api.get_texts());
			images.set(api.get_images());
			paths.set(api.get_paths());
		}
		resetSelectionHistory();

		loadZoomFromLocalStorage();
		const hasSavedOffset = loadViewportOffsetFromLocalStorage();

		if (!hasSavedOffset) {
			setTimeout(() => {
				centerViewportOnShapes();
			}, 0);
		}

			const saveState = () => {
				if (saveStateTimer) {
					clearTimeout(saveStateTimer);
				}
				saveStateTimer = setTimeout(() => {
					saveStateToLocalStorage();
					saveStateTimer = null;
				}, 200);
			};
			unsubscribeRectangles = rectangles.subscribe(saveState);
			unsubscribeEllipses = ellipses.subscribe(saveState);
			unsubscribeLines = lines.subscribe(saveState);
			unsubscribeArrows = arrows.subscribe(saveState);
			unsubscribeDiamonds = diamonds.subscribe(saveState);
			unsubscribeTexts = texts.subscribe(saveState);
			unsubscribeImages = images.subscribe(saveState);
			unsubscribePaths = paths.subscribe(saveState);
			unsubscribeGroups = groups.subscribe(saveState);

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
		if (unsubscribeTexts) unsubscribeTexts();
		if (unsubscribeImages) unsubscribeImages();
		if (unsubscribePaths) unsubscribePaths();
		if (unsubscribeGroups) unsubscribeGroups();
		if (saveStateTimer) {
			clearTimeout(saveStateTimer);
			saveStateTimer = null;
		}
		if (unsubscribeZoom) unsubscribeZoom();
		if (unsubscribeViewportOffset) unsubscribeViewportOffset();
	});
</script>

<div class="fixed inset-0 bg-stone-50">
	{#if $collaborationState.connectionStatus === 'reconnecting' || $collaborationState.isResyncing || $collaborationState.connectionStatus === 'error'}
		<div class="absolute top-14 left-1/2 z-50 -translate-x-1/2 rounded-md border border-stone-300 bg-white/95 px-3 py-1.5 text-xs text-stone-700 shadow-sm">
			{#if $collaborationState.connectionStatus === 'error'}
				<span>Collaboration issue: {$collaborationState.lastError ?? 'Connection interrupted'}</span>
				<button
					type="button"
					on:click={requestFullSync}
					class="ml-2 rounded border border-stone-300 bg-white px-2 py-0.5 text-[11px] text-stone-700 hover:bg-stone-50"
				>
					Resync now
				</button>
			{:else if $collaborationState.isResyncing}
				Resyncing board state...
			{:else}
				Reconnecting to collaboration session...
			{/if}
		</div>
	{/if}
	{#if $wasmLoaded}	
		<Canvas />
	{:else}
		<div class="flex items-center justify-center h-full bg-stone-50">
			<p class="text-stone-600 text-sm font-sans">Loading...</p>
		</div>
	{/if}
</div>
