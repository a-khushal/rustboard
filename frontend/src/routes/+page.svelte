<script lang="ts">
	import { onMount } from 'svelte';
	import { initWasm } from '$lib/wasm';

	let wasmLoaded = false;
	let editorApi: any = null;
	let rectangleCount = 0;

	onMount(async () => {
		try {
			editorApi = await initWasm();
			wasmLoaded = true;
			rectangleCount = editorApi.get_rectangles_count();
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
	});

	function addRectangle() {
		if (!editorApi) return;

		const x = Math.random() * 500;
		const y = Math.random() * 500;
		const width = 100;
		const height = 50;
		
		const id = editorApi.add_rectangle(x, y, width, height);
		rectangleCount = editorApi.get_rectangles_count();
		console.log(`Added rectangle with id: ${id}`);
	}
</script>

<div class="p-8 max-w-3xl mx-auto">
	<h1 class="text-3xl font-bold mb-6">Rustboard</h1>
	
	{#if wasmLoaded}
		<div class="mt-8 p-6 bg-sky-50 rounded-lg border border-sky-200">
			<p class="text-green-600 mb-4">Wasm module loaded successfully!</p>
			<p class="text-gray-700 mb-4">Rectangles: {rectangleCount}</p>
			<button 
				on:click={addRectangle}
				class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
			>
				Add Rectangle
			</button>
		</div>
	{:else}
		<p class="text-gray-600">Loading Wasm module...</p>
	{/if}
</div>