<script lang="ts">
	import { onMount } from 'svelte';
	import { rectangles, selectedRectangles, editorApi, viewportOffset } from '$lib/stores/editor';
	import { isPointInRectangle } from '$lib/utils/geometry';
	import { renderRectangles } from '$lib/utils/rendering';
	import { screenToWorld } from '$lib/utils/viewport';
	import { addRectangle, deleteRectangles, moveRectangle } from '$lib/utils/canvas-operations/index';
	import { handleViewportScroll } from '$lib/utils/viewport-scroll';
	import Toolbar from './Toolbar.svelte';
	import { activeTool, type Tool } from '$lib/stores/tools';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let dragRectStartPos = { x: 0, y: 0 };
	let draggedRectangle: any = null;
	let justCreatedRectangle = false;
	let isSpacePressed = false;
	let isPanning = false;
	let panStartPos = { x: 0, y: 0 };
	let panStartOffset = { x: 0, y: 0 };

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === ' ') {
			event.preventDefault();
			isSpacePressed = true;
			if (canvas && !isPanning) {
				canvas.style.cursor = 'grab';
			}
			return;
		}

		if (!$editorApi || $selectedRectangles.length === 0 || (event.key !== 'Delete' && event.key !== 'Backspace')) return;

		event.preventDefault();
		const idsToDelete = $selectedRectangles.map(rect => rect.id);
		deleteRectangles(idsToDelete);
		selectedRectangles.set([]);
		render();
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.key === ' ') {
			isSpacePressed = false;
		}
	}

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		event.preventDefault();
		canvas.focus();

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;

		if (isSpacePressed) {
			isPanning = true;
			panStartPos = { x: screenX, y: screenY };
			panStartOffset = { ...$viewportOffset };
			if (canvas) {
				canvas.style.cursor = 'grabbing';
			}
			return;
		}

		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset);

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
		
		const currentTool = String($activeTool).trim();
		
		if (currentTool === 'rectangle') {
			addRectangle(x, y);
			justCreatedRectangle = true;
			activeTool.set('select' as Tool);
		}
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;

		if (isPanning) {
			canvas.style.cursor = 'grabbing';
			const deltaX = screenX - panStartPos.x;
			const deltaY = screenY - panStartPos.y;
			viewportOffset.set({
				x: panStartOffset.x + deltaX,
				y: panStartOffset.y + deltaY,
			});
			return;
		}

		if (isSpacePressed) {
			canvas.style.cursor = 'grab';
			return;
		}

		const currentTool = String($activeTool).trim();
		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset);
		
		if (currentTool === 'rectangle') {
			canvas.style.cursor = 'crosshair';
		} else if (currentTool === 'select') {
			if (isDragging && draggedRectangle) {
				canvas.style.cursor = 'move';
			} else {
				let hoveringOverShape = false;
				
				if ($rectangles.length > 0) {
					for (let i = $rectangles.length - 1; i >= 0; i--) {
						if (isPointInRectangle(x, y, $rectangles[i])) {
							hoveringOverShape = true;
							break;
						}
					}
				}
				
				canvas.style.cursor = hoveringOverShape ? 'move' : 'default';
			}
		} else {
			canvas.style.cursor = 'default';
		}

		if (justCreatedRectangle || !draggedRectangle || !$editorApi) return;
		
		const dx = Math.abs(x - dragStartPos.x);
		const dy = Math.abs(y - dragStartPos.y);
		
		if (!isDragging && (dx > 5 || dy > 5)) {
			isDragging = true;
		}
		
		if (isDragging) {
			const deltaX = x - dragStartPos.x;
			const deltaY = y - dragStartPos.y;
			moveRectangle(
				draggedRectangle.id,
				dragRectStartPos.x + deltaX,
				dragRectStartPos.y + deltaY
			);
		}
	}
	
	function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
		}
		
		isDragging = false;
		draggedRectangle = null;
		setTimeout(() => { justCreatedRectangle = false; }, 100);
		canvas?.focus();
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
		window.addEventListener('keyup', handleKeyUp);
		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('keyup', handleKeyUp);
		};
	});

	$: if (canvas && $editorApi && !ctx) initCanvas();
	$: if (ctx && canvas) {
		$viewportOffset;
		$rectangles;
		$selectedRectangles;
		render();
	}
</script>

<div class="relative w-full h-full bg-stone-50">
	<Toolbar />
	<canvas
		on:mousedown={handleMouseDown}
		on:mousemove={handleMouseMove}
		on:mouseup={handleMouseUp}
		on:wheel={handleViewportScroll}
		on:keydown={handleKeyDown}
		bind:this={canvas}
		class="w-full h-full bg-stone-50"
		tabindex="0"
	></canvas>
</div>
