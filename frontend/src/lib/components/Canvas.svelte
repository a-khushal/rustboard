<script lang="ts">
	import { onMount } from 'svelte';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse 
	} from '$lib/stores/editor';
	import { isPointInRectangle, isPointInEllipse } from '$lib/utils/geometry';
	import { renderRectangles, renderEllipses } from '$lib/utils/rendering';
	import { screenToWorld } from '$lib/utils/viewport';
	import { 
		addRectangle, deleteRectangles, moveRectangle,
		addEllipse, deleteEllipses, moveEllipse
	} from '$lib/utils/canvas-operations/index';
	import { handleViewportScroll } from '$lib/utils/viewport-scroll';
	import Toolbar from './Toolbar.svelte';
	import ZoomControls from './ZoomControls.svelte';
	import { activeTool, type Tool } from '$lib/stores/tools';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let dragShapeStartPos = { x: 0, y: 0 };
	let draggedShape: Rectangle | Ellipse | null = null;
	let draggedShapeType: 'rectangle' | 'ellipse' | null = null;
	let justCreatedShape = false;
	let isSpacePressed = false;
	let isPanning = false;
	let panStartPos = { x: 0, y: 0 };
	let panStartOffset = { x: 0, y: 0 };
	let isCreatingShape = false;
	let createStartPos = { x: 0, y: 0 };
	let createCurrentPos = { x: 0, y: 0 };
	let isShiftPressedDuringCreation = false;
	let renderRequestId: number | null = null;

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === ' ') {
			event.preventDefault();
			isSpacePressed = true;
			if (canvas && !isPanning) {
				canvas.style.cursor = 'grab';
			}
			return;
		}

		if (event.key === 'Escape' && isCreatingShape) {
			event.preventDefault();
			isCreatingShape = false;
			createStartPos = { x: 0, y: 0 };
			createCurrentPos = { x: 0, y: 0 };
			scheduleRender();
			return;
		}

		if (!$editorApi || (event.key !== 'Delete' && event.key !== 'Backspace')) return;

		const hasSelectedRectangles = $selectedRectangles.length > 0;
		const hasSelectedEllipses = $selectedEllipses.length > 0;

		if (!hasSelectedRectangles && !hasSelectedEllipses) return;

		event.preventDefault();
		
		if (hasSelectedRectangles) {
			const idsToDelete = $selectedRectangles.map(rect => rect.id);
			deleteRectangles(idsToDelete);
			selectedRectangles.set([]);
		}
		
		if (hasSelectedEllipses) {
			const idsToDelete = $selectedEllipses.map(ellipse => ellipse.id);
			deleteEllipses(idsToDelete);
			selectedEllipses.set([]);
		}
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.key === ' ') {
			isSpacePressed = false;
		}
	}

	function handleShapeClick(
		shape: Rectangle | Ellipse,
		shapeType: 'rectangle' | 'ellipse',
		isShiftPressed: boolean,
		x: number,
		y: number
	) {
		if (shapeType === 'rectangle') {
			const clickedRect = shape as Rectangle;
			const index = $selectedRectangles.findIndex(r => r.id === clickedRect.id);
			
			if (isShiftPressed) {
				selectedRectangles.set(
					index >= 0
						? $selectedRectangles.filter(r => r.id !== clickedRect.id)
						: [...$selectedRectangles, clickedRect]
				);
			} else {
				selectedRectangles.set([clickedRect]);
				selectedEllipses.set([]);
			}

			draggedShape = clickedRect;
			draggedShapeType = 'rectangle';
			dragStartPos = { x, y };
			dragShapeStartPos = { x: clickedRect.position.x, y: clickedRect.position.y };
		} else {
			const clickedEllipse = shape as Ellipse;
			const index = $selectedEllipses.findIndex(e => e.id === clickedEllipse.id);
			
			if (isShiftPressed) {
				selectedEllipses.set(
					index >= 0
						? $selectedEllipses.filter(e => e.id !== clickedEllipse.id)
						: [...$selectedEllipses, clickedEllipse]
				);
			} else {
				selectedEllipses.set([clickedEllipse]);
				selectedRectangles.set([]);
			}

			draggedShape = clickedEllipse;
			draggedShapeType = 'ellipse';
			dragStartPos = { x, y };
			dragShapeStartPos = { x: clickedEllipse.position.x, y: clickedEllipse.position.y };
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
			canvas.style.cursor = 'grabbing';
			return;
		}

		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset, $zoom);

		justCreatedShape = false;
		const isShiftPressed = event.shiftKey;
		const currentTool = String($activeTool).trim();
		
		if (currentTool === 'select') {
			for (let i = $rectangles.length - 1; i >= 0; i--) {
				if (isPointInRectangle(x, y, $rectangles[i])) {
					handleShapeClick($rectangles[i], 'rectangle', isShiftPressed, x, y);
					return;
				}
			}

			for (let i = $ellipses.length - 1; i >= 0; i--) {
				if (isPointInEllipse(x, y, $ellipses[i])) {
					handleShapeClick($ellipses[i], 'ellipse', isShiftPressed, x, y);
					return;
				}
			}

			if (isShiftPressed) return;

			selectedRectangles.set([]);
			selectedEllipses.set([]);
		} else if (currentTool === 'rectangle' || currentTool === 'ellipse') {
			selectedRectangles.set([]);
			selectedEllipses.set([]);
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			createStartPos = { x, y };
			createCurrentPos = { x, y };
			scheduleRender();
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
		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset, $zoom);
		
		if (currentTool === 'rectangle' || currentTool === 'ellipse') {
			canvas.style.cursor = 'crosshair';
			if (isCreatingShape) {
				isShiftPressedDuringCreation = event.shiftKey;
				createCurrentPos = { x, y };
				scheduleRender();
			}
		} else if (currentTool === 'select') {
			if (isDragging && draggedShape) {
				canvas.style.cursor = 'move';
			} else {
				let hoveringOverShape = false;
				for (let i = $rectangles.length - 1; i >= 0; i--) {
					if (isPointInRectangle(x, y, $rectangles[i])) {
						hoveringOverShape = true;
						break;
					}
				}
				if (!hoveringOverShape) {
					for (let i = $ellipses.length - 1; i >= 0; i--) {
						if (isPointInEllipse(x, y, $ellipses[i])) {
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

		if (justCreatedShape || !draggedShape || !$editorApi) return;
		
		const dx = Math.abs(x - dragStartPos.x);
		const dy = Math.abs(y - dragStartPos.y);
		
		if (!isDragging && (dx > 5 || dy > 5)) {
			isDragging = true;
		}
		
		if (isDragging) {
			const deltaX = x - dragStartPos.x;
			const deltaY = y - dragStartPos.y;
			const newX = dragShapeStartPos.x + deltaX;
			const newY = dragShapeStartPos.y + deltaY;
			
			if (draggedShapeType === 'rectangle') {
				moveRectangle((draggedShape as Rectangle).id, newX, newY);
			} else if (draggedShapeType === 'ellipse') {
				moveEllipse((draggedShape as Ellipse).id, newX, newY);
			}
		}
	}
	
	function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
		}
		
		if (isCreatingShape) {
			const currentTool = String($activeTool).trim();
			const threshold = currentTool === 'ellipse' ? 10 : 5;
			
			if (currentTool === 'rectangle') {
				const width = Math.abs(createCurrentPos.x - createStartPos.x);
				const height = Math.abs(createCurrentPos.y - createStartPos.y);
				
				if (width > threshold && height > threshold) {
					const x = Math.min(createStartPos.x, createCurrentPos.x);
					const y = Math.min(createStartPos.y, createCurrentPos.y);
					addRectangle(x, y, width, height);
					justCreatedShape = true;
					activeTool.set('select' as Tool);
				}
			} else if (currentTool === 'ellipse') {
				const minX = Math.min(createStartPos.x, createCurrentPos.x);
				const maxX = Math.max(createStartPos.x, createCurrentPos.x);
				const minY = Math.min(createStartPos.y, createCurrentPos.y);
				const maxY = Math.max(createStartPos.y, createCurrentPos.y);

				const width = maxX - minX;
				const height = maxY - minY;

				const centerX = minX + width / 2;
				const centerY = minY + height / 2;
				
				let radius_x = width / 2;
				let radius_y = height / 2;

				if (isShiftPressedDuringCreation) {
					const maxRadius = Math.max(radius_x, radius_y);
					radius_x = maxRadius;
					radius_y = maxRadius;
				}

				if (radius_x > threshold && radius_y > threshold) {
					addEllipse(centerX, centerY, radius_x, radius_y);
					justCreatedShape = true;
					activeTool.set('select' as Tool);
				}
			}
			
			isCreatingShape = false;
			createStartPos = { x: 0, y: 0 };
			createCurrentPos = { x: 0, y: 0 };
			scheduleRender();
		}
		
		isDragging = false;
		draggedShape = null;
		draggedShapeType = null;
		setTimeout(() => { justCreatedShape = false; }, 100);
		canvas?.focus();
	}

	function render() {
		if (!ctx || !canvas) return;
		const renderCtx = ctx;
		
		renderCtx.clearRect(0, 0, canvas.width, canvas.height);
		renderCtx.fillStyle = '#fafaf9';
		renderCtx.fillRect(0, 0, canvas.width, canvas.height);
		
		const previewRect = isCreatingShape ? {
			x: Math.min(createStartPos.x, createCurrentPos.x),
			y: Math.min(createStartPos.y, createCurrentPos.y),
			width: Math.abs(createCurrentPos.x - createStartPos.x),
			height: Math.abs(createCurrentPos.y - createStartPos.y)
		} : null;
		
		const currentTool = String($activeTool).trim();
		const isCreatingRectangle = currentTool === 'rectangle' && isCreatingShape;
		const isCreatingEllipse = currentTool === 'ellipse' && isCreatingShape;
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$rectangles.forEach((rect) => {
			const isSelected = $selectedRectangles.some(selected => selected.id === rect.id);
			renderCtx.strokeStyle = isSelected ? '#ef4444' : '#000000';
			renderCtx.lineWidth = 2 / $zoom;
			renderCtx.strokeRect(rect.position.x, rect.position.y, rect.width, rect.height);
		});
		
		if (isCreatingRectangle && previewRect && previewRect.width > 0 && previewRect.height > 0) {
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2 / $zoom;
			renderCtx.globalAlpha = 0.5;
			renderCtx.strokeRect(previewRect.x, previewRect.y, previewRect.width, previewRect.height);
			renderCtx.globalAlpha = 1.0;
		}
		
		renderCtx.restore();
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$ellipses.forEach((ellipse) => {
			const isSelected = $selectedEllipses.some(selected => selected.id === ellipse.id);
			renderCtx.strokeStyle = isSelected ? '#ef4444' : '#000000';
			renderCtx.lineWidth = 2 / $zoom;
			renderCtx.beginPath();
			renderCtx.ellipse(ellipse.position.x, ellipse.position.y, ellipse.radius_x, ellipse.radius_y, 0, 0, 2 * Math.PI);
			renderCtx.stroke();
		});
		
		if (isCreatingEllipse) {
			const minX = Math.min(createStartPos.x, createCurrentPos.x);
			const maxX = Math.max(createStartPos.x, createCurrentPos.x);
			const minY = Math.min(createStartPos.y, createCurrentPos.y);
			const maxY = Math.max(createStartPos.y, createCurrentPos.y);

			const width = maxX - minX;
			const height = maxY - minY;

			const centerX = minX + width / 2;
			const centerY = minY + height / 2;
			
			let radius_x = width / 2;
			let radius_y = height / 2;
			
			if (isShiftPressedDuringCreation) {
				const maxRadius = Math.max(radius_x, radius_y);
				radius_x = maxRadius;
				radius_y = maxRadius;
			}
			
			if (radius_x > 0 && radius_y > 0) {
				renderCtx.strokeStyle = '#000000';
				renderCtx.lineWidth = 2 / $zoom;
				renderCtx.globalAlpha = 0.5;
				renderCtx.beginPath();
				renderCtx.ellipse(centerX, centerY, radius_x, radius_y, 0, 0, 2 * Math.PI);
				renderCtx.stroke();
				renderCtx.globalAlpha = 1.0;
			}
		}
		
		renderCtx.restore();
	}

	function scheduleRender() {
		if (renderRequestId !== null) {
			cancelAnimationFrame(renderRequestId);
		}
		renderRequestId = requestAnimationFrame(() => {
			render();
			renderRequestId = null;
		});
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
		$zoom;
		$rectangles;
		$selectedRectangles;
		$ellipses;
		$selectedEllipses;
		if (!isCreatingShape) {
			scheduleRender();
		}
	}
</script>

<div class="relative w-full h-full bg-stone-50">
	<Toolbar />
	<ZoomControls />
	<canvas
		on:mousedown={handleMouseDown}
		on:mousemove={handleMouseMove}
		on:mouseup={handleMouseUp}
		on:wheel={(e) => handleViewportScroll(e, canvas!)}
		on:keydown={handleKeyDown}
		bind:this={canvas}
		class="w-full h-full bg-stone-50"
		tabindex="0"
	></canvas>
</div>
