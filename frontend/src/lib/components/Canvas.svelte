<script lang="ts">
	import { onMount } from 'svelte';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		lines, selectedLines, arrows, selectedArrows,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse, type Line, type Arrow 
	} from '$lib/stores/editor';
	import { isPointInRectangle, isPointInEllipse, isPointOnLine } from '$lib/utils/geometry';
	import { screenToWorld } from '$lib/utils/viewport';
	import { 
		addRectangle, deleteRectangles, moveRectangle, resizeRectangle,
		addEllipse, deleteEllipses, moveEllipse, resizeEllipse,
		addLine, deleteLines, moveLine,
		addArrow, deleteArrows, moveArrow
	} from '$lib/utils/canvas-operations/index';
	import { handleViewportScroll } from '$lib/utils/viewport-scroll';
	import { zoomIn, zoomOut } from '$lib/utils/zoom';
	import { copyToClipboard, getClipboard, hasClipboardData } from '$lib/utils/clipboard';
	import Toolbar from './Toolbar.svelte';
	import ZoomControls from './ZoomControls.svelte';
	import UndoRedoControls from './UndoRedoControls.svelte';
	import { activeTool, type Tool } from '$lib/stores/tools';

	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let dragShapeStartPos = { x: 0, y: 0 };
	let draggedShape: Rectangle | Ellipse | Line | Arrow | null = null;
	let draggedShapeType: 'rectangle' | 'ellipse' | 'line' | 'arrow' | null = null;
	let justCreatedShape = false;
	let isSpacePressed = false;
	let isPanning = false;
	let panStartPos = { x: 0, y: 0 };
	let panStartOffset = { x: 0, y: 0 };
	let isCreatingShape = false;
	let createStartPos = { x: 0, y: 0 };
	let createCurrentPos = { x: 0, y: 0 };
	let isShiftPressedDuringCreation = false;
	let lineStart: { x: number; y: number } | null = null;
	let lineEnd: { x: number; y: number } | null = null;
	let arrowStart: { x: number; y: number } | null = null;
	let arrowEnd: { x: number; y: number } | null = null;
	let renderRequestId: number | null = null;
	let isResizing = false;
	let resizeHandleIndex: number | null = null;
	let resizeStartShape: Rectangle | Ellipse | Line | Arrow | null = null;
	let resizeStartShapeType: 'rectangle' | 'ellipse' | 'line' | 'arrow' | null = null;
	let resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
	let resizeStartMousePos = { x: 0, y: 0 };
	let isShiftPressedDuringResize = false;
	let dragOffset = { x: 0, y: 0 };
	let resizePreview: { x: number; y: number; width: number; height: number; type: 'rectangle' | 'ellipse' | 'line' | 'arrow'; id: number } | null = null;
	let lastMouseWorldPos: { x: number; y: number } | null = null;
	
	const resizeCursors = ['nwse-resize', 'nesw-resize', 'nwse-resize', 'nesw-resize'];

	function handleKeyDown(event: KeyboardEvent) {
		if ((event.ctrlKey || event.metaKey) && (event.key === 'z' || event.key === 'Z')) {
			event.preventDefault();
			if ($editorApi) {
				if (event.shiftKey) {
					const success = $editorApi.redo();
					if (success) {
						const updatedRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
						const updatedEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
						rectangles.set(updatedRectangles);
						ellipses.set(updatedEllipses);
					}
				} else {
					const success = $editorApi.undo();
					if (success) {
						const updatedRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
						const updatedEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
						rectangles.set(updatedRectangles);
						ellipses.set(updatedEllipses);
					}
				}
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'y' || event.key === 'Y')) {
			event.preventDefault();
			if ($editorApi) {
				const success = $editorApi.redo();
				if (success) {
					const updatedRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
					const updatedEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
					rectangles.set(updatedRectangles);
					ellipses.set(updatedEllipses);
				}
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === '+' || event.key === '=' || event.key === '-')) {
			event.preventDefault();
			if (event.key === '+' || event.key === '=') {
				zoomIn();
			} else if (event.key === '-') {
				zoomOut();
			}
			return;
		}

		if (event.key === ' ') {
			event.preventDefault();
			isSpacePressed = true;
			if (canvas && !isPanning) {
				canvas.style.cursor = 'grab';
			}
			return;
		}

		if (event.key === 'Escape') {
			event.preventDefault();
			if (isCreatingShape) {
				return;
			}
			if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'line' || $activeTool === 'arrow') {
				activeTool.set('select');
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'c' || event.key === 'C')) {
			event.preventDefault();
			if ($selectedRectangles.length > 0 || $selectedEllipses.length > 0) {
				copyToClipboard($selectedRectangles, $selectedEllipses);
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'v' || event.key === 'V')) {
			event.preventDefault();
			if (!$editorApi || !hasClipboardData()) return;
			
			const clipboard = getClipboard();
			const pastedRects: Rectangle[] = [];
			const pastedEllipses: Ellipse[] = [];

			const pasteOffset = { x: 20, y: 20 };
			let pasteX = pasteOffset.x;
			let pasteY = pasteOffset.y;

			if (lastMouseWorldPos) {
				pasteX = lastMouseWorldPos.x;
				pasteY = lastMouseWorldPos.y;
			} else {
				const viewportCenterX = window.innerWidth / 2;
				const viewportCenterY = window.innerHeight / 2;
				const centerWorld = screenToWorld(viewportCenterX, viewportCenterY, $viewportOffset, $zoom);
				pasteX = centerWorld.x;
				pasteY = centerWorld.y;
			}

			if (clipboard.rectangles.length > 0) {
				const minX = Math.min(...clipboard.rectangles.map(r => r.position.x));
				const minY = Math.min(...clipboard.rectangles.map(r => r.position.y));
				
				clipboard.rectangles.forEach(rect => {
					const offsetX = rect.position.x - minX + pasteX;
					const offsetY = rect.position.y - minY + pasteY;
					const newId = $editorApi.add_rectangle(offsetX, offsetY, rect.width, rect.height);
					const updatedRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
					const newRect = updatedRectangles.find(r => r.id === newId);
					if (newRect) pastedRects.push(newRect);
				});
				rectangles.set(Array.from($editorApi.get_rectangles() as Rectangle[]));
			}

			if (clipboard.ellipses.length > 0) {
				const minX = Math.min(...clipboard.ellipses.map(e => e.position.x));
				const minY = Math.min(...clipboard.ellipses.map(e => e.position.y));
				
				clipboard.ellipses.forEach(ellipse => {
					const offsetX = ellipse.position.x - minX + pasteX;
					const offsetY = ellipse.position.y - minY + pasteY;
					const newId = $editorApi.add_ellipse(offsetX, offsetY, ellipse.radius_x, ellipse.radius_y);
					const updatedEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
					const newEllipse = updatedEllipses.find(e => e.id === newId);
					if (newEllipse) pastedEllipses.push(newEllipse);
				});
				ellipses.set(Array.from($editorApi.get_ellipses() as Ellipse[]));
			}

			if (pastedRects.length > 0 || pastedEllipses.length > 0) {
				selectedRectangles.set(pastedRects);
				selectedEllipses.set(pastedEllipses);
			}
			return;
		}

		if (!$editorApi || (event.key !== 'Delete' && event.key !== 'Backspace')) return;

		const hasSelectedRectangles = $selectedRectangles.length > 0;
		const hasSelectedEllipses = $selectedEllipses.length > 0;
		const hasSelectedLines = $selectedLines.length > 0;
		const hasSelectedArrows = $selectedArrows.length > 0;

		if (!hasSelectedRectangles && !hasSelectedEllipses && !hasSelectedLines && !hasSelectedArrows) return;

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
		
		if (hasSelectedLines) {
			const idsToDelete = $selectedLines.map(line => line.id);
			deleteLines(idsToDelete);
			selectedLines.set([]);
		}
		
		if (hasSelectedArrows) {
			const idsToDelete = $selectedArrows.map(arrow => arrow.id);
			deleteArrows(idsToDelete);
			selectedArrows.set([]);
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

	function getLineResizeHandleAt(x: number, y: number, line: Line, zoom: number): number | null {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		
		const distToStart = Math.sqrt((x - line.start.x) ** 2 + (y - line.start.y) ** 2);
		const distToEnd = Math.sqrt((x - line.end.x) ** 2 + (y - line.end.y) ** 2);
		
		if (distToStart <= halfHandle) {
			return 0;
		} else if (distToEnd <= halfHandle) {
			return 1;
		}
		
		return null;
	}

	function getArrowResizeHandleAt(x: number, y: number, arrow: Arrow, zoom: number): number | null {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		
		const distToStart = Math.sqrt((x - arrow.start.x) ** 2 + (y - arrow.start.y) ** 2);
		const distToEnd = Math.sqrt((x - arrow.end.x) ** 2 + (y - arrow.end.y) ** 2);
		
		if (distToStart <= halfHandle) {
			return 0;
		} else if (distToEnd <= halfHandle) {
			return 1;
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
			dragOffset = { x: 0, y: 0 };
			isDragging = true;
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
			dragOffset = { x: 0, y: 0 };
			isDragging = true;
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
		
		if ($activeTool === 'select') {
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
					isShiftPressedDuringResize = isShiftPressed;
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
					isShiftPressedDuringResize = isShiftPressed;
					return;
				}
			}

			for (let i = $selectedLines.length - 1; i >= 0; i--) {
				const handleIndex = getLineResizeHandleAt(x, y, $selectedLines[i], $zoom);
				if (handleIndex !== null) {
					isResizing = true;
					resizeHandleIndex = handleIndex;
					resizeStartShape = $selectedLines[i];
					resizeStartShapeType = 'line';
					const line = $selectedLines[i];
					resizeStartPos = {
						x: line.start.x,
						y: line.start.y,
						width: line.end.x - line.start.x,
						height: line.end.y - line.start.y
					};
					resizeStartMousePos = { x, y };
					isShiftPressedDuringResize = isShiftPressed;
					return;
				}
			}

			for (let i = $selectedArrows.length - 1; i >= 0; i--) {
				const handleIndex = getArrowResizeHandleAt(x, y, $selectedArrows[i], $zoom);
				if (handleIndex !== null) {
					isResizing = true;
					resizeHandleIndex = handleIndex;
					resizeStartShape = $selectedArrows[i];
					resizeStartShapeType = 'arrow';
					const arrow = $selectedArrows[i];
					resizeStartPos = {
						x: arrow.start.x,
						y: arrow.start.y,
						width: arrow.end.x - arrow.start.x,
						height: arrow.end.y - arrow.start.y
					};
					resizeStartMousePos = { x, y };
					isShiftPressedDuringResize = isShiftPressed;
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

			for (let i = $lines.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $lines[i], 5 / $zoom)) {
					const clickedLine = $lines[i];
					const index = $selectedLines.findIndex(l => l.id === clickedLine.id);
					
					if (isShiftPressed) {
						selectedLines.set(
							index >= 0
								? $selectedLines.filter(l => l.id !== clickedLine.id)
								: [...$selectedLines, clickedLine]
						);
					} else {
						selectedLines.set([clickedLine]);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						selectedArrows.set([]);
					}
					
					draggedShape = clickedLine as any;
					draggedShapeType = 'line' as any;
					dragStartPos = { x, y };
					dragShapeStartPos = { x: clickedLine.start.x, y: clickedLine.start.y };
					dragOffset = { x: 0, y: 0 };
					isDragging = true;
					return;
				}
			}

			for (let i = $arrows.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $arrows[i] as any, 5 / $zoom)) {
					const clickedArrow = $arrows[i];
					const index = $selectedArrows.findIndex(a => a.id === clickedArrow.id);
					
					if (isShiftPressed) {
						selectedArrows.set(
							index >= 0
								? $selectedArrows.filter(a => a.id !== clickedArrow.id)
								: [...$selectedArrows, clickedArrow]
						);
					} else {
						selectedArrows.set([clickedArrow]);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						selectedLines.set([]);
					}
					
					draggedShape = clickedArrow as any;
					draggedShapeType = 'arrow' as any;
					dragStartPos = { x, y };
					dragShapeStartPos = { x: clickedArrow.start.x, y: clickedArrow.start.y };
					dragOffset = { x: 0, y: 0 };
					isDragging = true;
					return;
				}
			}

			if (isShiftPressed) return;

			selectedRectangles.set([]);
			selectedEllipses.set([]);
			selectedLines.set([]);
			selectedArrows.set([]);
		} else if ($activeTool === 'rectangle' || $activeTool === 'ellipse') {
			selectedRectangles.set([]);
			selectedEllipses.set([]);
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			createStartPos = { x, y };
			createCurrentPos = { x, y };
			scheduleRender();
		} else if ($activeTool === 'line') {
			selectedRectangles.set([]);
			selectedEllipses.set([]);
			selectedLines.set([]);
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			lineStart = { x, y };
			lineEnd = { x, y };
			scheduleRender();
		} else if ($activeTool === 'arrow') {
			selectedRectangles.set([]);
			selectedEllipses.set([]);
			selectedLines.set([]);
			selectedArrows.set([]);
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			arrowStart = { x, y };
			arrowEnd = { x, y };
			scheduleRender();
		}
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const worldPos = screenToWorld(screenX, screenY, $viewportOffset, $zoom);
		lastMouseWorldPos = { x: worldPos.x, y: worldPos.y };

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

		const { x, y } = worldPos;
		
		if (isResizing && resizeStartShape && resizeHandleIndex !== null && $editorApi) {
			isShiftPressedDuringResize = event.shiftKey;
			const deltaX = x - resizeStartMousePos.x;
			const deltaY = y - resizeStartMousePos.y;
			
			if (resizeStartShapeType === 'rectangle') {
				const rect = resizeStartShape as Rectangle;
				let minX: number, maxX: number, minY: number, maxY: number;
				
				if (resizeHandleIndex === 0) {
					minX = resizeStartPos.x + deltaX;
					minY = resizeStartPos.y + deltaY;
					maxX = resizeStartPos.x + resizeStartPos.width;
					maxY = resizeStartPos.y + resizeStartPos.height;
				} else if (resizeHandleIndex === 1) {
					minX = resizeStartPos.x;
					minY = resizeStartPos.y + deltaY;
					maxX = resizeStartPos.x + resizeStartPos.width + deltaX;
					maxY = resizeStartPos.y + resizeStartPos.height;
				} else if (resizeHandleIndex === 2) {
					minX = resizeStartPos.x;
					minY = resizeStartPos.y;
					maxX = resizeStartPos.x + resizeStartPos.width + deltaX;
					maxY = resizeStartPos.y + resizeStartPos.height + deltaY;
				} else {
					minX = resizeStartPos.x + deltaX;
					minY = resizeStartPos.y;
					maxX = resizeStartPos.x + resizeStartPos.width;
					maxY = resizeStartPos.y + resizeStartPos.height + deltaY;
				}
				
				if (isShiftPressedDuringResize) {
					const aspectRatio = resizeStartPos.width / resizeStartPos.height;
					const currentWidth = maxX - minX;
					const currentHeight = maxY - minY;
					const widthChange = Math.abs(currentWidth - resizeStartPos.width);
					const heightChange = Math.abs(currentHeight - resizeStartPos.height);
					
					if (widthChange > heightChange) {
						const newHeight = currentWidth / aspectRatio;
						const centerY = (minY + maxY) / 2;
						minY = centerY - newHeight / 2;
						maxY = centerY + newHeight / 2;
					} else {
						const newWidth = currentHeight * aspectRatio;
						const centerX = (minX + maxX) / 2;
						minX = centerX - newWidth / 2;
						maxX = centerX + newWidth / 2;
					}
				}
				
				const finalMinX = Math.min(minX, maxX);
				const finalMaxX = Math.max(minX, maxX);
				const finalMinY = Math.min(minY, maxY);
				const finalMaxY = Math.max(minY, maxY);
				
				const newX = finalMinX;
				const newY = finalMinY;
				const newWidth = finalMaxX - finalMinX;
				const newHeight = finalMaxY - finalMinY;
				
				if (newWidth > 0 && newHeight > 0) {
					resizePreview = { x: newX, y: newY, width: newWidth, height: newHeight, type: 'rectangle', id: rect.id };
					scheduleRender();
				}
			} else if (resizeStartShapeType === 'ellipse') {
				const ellipse = resizeStartShape as Ellipse;
				let minX: number, maxX: number, minY: number, maxY: number;
				
				if (resizeHandleIndex === 0) {
					minX = resizeStartPos.x + deltaX;
					minY = resizeStartPos.y + deltaY;
					maxX = resizeStartPos.x + resizeStartPos.width;
					maxY = resizeStartPos.y + resizeStartPos.height;
				} else if (resizeHandleIndex === 1) {
					minX = resizeStartPos.x;
					minY = resizeStartPos.y + deltaY;
					maxX = resizeStartPos.x + resizeStartPos.width + deltaX;
					maxY = resizeStartPos.y + resizeStartPos.height;
				} else if (resizeHandleIndex === 2) {
					minX = resizeStartPos.x;
					minY = resizeStartPos.y;
					maxX = resizeStartPos.x + resizeStartPos.width + deltaX;
					maxY = resizeStartPos.y + resizeStartPos.height + deltaY;
				} else {
					minX = resizeStartPos.x + deltaX;
					minY = resizeStartPos.y;
					maxX = resizeStartPos.x + resizeStartPos.width;
					maxY = resizeStartPos.y + resizeStartPos.height + deltaY;
				}
				
				if (isShiftPressedDuringResize) {
					const currentWidth = maxX - minX;
					const currentHeight = maxY - minY;
					const maxSize = Math.max(Math.abs(currentWidth), Math.abs(currentHeight));
					const centerX = (minX + maxX) / 2;
					const centerY = (minY + maxY) / 2;
					minX = centerX - maxSize / 2;
					maxX = centerX + maxSize / 2;
					minY = centerY - maxSize / 2;
					maxY = centerY + maxSize / 2;
				}
				
				const finalMinX = Math.min(minX, maxX);
				const finalMaxX = Math.max(minX, maxX);
				const finalMinY = Math.min(minY, maxY);
				const finalMaxY = Math.max(minY, maxY);
				
				const newWidth = finalMaxX - finalMinX;
				const newHeight = finalMaxY - finalMinY;
				
				if (newWidth > 0 && newHeight > 0) {
					const centerX = finalMinX + newWidth / 2;
					const centerY = finalMinY + newHeight / 2;
					const radiusX = newWidth / 2;
					const radiusY = newHeight / 2;
					
					resizePreview = { x: centerX, y: centerY, width: radiusX * 2, height: radiusY * 2, type: 'ellipse', id: ellipse.id };
					scheduleRender();
				}
			} else if (resizeStartShapeType === 'line') {
				const line = resizeStartShape as Line;
				let newStartX = resizeStartPos.x;
				let newStartY = resizeStartPos.y;
				let newEndX = resizeStartPos.x + resizeStartPos.width;
				let newEndY = resizeStartPos.y + resizeStartPos.height;
				
				if (resizeHandleIndex === 0) {
					newStartX = resizeStartPos.x + deltaX;
					newStartY = resizeStartPos.y + deltaY;
				} else if (resizeHandleIndex === 1) {
					newEndX = resizeStartPos.x + resizeStartPos.width + deltaX;
					newEndY = resizeStartPos.y + resizeStartPos.height + deltaY;
				}
				
				if (isShiftPressedDuringResize) {
					const dx = newEndX - newStartX;
					const dy = newEndY - newStartY;
					const angle = Math.atan2(dy, dx);
					const length = Math.sqrt(dx * dx + dy * dy);
					const snapAngle = Math.round(angle / (Math.PI / 8)) * (Math.PI / 8);
					
					if (resizeHandleIndex === 0) {
						newStartX = newEndX - length * Math.cos(snapAngle);
						newStartY = newEndY - length * Math.sin(snapAngle);
					} else {
						newEndX = newStartX + length * Math.cos(snapAngle);
						newEndY = newStartY + length * Math.sin(snapAngle);
					}
				}
				
				resizePreview = { 
					x: newStartX, 
					y: newStartY, 
					width: newEndX - newStartX, 
					height: newEndY - newStartY, 
					type: 'line' as any, 
					id: line.id 
				};
				scheduleRender();
			} else if (resizeStartShapeType === 'arrow') {
				const arrow = resizeStartShape as Arrow;
				let newStartX = resizeStartPos.x;
				let newStartY = resizeStartPos.y;
				let newEndX = resizeStartPos.x + resizeStartPos.width;
				let newEndY = resizeStartPos.y + resizeStartPos.height;
				
				if (resizeHandleIndex === 0) {
					newStartX = resizeStartPos.x + deltaX;
					newStartY = resizeStartPos.y + deltaY;
				} else if (resizeHandleIndex === 1) {
					newEndX = resizeStartPos.x + resizeStartPos.width + deltaX;
					newEndY = resizeStartPos.y + resizeStartPos.height + deltaY;
				}
				
				if (isShiftPressedDuringResize) {
					const dx = newEndX - newStartX;
					const dy = newEndY - newStartY;
					const angle = Math.atan2(dy, dx);
					const length = Math.sqrt(dx * dx + dy * dy);
					const snapAngle = Math.round(angle / (Math.PI / 8)) * (Math.PI / 8);
					
					if (resizeHandleIndex === 0) {
						newStartX = newEndX - length * Math.cos(snapAngle);
						newStartY = newEndY - length * Math.sin(snapAngle);
					} else {
						newEndX = newStartX + length * Math.cos(snapAngle);
						newEndY = newStartY + length * Math.sin(snapAngle);
					}
				}
				
				resizePreview = { 
					x: newStartX, 
					y: newStartY, 
					width: newEndX - newStartX, 
					height: newEndY - newStartY, 
					type: 'arrow' as any, 
					id: arrow.id 
				};
				scheduleRender();
			}
			return;
		}
		
		if ($activeTool === 'rectangle' || $activeTool === 'ellipse') {
			canvas.style.cursor = 'crosshair';
			if (isCreatingShape) {
				isShiftPressedDuringCreation = event.shiftKey;
				createCurrentPos = { x, y };
				scheduleRender();
			}
		} else if ($activeTool === 'line') {
			canvas.style.cursor = 'crosshair';
			if (isCreatingShape && lineStart) {
				lineEnd = { x, y };
				scheduleRender();
			}
		} else if ($activeTool === 'arrow') {
			canvas.style.cursor = 'crosshair';
			if (isCreatingShape && arrowStart) {
				arrowEnd = { x, y };
				scheduleRender();
			}
		} else if ($activeTool === 'select') {
			if (isResizing) {
				canvas.style.cursor = resizeHandleIndex !== null ? resizeCursors[resizeHandleIndex] : 'default';
			} else if (isDragging && draggedShape) {
				canvas.style.cursor = 'move';
			} else {
				for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
					const handleIndex = getResizeHandleAt(x, y, $selectedRectangles[i], 'rectangle', $zoom);
					if (handleIndex !== null) {
						canvas.style.cursor = resizeCursors[handleIndex];
						return;
					}
				}
				for (let i = $selectedEllipses.length - 1; i >= 0; i--) {
					const handleIndex = getResizeHandleAt(x, y, $selectedEllipses[i], 'ellipse', $zoom);
					if (handleIndex !== null) {
						canvas.style.cursor = resizeCursors[handleIndex];
						return;
					}
				}
				
				for (let i = $selectedLines.length - 1; i >= 0; i--) {
					const handleIndex = getLineResizeHandleAt(x, y, $selectedLines[i], $zoom);
					if (handleIndex !== null) {
						canvas.style.cursor = 'pointer';
						return;
					}
				}
				
				for (let i = $selectedArrows.length - 1; i >= 0; i--) {
					const handleIndex = getArrowResizeHandleAt(x, y, $selectedArrows[i], $zoom);
					if (handleIndex !== null) {
						canvas.style.cursor = 'pointer';
						return;
					}
				}
				
				for (let i = $rectangles.length - 1; i >= 0; i--) {
					if (isPointInRectangle(x, y, $rectangles[i])) {
						canvas.style.cursor = 'move';
						return;
					}
				}
				for (let i = $ellipses.length - 1; i >= 0; i--) {
					if (isPointInEllipse(x, y, $ellipses[i])) {
						canvas.style.cursor = 'move';
						return;
					}
				}
			for (let i = $lines.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $lines[i], 5 / $zoom)) {
					canvas.style.cursor = 'move';
					return;
				}
			}
			for (let i = $arrows.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $arrows[i] as any, 5 / $zoom)) {
					canvas.style.cursor = 'move';
					return;
				}
			}
				canvas.style.cursor = 'default';
			}
		} else {
			canvas.style.cursor = 'default';
		}

		if (isDragging && draggedShape && $editorApi) {
			const deltaX = x - dragStartPos.x;
			const deltaY = y - dragStartPos.y;
			
			dragOffset = { x: deltaX, y: deltaY };
			scheduleRender();
		}
	}
	
	function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
		}
		
		if (isResizing && resizePreview && $editorApi) {
			if (resizePreview.type === 'rectangle') {
				moveRectangle(resizePreview.id, resizePreview.x, resizePreview.y, true);
				resizeRectangle(resizePreview.id, resizePreview.width, resizePreview.height, false);
			} else if (resizePreview.type === 'ellipse') {
				const centerX = resizePreview.x;
				const centerY = resizePreview.y;
				const radiusX = resizePreview.width / 2;
				const radiusY = resizePreview.height / 2;
				moveEllipse(resizePreview.id, centerX, centerY, true);
				resizeEllipse(resizePreview.id, radiusX, radiusY, false);
			} else if (resizePreview.type === 'line') {
				const newStartX = resizePreview.x;
				const newStartY = resizePreview.y;
				const newEndX = resizePreview.x + resizePreview.width;
				const newEndY = resizePreview.y + resizePreview.height;
				moveLine(resizePreview.id, newStartX, newStartY, newEndX, newEndY, true);
			} else if (resizePreview.type === 'arrow') {
				const newStartX = resizePreview.x;
				const newStartY = resizePreview.y;
				const newEndX = resizePreview.x + resizePreview.width;
				const newEndY = resizePreview.y + resizePreview.height;
				moveArrow(resizePreview.id, newStartX, newStartY, newEndX, newEndY, true);
			}
			resizePreview = null;
		}
		
		if (isResizing) {
			isResizing = false;
			resizeHandleIndex = null;
			resizeStartShape = null;
			resizeStartShapeType = null;
			resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
			resizeStartMousePos = { x: 0, y: 0 };
			isShiftPressedDuringResize = false;
			resizePreview = null;
		}
		
		if (isDragging && draggedShape && $editorApi) {
			const newX = dragShapeStartPos.x + dragOffset.x;
			const newY = dragShapeStartPos.y + dragOffset.y;
			
			if (draggedShapeType === 'rectangle') {
				moveRectangle((draggedShape as Rectangle).id, newX, newY, true);
			} else if (draggedShapeType === 'ellipse') {
				moveEllipse((draggedShape as Ellipse).id, newX, newY, true);
			} else if (draggedShapeType === 'line') {
				const line = draggedShape as Line;
				const newStartX = line.start.x + dragOffset.x;
				const newStartY = line.start.y + dragOffset.y;
				const newEndX = line.end.x + dragOffset.x;
				const newEndY = line.end.y + dragOffset.y;
				moveLine(line.id, newStartX, newStartY, newEndX, newEndY, true);
			} else if (draggedShapeType === 'arrow') {
				const arrow = draggedShape as Arrow;
				const newStartX = arrow.start.x + dragOffset.x;
				const newStartY = arrow.start.y + dragOffset.y;
				const newEndX = arrow.end.x + dragOffset.x;
				const newEndY = arrow.end.y + dragOffset.y;
				moveArrow(arrow.id, newStartX, newStartY, newEndX, newEndY, true);
			}
		}
		
		if (isCreatingShape) {
			const threshold = $activeTool === 'ellipse' ? 10 : 5;
			
			if ($activeTool === 'rectangle') {
				const width = Math.abs(createCurrentPos.x - createStartPos.x);
				const height = Math.abs(createCurrentPos.y - createStartPos.y);
				
				if (width > threshold && height > threshold) {
					const x = Math.min(createStartPos.x, createCurrentPos.x);
					const y = Math.min(createStartPos.y, createCurrentPos.y);
					addRectangle(x, y, width, height);
					selectedEllipses.set([]);
					if ($rectangles.length > 0) {
						const newestRect = $rectangles[$rectangles.length - 1];
						selectedRectangles.set([newestRect]);
					}
					justCreatedShape = true;
					activeTool.set('select' as Tool);
				}
			} else if ($activeTool === 'ellipse') {
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
					if ($ellipses.length > 0) {
						const newestEllipse = $ellipses[$ellipses.length - 1];
						selectedEllipses.set([newestEllipse]);
					}
					justCreatedShape = true;
					activeTool.set('select' as Tool);
				}
			} else if ($activeTool === 'line') {
				if (lineStart && lineEnd) {
					const dx = lineEnd.x - lineStart.x;
					const dy = lineEnd.y - lineStart.y;
					const length = Math.sqrt(dx * dx + dy * dy);
					
					if (length > 5) {
						addLine(lineStart.x, lineStart.y, lineEnd.x, lineEnd.y);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						if ($lines.length > 0) {
							const newestLine = $lines[$lines.length - 1];
							selectedLines.set([newestLine]);
						}
						justCreatedShape = true;
						activeTool.set('select' as Tool);
					}
				}
			} else if ($activeTool === 'arrow') {
				if (arrowStart && arrowEnd) {
					const dx = arrowEnd.x - arrowStart.x;
					const dy = arrowEnd.y - arrowStart.y;
					const length = Math.sqrt(dx * dx + dy * dy);
					
					if (length > 5) {
						addArrow(arrowStart.x, arrowStart.y, arrowEnd.x, arrowEnd.y);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						selectedLines.set([]);
						if ($arrows.length > 0) {
							const newestArrow = $arrows[$arrows.length - 1];
							selectedArrows.set([newestArrow]);
						}
						justCreatedShape = true;
						activeTool.set('select' as Tool);
					}
				}
			}
			
			isCreatingShape = false;
			createStartPos = { x: 0, y: 0 };
			createCurrentPos = { x: 0, y: 0 };
			lineStart = null;
			lineEnd = null;
			arrowStart = null;
			arrowEnd = null;
			scheduleRender();
		}
		
		isDragging = false;
		draggedShape = null;
		draggedShapeType = null;
		dragOffset = { x: 0, y: 0 };
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
		
		const isCreatingRectangle = $activeTool === 'rectangle' && isCreatingShape;
		const isCreatingEllipse = $activeTool === 'ellipse' && isCreatingShape;
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$rectangles.forEach((rect) => {
			const isSelected = $selectedRectangles.some(selected => selected.id === rect.id);
			const isDragged = isDragging && draggedShape && draggedShapeType === 'rectangle' && draggedShape.id === rect.id;
			const isResized = isResizing && resizePreview && resizePreview.type === 'rectangle' && resizePreview.id === rect.id;
			
			const renderX = isDragged ? rect.position.x + dragOffset.x : (isResized && resizePreview ? resizePreview.x : rect.position.x);
			const renderY = isDragged ? rect.position.y + dragOffset.y : (isResized && resizePreview ? resizePreview.y : rect.position.y);
			const renderWidth = isResized && resizePreview ? resizePreview.width : rect.width;
			const renderHeight = isResized && resizePreview ? resizePreview.height : rect.height;
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.strokeRect(renderX, renderY, renderWidth, renderHeight);
			
			if (isSelected) {
				renderResizeHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom);
			}
		});
		
		if (isCreatingRectangle && previewRect && previewRect.width > 0 && previewRect.height > 0) {
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
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
			const isDragged = isDragging && draggedShape && draggedShapeType === 'ellipse' && draggedShape.id === ellipse.id;
			const isResized = isResizing && resizePreview && resizePreview.type === 'ellipse' && resizePreview.id === ellipse.id;
			
			const renderX = isDragged ? ellipse.position.x + dragOffset.x : (isResized && resizePreview ? resizePreview.x : ellipse.position.x);
			const renderY = isDragged ? ellipse.position.y + dragOffset.y : (isResized && resizePreview ? resizePreview.y : ellipse.position.y);
			const renderRadiusX = isResized && resizePreview ? resizePreview.width / 2 : ellipse.radius_x;
			const renderRadiusY = isResized && resizePreview ? resizePreview.height / 2 : ellipse.radius_y;
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.beginPath();
			renderCtx.ellipse(renderX, renderY, renderRadiusX, renderRadiusY, 0, 0, 2 * Math.PI);
			renderCtx.stroke();
			
			if (isSelected) {
				const x = renderX - renderRadiusX;
				const y = renderY - renderRadiusY;
				const width = renderRadiusX * 2;
				const height = renderRadiusY * 2;
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
				renderCtx.lineWidth = 2;
				renderCtx.globalAlpha = 0.5;
				renderCtx.beginPath();
				renderCtx.ellipse(centerX, centerY, radius_x, radius_y, 0, 0, 2 * Math.PI);
				renderCtx.stroke();
				renderCtx.globalAlpha = 1.0;
			}
		}
		
		$lines.forEach((line: Line) => {
			const isSelected = $selectedLines.some(selected => selected.id === line.id);
			const isDragged = isDragging && draggedShape && draggedShapeType === 'line' && draggedShape.id === line.id;
			const isResized = isResizing && resizePreview && resizePreview.type === 'line' && resizePreview.id === line.id;
			
			let renderStartX: number, renderStartY: number, renderEndX: number, renderEndY: number;
			
			if (isResized && resizePreview) {
				renderStartX = resizePreview.x;
				renderStartY = resizePreview.y;
				renderEndX = resizePreview.x + resizePreview.width;
				renderEndY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
				renderStartX = line.start.x + dragOffset.x;
				renderStartY = line.start.y + dragOffset.y;
				renderEndX = line.end.x + dragOffset.x;
				renderEndY = line.end.y + dragOffset.y;
			} else {
				renderStartX = line.start.x;
				renderStartY = line.start.y;
				renderEndX = line.end.x;
				renderEndY = line.end.y;
			}
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.beginPath();
			renderCtx.moveTo(renderStartX, renderStartY);
			renderCtx.lineTo(renderEndX, renderEndY);
			renderCtx.stroke();
			
			if (isSelected) {
				const handleSize = 8 / $zoom;
				const halfHandle = handleSize / 2;
				
				renderCtx.fillStyle = '#ffffff';
				renderCtx.strokeStyle = '#1e88e5';
				renderCtx.lineWidth = 2 / $zoom;
				
				renderCtx.beginPath();
				renderCtx.arc(renderStartX, renderStartY, halfHandle, 0, 2 * Math.PI);
				renderCtx.fill();
				renderCtx.stroke();
				
				renderCtx.beginPath();
				renderCtx.arc(renderEndX, renderEndY, halfHandle, 0, 2 * Math.PI);
				renderCtx.fill();
				renderCtx.stroke();
			}
		});
		
		if (isCreatingShape && $activeTool === 'line' && lineStart && lineEnd) {
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			renderCtx.moveTo(lineStart.x, lineStart.y);
			renderCtx.lineTo(lineEnd.x, lineEnd.y);
			renderCtx.stroke();
			renderCtx.globalAlpha = 1.0;
		}
		
		$arrows.forEach((arrow: Arrow) => {
			const isSelected = $selectedArrows.some(selected => selected.id === arrow.id);
			const isDragged = isDragging && draggedShape && draggedShapeType === 'arrow' && draggedShape.id === arrow.id;
			const isResized = isResizing && resizePreview && resizePreview.type === 'arrow' && resizePreview.id === arrow.id;
			
			let renderStartX: number, renderStartY: number, renderEndX: number, renderEndY: number;
			
			if (isResized && resizePreview) {
				renderStartX = resizePreview.x;
				renderStartY = resizePreview.y;
				renderEndX = resizePreview.x + resizePreview.width;
				renderEndY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
				renderStartX = arrow.start.x + dragOffset.x;
				renderStartY = arrow.start.y + dragOffset.y;
				renderEndX = arrow.end.x + dragOffset.x;
				renderEndY = arrow.end.y + dragOffset.y;
			} else {
				renderStartX = arrow.start.x;
				renderStartY = arrow.start.y;
				renderEndX = arrow.end.x;
				renderEndY = arrow.end.y;
			}
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.beginPath();
			renderCtx.moveTo(renderStartX, renderStartY);
			renderCtx.lineTo(renderEndX, renderEndY);
			renderCtx.stroke();
			
			const dx = renderEndX - renderStartX;
			const dy = renderEndY - renderStartY;
			const angle = Math.atan2(dy, dx);
			const arrowLength = 20;
			const arrowAngle = Math.PI / 6;
			
			renderCtx.beginPath();
			renderCtx.moveTo(
				renderEndX - arrowLength * Math.cos(angle - arrowAngle),
				renderEndY - arrowLength * Math.sin(angle - arrowAngle)
			);
			renderCtx.lineTo(renderEndX, renderEndY);
			renderCtx.lineTo(
				renderEndX - arrowLength * Math.cos(angle + arrowAngle),
				renderEndY - arrowLength * Math.sin(angle + arrowAngle)
			);
			renderCtx.stroke();
			
			if (isSelected) {
				const handleSize = 8 / $zoom;
				const halfHandle = handleSize / 2;
				
				renderCtx.fillStyle = '#ffffff';
				renderCtx.strokeStyle = '#1e88e5';
				renderCtx.lineWidth = 2 / $zoom;
				
				renderCtx.beginPath();
				renderCtx.arc(renderStartX, renderStartY, halfHandle, 0, 2 * Math.PI);
				renderCtx.fill();
				renderCtx.stroke();
				
				renderCtx.beginPath();
				renderCtx.arc(renderEndX, renderEndY, halfHandle, 0, 2 * Math.PI);
				renderCtx.fill();
				renderCtx.stroke();
			}
		});
		
		if (isCreatingShape && $activeTool === 'arrow' && arrowStart && arrowEnd) {
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			renderCtx.moveTo(arrowStart.x, arrowStart.y);
			renderCtx.lineTo(arrowEnd.x, arrowEnd.y);
			renderCtx.stroke();
			
			const dx = arrowEnd.x - arrowStart.x;
			const dy = arrowEnd.y - arrowStart.y;
			const angle = Math.atan2(dy, dx);
			const arrowLength = 20;
			const arrowAngle = Math.PI / 6;
			
			renderCtx.beginPath();
			renderCtx.moveTo(
				arrowEnd.x - arrowLength * Math.cos(angle - arrowAngle),
				arrowEnd.y - arrowLength * Math.sin(angle - arrowAngle)
			);
			renderCtx.lineTo(arrowEnd.x, arrowEnd.y);
			renderCtx.lineTo(
				arrowEnd.x - arrowLength * Math.cos(angle + arrowAngle),
				arrowEnd.y - arrowLength * Math.sin(angle + arrowAngle)
			);
			renderCtx.stroke();
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
		
		const handleResize = () => {
			if (canvas) {
				canvas.width = window.innerWidth;
				canvas.height = window.innerHeight;
				scheduleRender();
			}
		};
		window.addEventListener('resize', handleResize);
		
		return () => {
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('keyup', handleKeyUp);
			window.removeEventListener('resize', handleResize);
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
		$lines;
		$selectedLines;
		$arrows;
		$selectedArrows;
		if (!isCreatingShape) {
			scheduleRender();
		}
	}
</script>

<div class="relative w-full h-full bg-stone-50">
	<Toolbar />
	<ZoomControls />
	<UndoRedoControls />
	<canvas
		on:mousedown={handleMouseDown}
		on:mousemove={handleMouseMove}
		on:mouseup={handleMouseUp}
		on:wheel={(e) => handleViewportScroll(e, canvas!)}
		bind:this={canvas}
		class="w-full h-full bg-stone-50"
		tabindex="0"
	></canvas>
</div>
