<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse 
	} from '$lib/stores/editor';
	import { isPointInRectangle, isPointInEllipse } from '$lib/utils/geometry';
	import { renderRectangles, renderEllipses } from '$lib/utils/rendering';
	import { screenToWorld } from '$lib/utils/viewport';
	import { 
		addRectangle, deleteRectangles, moveRectangle, resizeRectangle,
		addEllipse, deleteEllipses, moveEllipse, resizeEllipse
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
	let isResizing = false;
	let resizeHandleIndex: number | null = null;
	let resizeStartShape: Rectangle | Ellipse | null = null;
	let resizeStartShapeType: 'rectangle' | 'ellipse' | null = null;
	let resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
	let resizeStartMousePos = { x: 0, y: 0 };
	
	const resizeCursors = ['nwse-resize', 'nesw-resize', 'nwse-resize', 'nesw-resize'];

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

	function getResizeHandleAt(x: number, y: number, shape: Rectangle | Ellipse, shapeType: 'rectangle' | 'ellipse', zoom: number): number | null {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const gap = 4 / zoom;
		
		let boxX: number, boxY: number, boxWidth: number, boxHeight: number;
		
		if (shapeType === 'rectangle') {
			const rect = shape as Rectangle;
			boxX = rect.position.x - gap;
			boxY = rect.position.y - gap;
			boxWidth = rect.width + gap * 2;
			boxHeight = rect.height + gap * 2;
		} else {
			const ellipse = shape as Ellipse;
			boxX = ellipse.position.x - ellipse.radius_x - gap;
			boxY = ellipse.position.y - ellipse.radius_y - gap;
			boxWidth = (ellipse.radius_x * 2) + gap * 2;
			boxHeight = (ellipse.radius_y * 2) + gap * 2;
		}
		
		const corners = [
			{ x: boxX, y: boxY },
			{ x: boxX + boxWidth, y: boxY },
			{ x: boxX + boxWidth, y: boxY + boxHeight },
			{ x: boxX, y: boxY + boxHeight }
		];
		
		for (let i = 0; i < corners.length; i++) {
			const corner = corners[i];
			if (x >= corner.x - halfHandle && x <= corner.x + halfHandle &&
				y >= corner.y - halfHandle && y <= corner.y + halfHandle) {
				return i;
			}
		}
		
		return null;
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
			for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
				const handleIndex = getResizeHandleAt(x, y, $selectedRectangles[i], 'rectangle', $zoom);
				if (handleIndex !== null) {
					isResizing = true;
					resizeHandleIndex = handleIndex;
					resizeStartShape = $selectedRectangles[i];
					resizeStartShapeType = 'rectangle';
					resizeStartPos = {
						x: $selectedRectangles[i].position.x,
						y: $selectedRectangles[i].position.y,
						width: $selectedRectangles[i].width,
						height: $selectedRectangles[i].height
					};
					resizeStartMousePos = { x, y };
					return;
				}
			}

			for (let i = $selectedEllipses.length - 1; i >= 0; i--) {
				const handleIndex = getResizeHandleAt(x, y, $selectedEllipses[i], 'ellipse', $zoom);
				if (handleIndex !== null) {
					isResizing = true;
					resizeHandleIndex = handleIndex;
					resizeStartShape = $selectedEllipses[i];
					resizeStartShapeType = 'ellipse';
					const ellipse = $selectedEllipses[i];
					resizeStartPos = {
						x: ellipse.position.x - ellipse.radius_x,
						y: ellipse.position.y - ellipse.radius_y,
						width: ellipse.radius_x * 2,
						height: ellipse.radius_y * 2
					};
					resizeStartMousePos = { x, y };
					return;
				}
			}

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
		
		if (isResizing && resizeStartShape && resizeHandleIndex !== null && $editorApi) {
			const deltaX = x - resizeStartMousePos.x;
			const deltaY = y - resizeStartMousePos.y;
			
			if (resizeStartShapeType === 'rectangle') {
				const rect = resizeStartShape as Rectangle;
				let newX = resizeStartPos.x;
				let newY = resizeStartPos.y;
				let newWidth = resizeStartPos.width;
				let newHeight = resizeStartPos.height;
				
				if (resizeHandleIndex === 0) {
					newX = resizeStartPos.x + deltaX;
					newY = resizeStartPos.y + deltaY;
					newWidth = resizeStartPos.width - deltaX;
					newHeight = resizeStartPos.height - deltaY;
				} else if (resizeHandleIndex === 1) {
					newY = resizeStartPos.y + deltaY;
					newWidth = resizeStartPos.width + deltaX;
					newHeight = resizeStartPos.height - deltaY;
				} else if (resizeHandleIndex === 2) {
					newWidth = resizeStartPos.width + deltaX;
					newHeight = resizeStartPos.height + deltaY;
				} else if (resizeHandleIndex === 3) {
					newX = resizeStartPos.x + deltaX;
					newWidth = resizeStartPos.width - deltaX;
					newHeight = resizeStartPos.height + deltaY;
				}
				
				if (newWidth > 0 && newHeight > 0) {
					moveRectangle(rect.id, newX, newY);
					resizeRectangle(rect.id, newWidth, newHeight);
				}
			} else if (resizeStartShapeType === 'ellipse') {
				const ellipse = resizeStartShape as Ellipse;
				let newX = resizeStartPos.x;
				let newY = resizeStartPos.y;
				let newWidth = resizeStartPos.width;
				let newHeight = resizeStartPos.height;
				
				if (resizeHandleIndex === 0) {
					newX = resizeStartPos.x + deltaX;
					newY = resizeStartPos.y + deltaY;
					newWidth = resizeStartPos.width - deltaX;
					newHeight = resizeStartPos.height - deltaY;
				} else if (resizeHandleIndex === 1) {
					newY = resizeStartPos.y + deltaY;
					newWidth = resizeStartPos.width + deltaX;
					newHeight = resizeStartPos.height - deltaY;
				} else if (resizeHandleIndex === 2) {
					newWidth = resizeStartPos.width + deltaX;
					newHeight = resizeStartPos.height + deltaY;
				} else if (resizeHandleIndex === 3) {
					newX = resizeStartPos.x + deltaX;
					newWidth = resizeStartPos.width - deltaX;
					newHeight = resizeStartPos.height + deltaY;
				}
				
				if (newWidth > 0 && newHeight > 0) {
					const centerX = newX + newWidth / 2;
					const centerY = newY + newHeight / 2;
					const radiusX = newWidth / 2;
					const radiusY = newHeight / 2;
					
					moveEllipse(ellipse.id, centerX, centerY);
					resizeEllipse(ellipse.id, radiusX, radiusY);
				}
			}
			return;
		}
		
		if (currentTool === 'rectangle' || currentTool === 'ellipse') {
			canvas.style.cursor = 'crosshair';
			if (isCreatingShape) {
				isShiftPressedDuringCreation = event.shiftKey;
				createCurrentPos = { x, y };
				scheduleRender();
			}
		} else if (currentTool === 'select') {
			if (isResizing) {
				canvas.style.cursor = resizeHandleIndex !== null ? resizeCursors[resizeHandleIndex] : 'default';
			} else if (isDragging && draggedShape) {
				canvas.style.cursor = 'move';
			} else {
				let hoveringOverHandle = false;
				for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
					const handleIndex = getResizeHandleAt(x, y, $selectedRectangles[i], 'rectangle', $zoom);
					if (handleIndex !== null) {
						canvas.style.cursor = resizeCursors[handleIndex];
						hoveringOverHandle = true;
						break;
					}
				}
				if (!hoveringOverHandle) {
					for (let i = $selectedEllipses.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(x, y, $selectedEllipses[i], 'ellipse', $zoom);
						if (handleIndex !== null) {
							canvas.style.cursor = resizeCursors[handleIndex];
							hoveringOverHandle = true;
							break;
						}
					}
				}
				if (!hoveringOverHandle) {
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
	
	async function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
		}
		
		if (isResizing) {
			isResizing = false;
			resizeHandleIndex = null;
			resizeStartShape = null;
			resizeStartShapeType = null;
			resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
			resizeStartMousePos = { x: 0, y: 0 };
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
					selectedEllipses.set([]);
					await tick();
					const updatedRectangles = get(rectangles);
					if (updatedRectangles.length > 0) {
						const newestRect = updatedRectangles[updatedRectangles.length - 1];
						selectedRectangles.set([newestRect]);
					}
					await tick();
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
					selectedRectangles.set([]);
					await tick();
					const updatedEllipses = get(ellipses);
					if (updatedEllipses.length > 0) {
						const newestEllipse = updatedEllipses[updatedEllipses.length - 1];
						selectedEllipses.set([newestEllipse]);
					}
					await tick();
					justCreatedShape = true;
					activeTool.set('select' as Tool);
				}
			}
			
			isCreatingShape = false;
			createStartPos = { x: 0, y: 0 };
			createCurrentPos = { x: 0, y: 0 };
			await tick();
			scheduleRender();
		}
		
		isDragging = false;
		draggedShape = null;
		draggedShapeType = null;
		setTimeout(() => { justCreatedShape = false; }, 100);
		canvas?.focus();
	}

	function renderResizeHandles(ctx: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, zoom: number) {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const gap = 4 / zoom;
		
		const boxX = x - gap;
		const boxY = y - gap;
		const boxWidth = width + gap * 2;
		const boxHeight = height + gap * 2;
		
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 1 / zoom;
		ctx.beginPath();
		ctx.rect(boxX, boxY, boxWidth, boxHeight);
		ctx.stroke();
		
		ctx.fillStyle = '#ffffff';
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 2 / zoom;
		
		const corners = [
			{ x: boxX, y: boxY },
			{ x: boxX + boxWidth, y: boxY },
			{ x: boxX + boxWidth, y: boxY + boxHeight },
			{ x: boxX, y: boxY + boxHeight }
		];
		
		corners.forEach((corner) => {
			ctx.beginPath();
			ctx.rect(corner.x - halfHandle, corner.y - halfHandle, handleSize, handleSize);
			ctx.fill();
			ctx.stroke();
		});
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
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2 / $zoom;
			renderCtx.strokeRect(rect.position.x, rect.position.y, rect.width, rect.height);
			
			if (isSelected) {
				renderResizeHandles(renderCtx, rect.position.x, rect.position.y, rect.width, rect.height, $zoom);
			}
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
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2 / $zoom;
			renderCtx.beginPath();
			renderCtx.ellipse(ellipse.position.x, ellipse.position.y, ellipse.radius_x, ellipse.radius_y, 0, 0, 2 * Math.PI);
			renderCtx.stroke();
			
			if (isSelected) {
				const x = ellipse.position.x - ellipse.radius_x;
				const y = ellipse.position.y - ellipse.radius_y;
				const width = ellipse.radius_x * 2;
				const height = ellipse.radius_y * 2;
				renderResizeHandles(renderCtx, x, y, width, height, $zoom);
			}
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
