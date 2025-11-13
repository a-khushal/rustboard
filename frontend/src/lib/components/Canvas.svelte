<script lang="ts">
	import { onMount } from 'svelte';
	import { rectangles, selectedRectangles, editorApi, type Rectangle } from '$lib/stores/editor';
	import { isPointInRectangle } from '$lib/utils/geometry';
	import { renderRectangles } from '$lib/utils/rendering';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragOffset = { x: 0, y: 0 };
	let draggedRectangle: any = null;

	function handleKeyDown(event: KeyboardEvent) {
		if (!$editorApi || $selectedRectangles.length === 0) return;

		if (event.key === 'Delete' || event.key === 'Backspace') {
			event.preventDefault();
			$selectedRectangles.forEach(rect => {
				$editorApi.delete_rectangle(BigInt(rect.id));
			});
			selectedRectangles.set([]);
			updateRectangles();
			render();
		}
	}

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		canvas.focus();

		const rect = canvas.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;

		const isShiftPressed = event.shiftKey;

		for (let i = $rectangles.length - 1; i >= 0; i--) {
			if (isPointInRectangle(x, y, $rectangles[i])) {
				const clickedRect = $rectangles[i];
				const index = $selectedRectangles.findIndex(r => r.id === clickedRect.id);
				
				if (isShiftPressed) {
					if (index >= 0) {
						selectedRectangles.set($selectedRectangles.filter(r => r.id !== clickedRect.id));
					} else {
						selectedRectangles.set([...$selectedRectangles, clickedRect]);
					}
				} else {
					if (index >= 0) {
						selectedRectangles.set($selectedRectangles.filter(r => r.id !== clickedRect.id));
					} else {
						selectedRectangles.set([clickedRect]);
					}
				}

				isDragging = true;
				draggedRectangle = clickedRect;
				dragOffset.x = x - clickedRect.position.x;
				dragOffset.y = y - clickedRect.position.y;

				render();
				return;
			}
		}

		if (!isShiftPressed) {
			selectedRectangles.set([]);
		}
		addRectangle(x, y);
		render();
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;
		if (isDragging && draggedRectangle && $editorApi) {
			const newX = x - dragOffset.x;
			const newY = y - dragOffset.y;
			$editorApi.move_rectangle(BigInt(draggedRectangle.id), newX, newY);
			updateRectangles();
			return;
		}

		if (isDragging) {
			return;
		}

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
	
    function handleMouseUp() {
		isDragging = false;
		draggedRectangle = null;
	}

	function addRectangle(x: number, y: number) {
		if (!$editorApi) return;

		const width = 100;
		const height = 50;
		
		const newId = $editorApi.add_rectangle(x, y, width, height);
		const updatedRectangles = $editorApi.get_rectangles();
		rectangles.set(updatedRectangles);
		
		const newRect = updatedRectangles.find((r: Rectangle) => r.id === Number(newId));
		if (newRect) {
			selectedRectangles.set([newRect]);
		}
	}

	async function updateRectangles() {
		if (!$editorApi) return;
		const updatedRectangles = $editorApi.get_rectangles();
		rectangles.set(updatedRectangles);
		
		if ($selectedRectangles.length > 0) {
			const updatedSelection = $selectedRectangles
				.map(selected => updatedRectangles.find((r: Rectangle) => r.id === selected.id))
				.filter((r): r is Rectangle => r !== undefined);
			selectedRectangles.set(updatedSelection);
		}
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

		window.addEventListener('keydown', handleKeyDown);
		
		return () => {
			window.removeEventListener('keydown', handleKeyDown);
		};
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
	on:mouseup={handleMouseUp}
	on:keydown={handleKeyDown}
	bind:this={canvas}
	class="w-full h-full bg-white"
	tabindex="0"
></canvas>
