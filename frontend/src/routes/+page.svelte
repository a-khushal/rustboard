<script lang="ts">
	import { onMount } from 'svelte';
	import { initWasm } from '$lib/wasm';

	let wasmLoaded = false;
	let editorApi: any = null;
	let rectangles: any[] = [];
	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;

	onMount(async () => {
		try {
			editorApi = await initWasm();
			wasmLoaded = true;
			await updateRectangles();
		} catch (error) {
			console.error('Failed to initialize Wasm:', error);
		}
	});

	$: if (canvas && wasmLoaded && !ctx) {
		ctx = canvas.getContext('2d');
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		render();
	}

	$: if (rectangles.length > 0 && ctx && canvas) {
		render();
	}

	async function updateRectangles() {
		if (!editorApi) return;
		rectangles = editorApi.get_rectangles();
	}

	function addRectangle() {
		if (!editorApi || !canvas) return;

		const x = Math.random() * (canvas.width - 100);
		const y = Math.random() * (canvas.height - 50);
		const width = 100;
		const height = 50;
		
		editorApi.add_rectangle(x, y, width, height);
		updateRectangles();
	}

	function render() {
		if (!ctx || !canvas) return;

		ctx.clearRect(0, 0, canvas.width, canvas.height);
		ctx.fillStyle = '#ffffff';
		ctx.fillRect(0, 0, canvas.width, canvas.height);

		rectangles.forEach((rect) => {
			ctx!.fillStyle = '#3b82f6';
			ctx!.strokeStyle = '#1e40af';
			ctx!.lineWidth = 2;
			
			ctx!.fillRect(
				rect.position.x,
				rect.position.y,
				rect.width,
				rect.height
			);
			
			ctx!.strokeRect(
				rect.position.x,
				rect.position.y,
				rect.width,
				rect.height
			);
		});
	}
</script>

<div class="fixed inset-0">
	{#if wasmLoaded}
		<button 
			on:click={addRectangle}
			style="position: absolute; top: 10px; left: 10px; padding: 8px 16px; border: 1px solid black; z-index: 10; color: black;"
		>
			Add Rectangle
		</button>
		
		<canvas
			bind:this={canvas}
			class="w-full h-full bg-white"
		></canvas>
	{:else}
		<div class="flex items-center justify-center h-full bg-white">
			<p class="text-black">Loading...</p>
		</div>
	{/if}
</div>
