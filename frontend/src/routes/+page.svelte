<script lang="ts">
	import { onMount } from 'svelte';
	import { initWasm } from '$lib/wasm';
	import { wasmLoaded, editorApi, rectangles } from '$lib/stores/editor';
	import Canvas from '$lib/components/Canvas.svelte';

	onMount(async () => {
		try {
			const api = await initWasm();
			editorApi.set(api);
			wasmLoaded.set(true);
			rectangles.set(api.get_rectangles());
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
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
