<script lang="ts">
	import { onMount } from 'svelte';
	import { rectangles, selectedRectangles, editorApi } from '$lib/stores/editor';
	import { isPointInRectangle } from '$lib/utils/geometry';
	import { renderRectangles } from '$lib/utils/rendering';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;

		const isShiftPressed = event.shiftKey;

		for (let i = $rectangles.length - 1; i >= 0; i--) {
			if (isPointInRectangle(x, y, $rectangles[i])) {
				const index = $selectedRectangles.findIndex(r => r.id === $rectangles[i].id);
				if (isShiftPressed) {
					if (index >= 0) {
						selectedRectangles.set($selectedRectangles.filter(r => r.id !== $rectangles[i].id));
					} else {
						selectedRectangles.set([...$selectedRectangles, $rectangles[i]]);
					}
				} else {
					selectedRectangles.set(index >= 0
						? $selectedRectangles.filter(r => r.id !== $rectangles[i].id)
						: [$rectangles[i]]
					);
				}

				render();
				return;
			}
		}

		if (!isShiftPressed) {
			selectedRectangles.set([]);
		}
		addRectangle(x, y);
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;

		for (let i = $rectangles.length - 1; i >= 0; i--) {
			if (isPointInRectangle(x, y, $rectangles[i])) {
				selectedRectangles.set([$rectangles[i]]);
				render();
				return;
			}
		}

		selectedRectangles.set([]);
		render();
	}

	function addRectangle(x: number, y: number) {
		if (!$editorApi) return;

		const width = 100;
		const height = 50;
		
		$editorApi.add_rectangle(x, y, width, height);
		updateRectangles();
	}

	async function updateRectangles() {
		if (!$editorApi) return;
		rectangles.set($editorApi.get_rectangles());
	}

	function render() {
		if (!ctx || !canvas) return;
		renderRectangles(ctx, canvas, $rectangles, $selectedRectangles);
	}

	onMount(() => {
		if (canvas) {
			ctx = canvas.getContext('2d');
			canvas.width = window.innerWidth;
			canvas.height = window.innerHeight;
			render();
		}
	});

	$: if (canvas && $editorApi && !ctx) {
		ctx = canvas.getContext('2d');
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		render();
	}

	$: if ($rectangles.length > 0 && ctx && canvas) {
		render();
	}

	$: if ($selectedRectangles.length > 0 && ctx && canvas) {
		render();
	}
</script>

<canvas
	on:mousedown={handleMouseDown}
	on:mousemove={handleMouseMove}
	bind:this={canvas}
	class="w-full h-full bg-white"
></canvas>

