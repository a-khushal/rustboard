<script lang="ts">
	import { onMount } from 'svelte';
	import { initWasm } from '$lib/wasm';

	let wasmLoaded = false;
	let greeting = '';
	let sum = 0;

	onMount(async () => {
		try {
			const editorApi = await initWasm();
			wasmLoaded = true;
			greeting = editorApi.greet('Rustboard');
			sum = editorApi.add(42, 13);
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
	});
</script>

<div class="p-8 max-w-3xl mx-auto">
	<h1 class="text-3xl font-bold mb-6">Rustboard - Graphite.rs Style Boilerplate</h1>
	
	{#if wasmLoaded}
		<div class="mt-8 p-6 bg-sky-50 rounded-lg border border-sky-200">
			<p class="text-green-600 mb-4">✅ Wasm module loaded successfully!</p>
			<p class="text-2xl font-bold text-sky-700 my-4">{greeting}</p>
			<p class="text-gray-700">Rust calculation: 42 + 13 = {sum}</p>
		</div>
	{:else}
		<p class="text-gray-600">Loading Wasm module...</p>
	{/if}
</div>

