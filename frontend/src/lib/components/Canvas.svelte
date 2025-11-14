<script lang="ts">
	import { onMount } from 'svelte';
	import { rectangles, selectedRectangles, editorApi, type Rectangle, viewportOffset } from '$lib/stores/editor';
	import { isPointInRectangle } from '$lib/utils/geometry';
	import { renderRectangles } from '$lib/utils/rendering';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let dragRectStartPos = { x: 0, y: 0 };
	let draggedRectangle: any = null;
	let justCreatedRectangle = false;

	function screenToWorld(screenX: number, screenY: number): { x: number, y: number } {
		const offset = $viewportOffset;
		return {
			x: screenX - offset.x,
			y: screenY - offset.y,
		}
	}

	function handleWheel(event: WheelEvent) {
		event.preventDefault();

		if (event.shiftKey) {
			const horizontalDelta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
			viewportOffset.update(offset => ({
				x: offset.x - horizontalDelta,
				y: offset.y,
			}));
		} else {
			viewportOffset.update(offset => ({
				x: offset.x,
				y: offset.y - event.deltaY,
			}));
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!$editorApi || $selectedRectangles.length === 0 || (event.key !== 'Delete' && event.key !== 'Backspace')) return;

		event.preventDefault();
		$selectedRectangles.forEach(rect => {
			$editorApi.delete_rectangle(BigInt(rect.id));
		});
		selectedRectangles.set([]);
		updateRectangles();
		render();
	}

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		event.preventDefault();
		canvas.focus();

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const { x, y } = screenToWorld(screenX, screenY);

		justCreatedRectangle = false;
		const isShiftPressed = event.shiftKey;

		for (let i = $rectangles.length - 1; i >= 0; i--) {
			if (isPointInRectangle(x, y, $rectangles[i])) {
				const clickedRect = $rectangles[i];
				const index = $selectedRectangles.findIndex(r => r.id === clickedRect.id);
				
				if (isShiftPressed) {
					selectedRectangles.set(
						index >= 0
							? $selectedRectangles.filter(r => r.id !== clickedRect.id)
							: [...$selectedRectangles, clickedRect]
					);
				} else {
					selectedRectangles.set([clickedRect]);
				}

				draggedRectangle = clickedRect;
				dragStartPos = { x, y };
				dragRectStartPos = { x: clickedRect.position.x, y: clickedRect.position.y };
				return;
			}
		}

		if (isShiftPressed) return;

		selectedRectangles.set([]);
		addRectangle(x, y);
		justCreatedRectangle = true;
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas || justCreatedRectangle || !draggedRectangle || !$editorApi) return;

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const { x, y } = screenToWorld(screenX, screenY);
		
		const dx = Math.abs(x - dragStartPos.x);
		const dy = Math.abs(y - dragStartPos.y);
		
		if (!isDragging && (dx > 5 || dy > 5)) {
			isDragging = true;
		}
		
		if (isDragging) {
			const deltaX = x - dragStartPos.x;
			const deltaY = y - dragStartPos.y;
			$editorApi.move_rectangle(
				BigInt(draggedRectangle.id),
				dragRectStartPos.x + deltaX,
				dragRectStartPos.y + deltaY
			);
			updateRectangles();
		}
	}
	
	function handleMouseUp() {
		isDragging = false;
		draggedRectangle = null;
		setTimeout(() => { justCreatedRectangle = false; }, 100);
		canvas?.focus();
	}

	function addRectangle(x: number, y: number) {
		if (!$editorApi) return;

		const width = 100;
		const height = 50;
		
		const newId = $editorApi.add_rectangle(x, y, width, height);
		const updatedRectangles = $editorApi.get_rectangles() as Rectangle[];
		rectangles.set(updatedRectangles);
		
		const newRect = updatedRectangles.find((r: Rectangle) => r.id === Number(newId));
		if (newRect) {
			selectedRectangles.set([newRect]);
		}
	}

	function updateRectangles() {
		if (!$editorApi) return;
		const updatedRectangles = $editorApi.get_rectangles() as Rectangle[];
		rectangles.set(updatedRectangles);
		
		if ($selectedRectangles.length > 0) {
			const selectedIds = new Set($selectedRectangles.map(r => r.id));
			const updatedSelection = updatedRectangles.filter((r: Rectangle) => selectedIds.has(r.id));
			selectedRectangles.set(updatedSelection.length > 0 ? updatedSelection : []);
		}
	}

	function render() {
		if (!ctx || !canvas) return;
		renderRectangles(ctx, canvas, $rectangles, $selectedRectangles, $viewportOffset);
	}

	function initCanvas() {
		if (!canvas) return;
		ctx = canvas.getContext('2d');
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
	}

	onMount(() => {
		initCanvas();
		window.addEventListener('keydown', handleKeyDown);
		return () => window.removeEventListener('keydown', handleKeyDown);
	});

	$: if (canvas && $editorApi && !ctx) initCanvas();
	$: if (ctx && canvas) {
		$viewportOffset;
		$rectangles;
		$selectedRectangles;
		render();
	}
</script>

<canvas
	on:mousedown={handleMouseDown}
	on:mousemove={handleMouseMove}
	on:mouseup={handleMouseUp}
	on:wheel={handleWheel}
	on:keydown={handleKeyDown}
	bind:this={canvas}
	class="w-full h-full bg-white"
	tabindex="0"
></canvas>
