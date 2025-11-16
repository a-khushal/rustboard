<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { initWasm } from '$lib/wasm';
	import { wasmLoaded, editorApi, rectangles, ellipses } from '$lib/stores/editor';
	import { loadStateFromLocalStorage, saveStateToLocalStorage } from '$lib/utils/storage';
	import Canvas from '$lib/components/Canvas.svelte';

	let unsubscribeRectangles: (() => void) | null = null;
	let unsubscribeEllipses: (() => void) | null = null;

	onMount(async () => {
		try {
			const api = await initWasm();
			editorApi.set(api);
			wasmLoaded.set(true);
			
			const loaded = loadStateFromLocalStorage();
			if (!loaded) {
				rectangles.set(api.get_rectangles());
				ellipses.set(api.get_ellipses());
			}

			unsubscribeRectangles = rectangles.subscribe(() => {
				saveStateToLocalStorage();
			});

			unsubscribeEllipses = ellipses.subscribe(() => {
				saveStateToLocalStorage();
			});
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
	});

	onDestroy(() => {
		if (unsubscribeRectangles) unsubscribeRectangles();
		if (unsubscribeEllipses) unsubscribeEllipses();
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
