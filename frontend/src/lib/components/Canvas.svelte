<script lang="ts">
	import { onMount } from 'svelte';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		lines, selectedLines, arrows, selectedArrows,
		diamonds, selectedDiamonds,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond 
	} from '$lib/stores/editor';
	import { isPointInRectangle, isPointInEllipse, isPointOnLine, isPointInDiamond, rectangleIntersectsBox, ellipseIntersectsBox, lineIntersectsBox, arrowIntersectsBox, diamondIntersectsBox } from '$lib/utils/geometry';
	import { screenToWorld } from '$lib/utils/viewport';
	import { 
		addRectangle, moveRectangle, resizeRectangle,
		addEllipse, moveEllipse, resizeEllipse,
		addDiamond, moveDiamond, resizeDiamond,
		addLine, moveLine,
		addArrow, moveArrow
	} from '$lib/utils/canvas-operations/index';
	import { handleViewportScroll } from '$lib/utils/viewport-scroll';
	import { zoomIn, zoomOut } from '$lib/utils/zoom';
	import { copyToClipboard, getClipboard, hasClipboardData } from '$lib/utils/clipboard';
	import { updateAllStoresAfterUndoRedo } from '$lib/utils/undo-redo';
	import { pasteShapes } from '$lib/utils/paste-shapes';
	import { clearAllSelections } from '$lib/utils/selection';
	import { deleteShapes } from '$lib/utils/delete-shapes';
	import Toolbar from './Toolbar.svelte';
	import ZoomControls from './ZoomControls.svelte';
	import UndoRedoControls from './UndoRedoControls.svelte';
	import { activeTool, type Tool } from '$lib/stores/tools';

	type BoundingBox = { x: number; y: number; width: number; height: number; rawWidth?: number; rawHeight?: number; scaleX?: number; scaleY?: number };
	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let draggedShape: Rectangle | Ellipse | Line | Arrow | Diamond | null = null;
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
	let resizeStartShape: Rectangle | Ellipse | Line | Arrow | Diamond | null = null;
	let resizeStartShapeType: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | null = null;
	let resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
	let resizeStartMousePos = { x: 0, y: 0 };
	let isShiftPressedDuringResize = false;
	let dragOffset = { x: 0, y: 0 };
	let resizePreview: { x: number; y: number; width: number; height: number; type: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond'; id: number } | null = null;
	let lastMouseWorldPos: { x: number; y: number } | null = null;
	let isSelectingBox = false;
	let selectionBoxStart: { x: number; y: number } | null = null;
	let selectionBoxEnd: { x: number; y: number } | null = null;
	let selectedShapesStartPositions: {
			rectangles: Map<number, { x: number; y: number; width: number; height: number }>;
			ellipses: Map<number, { x: number; y: number; radius_x: number; radius_y: number }>;
			diamonds: Map<number, { x: number; y: number; width: number; height: number }>;
			lines: Map<number, { start: { x: number; y: number }; end: { x: number; y: number } }>;
			arrows: Map<number, { start: { x: number; y: number }; end: { x: number; y: number } }>;
		} = {
			rectangles: new Map(),
			ellipses: new Map(),
			diamonds: new Map(),
			lines: new Map(),
			arrows: new Map()
		};
	let isGroupResizing = false;
let groupResizeHandleIndex: number | null = null;
let groupResizeStartBox: BoundingBox | null = null;
let groupResizeCurrentBox: BoundingBox | null = null;
let groupResizePadding = 0;
	let renderDependencies: Record<string, unknown> | null = null;
	const resizeCursors = ['nwse-resize', 'nesw-resize', 'nwse-resize', 'nesw-resize', 'ns-resize', 'ew-resize', 'ns-resize', 'ew-resize'];

	function handleKeyDown(event: KeyboardEvent) {
		if ((event.ctrlKey || event.metaKey) && (event.key === 'z' || event.key === 'Z')) {
			event.preventDefault();
			if ($editorApi) {
				const success = event.shiftKey ? $editorApi.redo() : $editorApi.undo();
				if (success) {
					updateAllStoresAfterUndoRedo();
					scheduleRender();
				}
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'y' || event.key === 'Y')) {
			event.preventDefault();
			if ($editorApi) {
				const success = $editorApi.redo();
				if (success) {
					updateAllStoresAfterUndoRedo();
					scheduleRender();
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
			if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond' || $activeTool === 'line' || $activeTool === 'arrow') {
				activeTool.set('select');
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'c' || event.key === 'C')) {
			event.preventDefault();
			if ($selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0) {
				copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds);
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'd') {
			event.preventDefault();
			event.stopPropagation();
			if (!$editorApi) return;
			
			const hasSelection = $selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0;
			if (!hasSelection) return;
			
			copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds);
			const clipboard = getClipboard();
			
			const bounds: Array<{ minX: number; minY: number }> = [];
			clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
			clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
			clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
			clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
			clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
			
			if (bounds.length === 0) return;
			
			const minX = Math.min(...bounds.map(b => b.minX));
			const minY = Math.min(...bounds.map(b => b.minY));
			const duplicateX = minX + 20;
			const duplicateY = minY + 20;
			
			pasteShapes(clipboard, duplicateX, duplicateY);
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') {
			event.preventDefault();
			selectedRectangles.set([...$rectangles]);
			selectedEllipses.set([...$ellipses]);
			selectedDiamonds.set([...$diamonds]);
			selectedLines.set([...$lines]);
			selectedArrows.set([...$arrows]);
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'v') {
			event.preventDefault();
			if (!$editorApi || !hasClipboardData()) return;
			
			const clipboard = getClipboard();
			let pasteX: number, pasteY: number;
			
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
			
			pasteShapes(clipboard, pasteX, pasteY);
			return;
		}

		if (!$editorApi || (event.key !== 'Delete' && event.key !== 'Backspace')) return;

		const hasSelectedRectangles = $selectedRectangles.length > 0;
		const hasSelectedEllipses = $selectedEllipses.length > 0;
		const hasSelectedDiamonds = $selectedDiamonds.length > 0;
		const hasSelectedLines = $selectedLines.length > 0;
		const hasSelectedArrows = $selectedArrows.length > 0;

		if (!hasSelectedRectangles && !hasSelectedEllipses && !hasSelectedDiamonds && !hasSelectedLines && !hasSelectedArrows) return;

		event.preventDefault();
		
		const rectangleIds = hasSelectedRectangles ? $selectedRectangles.map(rect => rect.id) : [];
		const ellipseIds = hasSelectedEllipses ? $selectedEllipses.map(ellipse => ellipse.id) : [];
		const diamondIds = hasSelectedDiamonds ? $selectedDiamonds.map(diamond => diamond.id) : [];
		const lineIds = hasSelectedLines ? $selectedLines.map(line => line.id) : [];
		const arrowIds = hasSelectedArrows ? $selectedArrows.map(arrow => arrow.id) : [];
		
		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds);
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.key === ' ') {
			isSpacePressed = false;
		}
	}

	function getResizeHandleAt(
		x: number,
		y: number,
		shape: Rectangle | Ellipse | Diamond,
		shapeType: 'rectangle' | 'ellipse' | 'diamond',
		zoom: number
	): number | null {
		const gap = 4 / zoom;
		
		let boxX: number, boxY: number, boxWidth: number, boxHeight: number;
		
		if (shapeType === 'rectangle') {
			const rect = shape as Rectangle;
			boxX = rect.position.x - gap;
			boxY = rect.position.y - gap;
			boxWidth = rect.width + gap * 2;
			boxHeight = rect.height + gap * 2;
		} else if (shapeType === 'diamond') {
			const diamond = shape as Diamond;
			boxX = diamond.position.x - gap;
			boxY = diamond.position.y - gap;
			boxWidth = diamond.width + gap * 2;
			boxHeight = diamond.height + gap * 2;
		} else {
			const ellipse = shape as Ellipse;
			boxX = ellipse.position.x - ellipse.radius_x - gap;
			boxY = ellipse.position.y - ellipse.radius_y - gap;
			boxWidth = (ellipse.radius_x * 2) + gap * 2;
			boxHeight = (ellipse.radius_y * 2) + gap * 2;
		}
		
		return hitTestHandles(
			x,
			y,
			{ x: boxX, y: boxY, width: boxWidth, height: boxHeight },
			zoom
		);
	}

	function getHandlePositions(box: { x: number; y: number; width: number; height: number }): Array<{ x: number; y: number }> {
		const { x, y, width, height } = box;
		const midX = x + width / 2;
		const midY = y + height / 2;
		return [
			{ x, y },
			{ x: x + width, y },
			{ x: x + width, y: y + height },
			{ x, y: y + height },
			{ x: midX, y },
			{ x: x + width, y: midY },
			{ x: midX, y: y + height },
			{ x, y: midY }
		];
	}

	function getSelectionPaddingValue(currentZoom: number): number {
		return 4 / currentZoom;
	}

	function expandBoundingBox(box: BoundingBox, padding: number): BoundingBox {
		return {
			x: box.x - padding,
			y: box.y - padding,
			width: box.width + padding * 2,
			height: box.height + padding * 2
		};
	}

	function hitTestHandles(
		x: number,
		y: number,
		box: { x: number; y: number; width: number; height: number },
		zoom: number
	): number | null {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const edgeTolerance = Math.max(6 / zoom, handleSize * 0.75);
		const handles = getHandlePositions(box);

		for (let i = 0; i < handles.length; i++) {
			if (i < 4) {
				const handle = handles[i];
				if (
					Math.abs(x - handle.x) <= halfHandle &&
					Math.abs(y - handle.y) <= halfHandle
				) {
					return i;
				}
			}
		}

		const left = box.x;
		const right = box.x + box.width;
		const top = box.y;
		const bottom = box.y + box.height;

		if (
			Math.abs(y - top) <= edgeTolerance &&
			x >= left - edgeTolerance &&
			x <= right + edgeTolerance
		) {
			return 4;
		}

		if (
			Math.abs(x - right) <= edgeTolerance &&
			y >= top - edgeTolerance &&
			y <= bottom + edgeTolerance
		) {
			return 5;
		}

		if (
			Math.abs(y - bottom) <= edgeTolerance &&
			x >= left - edgeTolerance &&
			x <= right + edgeTolerance
		) {
			return 6;
		}

		if (
			Math.abs(x - left) <= edgeTolerance &&
			y >= top - edgeTolerance &&
			y <= bottom + edgeTolerance
		) {
			return 7;
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

	function storeSelectedShapesStartPositions() {
		selectedShapesStartPositions.rectangles.clear();
		selectedShapesStartPositions.ellipses.clear();
		selectedShapesStartPositions.diamonds.clear();
		selectedShapesStartPositions.lines.clear();
		selectedShapesStartPositions.arrows.clear();
		
		$selectedRectangles.forEach(rect => {
			selectedShapesStartPositions.rectangles.set(rect.id, { x: rect.position.x, y: rect.position.y, width: rect.width, height: rect.height });
		});
		
		$selectedEllipses.forEach(ellipse => {
			selectedShapesStartPositions.ellipses.set(ellipse.id, { x: ellipse.position.x, y: ellipse.position.y, radius_x: ellipse.radius_x, radius_y: ellipse.radius_y });
		});
		
		$selectedDiamonds.forEach(diamond => {
			selectedShapesStartPositions.diamonds.set(diamond.id, { x: diamond.position.x, y: diamond.position.y, width: diamond.width, height: diamond.height });
		});
		
		$selectedLines.forEach(line => {
			selectedShapesStartPositions.lines.set(line.id, { start: { x: line.start.x, y: line.start.y }, end: { x: line.end.x, y: line.end.y } });
		});
		
		$selectedArrows.forEach(arrow => {
			selectedShapesStartPositions.arrows.set(arrow.id, { start: { x: arrow.start.x, y: arrow.start.y }, end: { x: arrow.end.x, y: arrow.end.y } });
		});
	}

	function getGroupResizeHandleAt(x: number, y: number, box: BoundingBox, zoom: number): number | null {
		return hitTestHandles(x, y, box, zoom);
	}

	function renderGroupResizeHandles(ctx: CanvasRenderingContext2D, box: BoundingBox, zoom: number) {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const handles = getHandlePositions(box).slice(0, 4);

		ctx.fillStyle = '#ffffff';
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 2 / zoom;

		handles.forEach(handle => {
			ctx.beginPath();
			ctx.rect(handle.x - halfHandle, handle.y - halfHandle, handleSize, handleSize);
			ctx.fill();
			ctx.stroke();
		});
	}

	function computeGroupResizeBox(targetX: number, targetY: number): BoundingBox | null {
		if (!groupResizeStartBox || groupResizeHandleIndex === null) return null;
		const start = groupResizeStartBox;
		const startLeft = start.x;
		const startRight = start.x + start.width;
		const startTop = start.y;
		const startBottom = start.y + start.height;
		const padding = groupResizePadding;

		const adjustX = (value: number): number => {
			if (groupResizeHandleIndex === 0 || groupResizeHandleIndex === 3 || groupResizeHandleIndex === 7) {
				return value + padding;
			}
			if (groupResizeHandleIndex === 1 || groupResizeHandleIndex === 2 || groupResizeHandleIndex === 5) {
				return value - padding;
			}
			return value;
		};

		const adjustY = (value: number): number => {
			if (groupResizeHandleIndex === 0 || groupResizeHandleIndex === 1 || groupResizeHandleIndex === 4) {
				return value + padding;
			}
			if (groupResizeHandleIndex === 2 || groupResizeHandleIndex === 3 || groupResizeHandleIndex === 6) {
				return value - padding;
			}
			return value;
		};

		let newLeft = startLeft;
		let newRight = startRight;
		let newTop = startTop;
		let newBottom = startBottom;

		switch (groupResizeHandleIndex) {
			case 0:
				newLeft = adjustX(targetX);
				newTop = adjustY(targetY);
				break;
			case 1:
				newRight = adjustX(targetX);
				newTop = adjustY(targetY);
				break;
			case 2:
				newRight = adjustX(targetX);
				newBottom = adjustY(targetY);
				break;
			case 3:
				newLeft = adjustX(targetX);
				newBottom = adjustY(targetY);
				break;
			case 4:
				newTop = adjustY(targetY);
				break;
			case 5:
				newRight = adjustX(targetX);
				break;
			case 6:
				newBottom = adjustY(targetY);
				break;
			case 7:
				newLeft = adjustX(targetX);
				break;
		}

		const rawWidth = newRight - newLeft;
		const rawHeight = newBottom - newTop;
		const normalizedX = rawWidth >= 0 ? newLeft : newRight;
		const normalizedY = rawHeight >= 0 ? newTop : newBottom;

		return {
			x: normalizedX,
			y: normalizedY,
			width: Math.abs(rawWidth),
			height: Math.abs(rawHeight),
			rawWidth,
			rawHeight,
			scaleX: start.width === 0 ? Math.sign(rawWidth) || 1 : rawWidth / start.width,
			scaleY: start.height === 0 ? Math.sign(rawHeight) || 1 : rawHeight / start.height
		};
	}

	function resizeSelectedShapesToBox(targetBox: BoundingBox, saveHistory: boolean) {
		if (!groupResizeStartBox) return;
		const startBox = groupResizeStartBox;
		const scaleX =
			startBox.width === 0
				? (Math.sign(targetBox.rawWidth ?? targetBox.width) || 1)
				: (targetBox.rawWidth ?? targetBox.width) / startBox.width;
		const scaleY =
			startBox.height === 0
				? (Math.sign(targetBox.rawHeight ?? targetBox.height) || 1)
				: (targetBox.rawHeight ?? targetBox.height) / startBox.height;

		const originX = scaleX >= 0 ? targetBox.x : targetBox.x + targetBox.width;
		const originY = scaleY >= 0 ? targetBox.y : targetBox.y + targetBox.height;

		selectedShapesStartPositions.rectangles.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newX = originX + relativeX * scaleX;
			const newY = originY + relativeY * scaleY;
			const newWidth = Math.abs(startPos.width * scaleX);
			const newHeight = Math.abs(startPos.height * scaleY);
			moveRectangle(id, newX, newY, saveHistory);
			resizeRectangle(id, newWidth, newHeight, saveHistory);
		});

		selectedShapesStartPositions.ellipses.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newX = originX + relativeX * scaleX;
			const newY = originY + relativeY * scaleY;
			const newRadiusX = Math.abs(startPos.radius_x * scaleX);
			const newRadiusY = Math.abs(startPos.radius_y * scaleY);
			moveEllipse(id, newX, newY, saveHistory);
			resizeEllipse(id, newRadiusX, newRadiusY, saveHistory);
		});

		selectedShapesStartPositions.diamonds.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newX = originX + relativeX * scaleX;
			const newY = originY + relativeY * scaleY;
			const newWidth = Math.abs(startPos.width * scaleX);
			const newHeight = Math.abs(startPos.height * scaleY);
			moveDiamond(id, newX, newY, saveHistory);
			resizeDiamond(id, newWidth, newHeight, saveHistory);
		});

		selectedShapesStartPositions.lines.forEach((startPos, id) => {
			const startRelativeX = startPos.start.x - startBox.x;
			const startRelativeY = startPos.start.y - startBox.y;
			const endRelativeX = startPos.end.x - startBox.x;
			const endRelativeY = startPos.end.y - startBox.y;
			const newStartX = originX + startRelativeX * scaleX;
			const newStartY = originY + startRelativeY * scaleY;
			const newEndX = originX + endRelativeX * scaleX;
			const newEndY = originY + endRelativeY * scaleY;
			moveLine(id, newStartX, newStartY, newEndX, newEndY, saveHistory);
		});

		selectedShapesStartPositions.arrows.forEach((startPos, id) => {
			const startRelativeX = startPos.start.x - startBox.x;
			const startRelativeY = startPos.start.y - startBox.y;
			const endRelativeX = startPos.end.x - startBox.x;
			const endRelativeY = startPos.end.y - startBox.y;
			const newStartX = originX + startRelativeX * scaleX;
			const newStartY = originY + startRelativeY * scaleY;
			const newEndX = originX + endRelativeX * scaleX;
			const newEndY = originY + endRelativeY * scaleY;
			moveArrow(id, newStartX, newStartY, newEndX, newEndY, saveHistory);
		});
	}

	function handleShapeClick(
		shape: Rectangle | Ellipse | Diamond,
		shapeType: 'rectangle' | 'ellipse' | 'diamond',
		isShiftPressed: boolean,
		x: number,
		y: number
	) {
		if (shapeType === 'rectangle') {
			const clickedRect = shape as Rectangle;
			const index = $selectedRectangles.findIndex(r => r.id === clickedRect.id);
			const isAlreadySelected = index >= 0;
			
			if (isShiftPressed) {
				selectedRectangles.set(
					isAlreadySelected
						? $selectedRectangles.filter(r => r.id !== clickedRect.id)
						: [...$selectedRectangles, clickedRect]
				);
			} else if (!isAlreadySelected) {
				selectedRectangles.set([clickedRect]);
				selectedEllipses.set([]);
				selectedDiamonds.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
			}

			draggedShape = clickedRect;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'diamond') {
			const clickedDiamond = shape as Diamond;
			const index = $selectedDiamonds.findIndex(d => d.id === clickedDiamond.id);
			const isAlreadySelected = index >= 0;
			
			if (isShiftPressed) {
				selectedDiamonds.set(
					isAlreadySelected
						? $selectedDiamonds.filter(d => d.id !== clickedDiamond.id)
						: [...$selectedDiamonds, clickedDiamond]
				);
			} else if (!isAlreadySelected) {
				selectedDiamonds.set([clickedDiamond]);
				selectedRectangles.set([]);
				selectedEllipses.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
			}

			draggedShape = clickedDiamond;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else {
			const clickedEllipse = shape as Ellipse;
			const index = $selectedEllipses.findIndex(e => e.id === clickedEllipse.id);
			const isAlreadySelected = index >= 0;
			
			if (isShiftPressed) {
				selectedEllipses.set(
					isAlreadySelected
						? $selectedEllipses.filter(e => e.id !== clickedEllipse.id)
						: [...$selectedEllipses, clickedEllipse]
				);
			} else if (!isAlreadySelected) {
				selectedEllipses.set([clickedEllipse]);
				selectedRectangles.set([]);
				selectedDiamonds.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
			}

			draggedShape = clickedEllipse;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
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

		const isShiftPressed = event.shiftKey;
		
		if ($activeTool === 'select') {
			const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length;
			const allowIndividualHandles = totalSelectedCount <= 1;
			let rawGroupBox: BoundingBox | null = null;
			let visualGroupBox: BoundingBox | null = null;
			const selectionPaddingValue = getSelectionPaddingValue($zoom);
			if (totalSelectedCount > 1) {
				rawGroupBox = groupResizeCurrentBox ?? calculateGroupBoundingBox();
				if (rawGroupBox) {
					visualGroupBox = expandBoundingBox(rawGroupBox, selectionPaddingValue);
				}
			}

			if (allowIndividualHandles) {
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

				for (let i = $selectedDiamonds.length - 1; i >= 0; i--) {
					const handleIndex = getResizeHandleAt(x, y, $selectedDiamonds[i], 'diamond', $zoom);
					if (handleIndex !== null) {
						isResizing = true;
						resizeHandleIndex = handleIndex;
						resizeStartShape = $selectedDiamonds[i];
						resizeStartShapeType = 'diamond';
						resizeStartPos = {
							x: $selectedDiamonds[i].position.x,
							y: $selectedDiamonds[i].position.y,
							width: $selectedDiamonds[i].width,
							height: $selectedDiamonds[i].height
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
			}

			if (visualGroupBox && rawGroupBox) {
				const groupHandleIndex = getGroupResizeHandleAt(x, y, visualGroupBox, $zoom);
				if (groupHandleIndex !== null) {
					isGroupResizing = true;
					groupResizeHandleIndex = groupHandleIndex;
					groupResizeStartBox = rawGroupBox;
					groupResizeCurrentBox = rawGroupBox;
					groupResizePadding = selectionPaddingValue;
					storeSelectedShapesStartPositions();
					if ($editorApi) {
						$editorApi.save_snapshot();
					}
					canvas.style.cursor = resizeCursors[groupHandleIndex];
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

			for (let i = $diamonds.length - 1; i >= 0; i--) {
				if (isPointInDiamond(x, y, $diamonds[i])) {
					handleShapeClick($diamonds[i], 'diamond', isShiftPressed, x, y);
					return;
				}
			}

			for (let i = $lines.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $lines[i], 5 / $zoom)) {
					const clickedLine = $lines[i];
					const index = $selectedLines.findIndex(l => l.id === clickedLine.id);
					const isAlreadySelected = index >= 0;
					
					if (isShiftPressed) {
						selectedLines.set(
							isAlreadySelected
								? $selectedLines.filter(l => l.id !== clickedLine.id)
								: [...$selectedLines, clickedLine]
						);
					} else if (!isAlreadySelected) {
						selectedLines.set([clickedLine]);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						selectedDiamonds.set([]);
						selectedArrows.set([]);
					}
					
					draggedShape = clickedLine;
					dragStartPos = { x, y };
					dragOffset = { x: 0, y: 0 };
					storeSelectedShapesStartPositions();
					isDragging = true;
					return;
				}
			}

			for (let i = $arrows.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $arrows[i], 5 / $zoom)) {
					const clickedArrow = $arrows[i];
					const index = $selectedArrows.findIndex(a => a.id === clickedArrow.id);
					const isAlreadySelected = index >= 0;
					
					if (isShiftPressed) {
						selectedArrows.set(
							isAlreadySelected
								? $selectedArrows.filter(a => a.id !== clickedArrow.id)
								: [...$selectedArrows, clickedArrow]
						);
					} else if (!isAlreadySelected) {
						selectedArrows.set([clickedArrow]);
						selectedRectangles.set([]);
						selectedEllipses.set([]);
						selectedDiamonds.set([]);
						selectedLines.set([]);
					}
					
					draggedShape = clickedArrow;
					dragStartPos = { x, y };
					dragOffset = { x: 0, y: 0 };
					storeSelectedShapesStartPositions();
					isDragging = true;
					return;
				}
			}

			if (totalSelectedCount > 1 && visualGroupBox) {
				if (
					x >= visualGroupBox.x &&
					x <= visualGroupBox.x + visualGroupBox.width &&
					y >= visualGroupBox.y &&
					y <= visualGroupBox.y + visualGroupBox.height
				) {
					const firstSelectedRect = $selectedRectangles[0];
					const firstSelectedEllipse = $selectedEllipses[0];
					const firstSelectedDiamond = $selectedDiamonds[0];
					const firstSelectedLine = $selectedLines[0];
					const firstSelectedArrow = $selectedArrows[0];
					
					if (firstSelectedRect) {
						draggedShape = firstSelectedRect;
					} else if (firstSelectedEllipse) {
						draggedShape = firstSelectedEllipse;
					} else if (firstSelectedDiamond) {
						draggedShape = firstSelectedDiamond;
					} else if (firstSelectedLine) {
						draggedShape = firstSelectedLine;
					} else if (firstSelectedArrow) {
						draggedShape = firstSelectedArrow;
					}
					
					if (draggedShape) {
						dragStartPos = { x, y };
						dragOffset = { x: 0, y: 0 };
						storeSelectedShapesStartPositions();
						isDragging = true;
						return;
					}
				}
			}

			if (isShiftPressed) return;

			isSelectingBox = true;
			selectionBoxStart = { x, y };
			selectionBoxEnd = { x, y };
			clearAllSelections();
		} else if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond') {
			clearAllSelections();
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			createStartPos = { x, y };
			createCurrentPos = { x, y };
			scheduleRender();
		} else if ($activeTool === 'line') {
			clearAllSelections();
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			lineStart = { x, y };
			lineEnd = { x, y };
			scheduleRender();
		} else if ($activeTool === 'arrow') {
			clearAllSelections();
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
		
		if (isGroupResizing && groupResizeHandleIndex !== null) {
			const newBox = computeGroupResizeBox(x, y);
			if (newBox) {
				groupResizeCurrentBox = newBox;
				resizeSelectedShapesToBox(newBox, false);
				canvas.style.cursor = resizeCursors[groupResizeHandleIndex];
				scheduleRender();
			}
			return;
		}
		
		if (isResizing && resizeStartShape && resizeHandleIndex !== null && $editorApi) {
			isShiftPressedDuringResize = event.shiftKey;
			const deltaX = x - resizeStartMousePos.x;
			const deltaY = y - resizeStartMousePos.y;
			
			if (resizeStartShapeType === 'rectangle') {
				const rect = resizeStartShape as Rectangle;
				const affectsLeft = resizeHandleIndex === 0 || resizeHandleIndex === 3 || resizeHandleIndex === 7;
				const affectsRight = resizeHandleIndex === 1 || resizeHandleIndex === 2 || resizeHandleIndex === 5;
				const affectsTop = resizeHandleIndex === 0 || resizeHandleIndex === 1 || resizeHandleIndex === 4;
				const affectsBottom = resizeHandleIndex === 2 || resizeHandleIndex === 3 || resizeHandleIndex === 6;
				const adjustsHorizontal = affectsLeft || affectsRight;
				const adjustsVertical = affectsTop || affectsBottom;

				let left = resizeStartPos.x;
				let right = resizeStartPos.x + resizeStartPos.width;
				let top = resizeStartPos.y;
				let bottom = resizeStartPos.y + resizeStartPos.height;

				if (affectsLeft) left = resizeStartPos.x + deltaX;
				if (affectsRight) right = resizeStartPos.x + resizeStartPos.width + deltaX;
				if (affectsTop) top = resizeStartPos.y + deltaY;
				if (affectsBottom) bottom = resizeStartPos.y + resizeStartPos.height + deltaY;

				if (isShiftPressedDuringResize && adjustsHorizontal && adjustsVertical && resizeStartPos.height !== 0) {
					const aspectRatio = resizeStartPos.width / resizeStartPos.height;
					const width = right - left;
					const height = bottom - top;
					const absWidth = Math.abs(width);
					const absHeight = Math.abs(height);

					if (absHeight === 0 || absWidth / absHeight > aspectRatio) {
						const targetHeight = absWidth / aspectRatio;
						if (height >= 0) {
							bottom = top + targetHeight;
						} else {
							top = bottom + targetHeight;
						}
					} else {
						const targetWidth = absHeight * aspectRatio;
						if (width >= 0) {
							right = left + targetWidth;
						} else {
							left = right + targetWidth;
						}
					}
				}

				const finalLeft = Math.min(left, right);
				const finalRight = Math.max(left, right);
				const finalTop = Math.min(top, bottom);
				const finalBottom = Math.max(top, bottom);

				const newWidth = finalRight - finalLeft;
				const newHeight = finalBottom - finalTop;

				resizePreview = { x: finalLeft, y: finalTop, width: newWidth, height: newHeight, type: 'rectangle', id: rect.id };
				scheduleRender();
			} else if (resizeStartShapeType === 'ellipse') {
				const ellipse = resizeStartShape as Ellipse;
				const affectsLeft = resizeHandleIndex === 0 || resizeHandleIndex === 3 || resizeHandleIndex === 7;
				const affectsRight = resizeHandleIndex === 1 || resizeHandleIndex === 2 || resizeHandleIndex === 5;
				const affectsTop = resizeHandleIndex === 0 || resizeHandleIndex === 1 || resizeHandleIndex === 4;
				const affectsBottom = resizeHandleIndex === 2 || resizeHandleIndex === 3 || resizeHandleIndex === 6;
				const adjustsHorizontal = affectsLeft || affectsRight;
				const adjustsVertical = affectsTop || affectsBottom;

				let left = resizeStartPos.x;
				let right = resizeStartPos.x + resizeStartPos.width;
				let top = resizeStartPos.y;
				let bottom = resizeStartPos.y + resizeStartPos.height;

				if (affectsLeft) left = resizeStartPos.x + deltaX;
				if (affectsRight) right = resizeStartPos.x + resizeStartPos.width + deltaX;
				if (affectsTop) top = resizeStartPos.y + deltaY;
				if (affectsBottom) bottom = resizeStartPos.y + resizeStartPos.height + deltaY;

				if (isShiftPressedDuringResize && adjustsHorizontal && adjustsVertical) {
					const width = right - left;
					const height = bottom - top;
					const maxSize = Math.max(Math.abs(width), Math.abs(height));
					if (height >= 0) {
						bottom = top + Math.sign(height || 1) * maxSize;
					} else {
						top = bottom + Math.sign(height || 1) * maxSize;
					}
					if (width >= 0) {
						right = left + Math.sign(width || 1) * maxSize;
					} else {
						left = right + Math.sign(width || 1) * maxSize;
					}
				}

				const finalLeft = Math.min(left, right);
				const finalRight = Math.max(left, right);
				const finalTop = Math.min(top, bottom);
				const finalBottom = Math.max(top, bottom);

				const newWidth = finalRight - finalLeft;
				const newHeight = finalBottom - finalTop;
				const centerX = finalLeft + newWidth / 2;
				const centerY = finalTop + newHeight / 2;
				const radiusX = Math.abs(newWidth / 2);
				const radiusY = Math.abs(newHeight / 2);

				resizePreview = { x: centerX, y: centerY, width: radiusX * 2, height: radiusY * 2, type: 'ellipse', id: ellipse.id };
				scheduleRender();
			} else if (resizeStartShapeType === 'diamond') {
				const diamond = resizeStartShape as Diamond;
				const affectsLeft = resizeHandleIndex === 0 || resizeHandleIndex === 3 || resizeHandleIndex === 7;
				const affectsRight = resizeHandleIndex === 1 || resizeHandleIndex === 2 || resizeHandleIndex === 5;
				const affectsTop = resizeHandleIndex === 0 || resizeHandleIndex === 1 || resizeHandleIndex === 4;
				const affectsBottom = resizeHandleIndex === 2 || resizeHandleIndex === 3 || resizeHandleIndex === 6;
				const adjustsHorizontal = affectsLeft || affectsRight;
				const adjustsVertical = affectsTop || affectsBottom;

				let left = resizeStartPos.x;
				let right = resizeStartPos.x + resizeStartPos.width;
				let top = resizeStartPos.y;
				let bottom = resizeStartPos.y + resizeStartPos.height;

				if (affectsLeft) left = resizeStartPos.x + deltaX;
				if (affectsRight) right = resizeStartPos.x + resizeStartPos.width + deltaX;
				if (affectsTop) top = resizeStartPos.y + deltaY;
				if (affectsBottom) bottom = resizeStartPos.y + resizeStartPos.height + deltaY;

				if (isShiftPressedDuringResize && adjustsHorizontal && adjustsVertical && resizeStartPos.height !== 0) {
					const aspectRatio = resizeStartPos.width / resizeStartPos.height;
					const width = right - left;
					const height = bottom - top;
					const absWidth = Math.abs(width);
					const absHeight = Math.abs(height);

					if (absHeight === 0 || absWidth / absHeight > aspectRatio) {
						const targetHeight = absWidth / aspectRatio;
						if (height >= 0) {
							bottom = top + targetHeight;
						} else {
							top = bottom + targetHeight;
						}
					} else {
						const targetWidth = absHeight * aspectRatio;
						if (width >= 0) {
							right = left + targetWidth;
						} else {
							left = right + targetWidth;
						}
					}
				}

				const finalLeft = Math.min(left, right);
				const finalRight = Math.max(left, right);
				const finalTop = Math.min(top, bottom);
				const finalBottom = Math.max(top, bottom);

				const newWidth = finalRight - finalLeft;
				const newHeight = finalBottom - finalTop;

				resizePreview = { x: finalLeft, y: finalTop, width: newWidth, height: newHeight, type: 'diamond', id: diamond.id };
				scheduleRender();
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
					type: 'line', 
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
					type: 'arrow', 
					id: arrow.id 
				};
				scheduleRender();
			}
			return;
		}
		
		if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond') {
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
				const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length;
				const selectionPaddingValue = getSelectionPaddingValue($zoom);
				const rawGroupBoxForCursor =
					totalSelectedCount > 1 ? groupResizeCurrentBox ?? calculateGroupBoundingBox() : null;
				const visualGroupBoxForCursor = rawGroupBoxForCursor
					? expandBoundingBox(rawGroupBoxForCursor, selectionPaddingValue)
					: null;

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
				
				for (let i = $selectedDiamonds.length - 1; i >= 0; i--) {
					const handleIndex = getResizeHandleAt(x, y, $selectedDiamonds[i], 'diamond', $zoom);
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
				
			if (visualGroupBoxForCursor) {
				const groupHandleIndex = getGroupResizeHandleAt(x, y, visualGroupBoxForCursor, $zoom);
				if (groupHandleIndex !== null) {
					canvas.style.cursor = resizeCursors[groupHandleIndex];
					return;
				}
			}
				
				if (visualGroupBoxForCursor) {
					const box = visualGroupBoxForCursor;
					if (x >= box.x && x <= box.x + box.width && y >= box.y && y <= box.y + box.height) {
						canvas.style.cursor = 'move';
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
				for (let i = $diamonds.length - 1; i >= 0; i--) {
					if (isPointInDiamond(x, y, $diamonds[i])) {
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
				if (isPointOnLine(x, y, $arrows[i], 5 / $zoom)) {
					canvas.style.cursor = 'move';
					return;
				}
			}
				canvas.style.cursor = 'default';
			}
		} else {
			canvas.style.cursor = 'default';
		}

		if (isSelectingBox && selectionBoxStart) {
			selectionBoxEnd = { x, y };
			scheduleRender();
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
			} else if (resizePreview.type === 'diamond') {
				moveDiamond(resizePreview.id, resizePreview.x, resizePreview.y, true);
				resizeDiamond(resizePreview.id, resizePreview.width, resizePreview.height, false);
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
		
		if (isSelectingBox && selectionBoxStart && selectionBoxEnd) {
			const boxX = Math.min(selectionBoxStart.x, selectionBoxEnd.x);
			const boxY = Math.min(selectionBoxStart.y, selectionBoxEnd.y);
			const boxWidth = Math.abs(selectionBoxEnd.x - selectionBoxStart.x);
			const boxHeight = Math.abs(selectionBoxEnd.y - selectionBoxStart.y);
			
			if (boxWidth > 2 && boxHeight > 2) {
				const box = { x: boxX, y: boxY, width: boxWidth, height: boxHeight };
				
				const selectedRects: Rectangle[] = [];
				const selectedElls: Ellipse[] = [];
				const selectedDias: Diamond[] = [];
				const selectedLinesArray: Line[] = [];
				const selectedArrs: Arrow[] = [];
				
				$rectangles.forEach(rect => {
					if (rectangleIntersectsBox(rect, box)) {
						selectedRects.push(rect);
					}
				});
				
				$ellipses.forEach(ellipse => {
					if (ellipseIntersectsBox(ellipse, box)) {
						selectedElls.push(ellipse);
					}
				});
				
				$diamonds.forEach(diamond => {
					if (diamondIntersectsBox(diamond, box)) {
						selectedDias.push(diamond);
					}
				});
				
				$lines.forEach(line => {
					if (lineIntersectsBox(line, box)) {
						selectedLinesArray.push(line);
					}
				});
				
				$arrows.forEach(arrow => {
					if (arrowIntersectsBox(arrow, box)) {
						selectedArrs.push(arrow);
					}
				});
				
				selectedRectangles.set(selectedRects);
				selectedEllipses.set(selectedElls);
				selectedDiamonds.set(selectedDias);
				selectedLines.set(selectedLinesArray);
				selectedArrows.set(selectedArrs);
			}
			
			isSelectingBox = false;
			selectionBoxStart = null;
			selectionBoxEnd = null;
			scheduleRender();
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
		
		if (isGroupResizing) {
			if (groupResizeCurrentBox) {
				resizeSelectedShapesToBox(groupResizeCurrentBox, false);
			}
			if ($editorApi) {
				$editorApi.save_snapshot();
			}
			isGroupResizing = false;
			groupResizeHandleIndex = null;
			groupResizeStartBox = null;
			groupResizeCurrentBox = null;
			groupResizePadding = 0;
		}
		
		if (isDragging && draggedShape && $editorApi) {
			$editorApi.save_snapshot();
			
			selectedShapesStartPositions.rectangles.forEach((startPos, id) => {
				const newX = startPos.x + dragOffset.x;
				const newY = startPos.y + dragOffset.y;
				moveRectangle(id, newX, newY, false);
			});
			
			selectedShapesStartPositions.ellipses.forEach((startPos, id) => {
				const newX = startPos.x + dragOffset.x;
				const newY = startPos.y + dragOffset.y;
				moveEllipse(id, newX, newY, false);
			});
			
			selectedShapesStartPositions.diamonds.forEach((startPos, id) => {
				const newX = startPos.x + dragOffset.x;
				const newY = startPos.y + dragOffset.y;
				moveDiamond(id, newX, newY, false);
			});
			
			selectedShapesStartPositions.lines.forEach((startPos, id) => {
				const newStartX = startPos.start.x + dragOffset.x;
				const newStartY = startPos.start.y + dragOffset.y;
				const newEndX = startPos.end.x + dragOffset.x;
				const newEndY = startPos.end.y + dragOffset.y;
				moveLine(id, newStartX, newStartY, newEndX, newEndY, false);
			});
			
			selectedShapesStartPositions.arrows.forEach((startPos, id) => {
				const newStartX = startPos.start.x + dragOffset.x;
				const newStartY = startPos.start.y + dragOffset.y;
				const newEndX = startPos.end.x + dragOffset.x;
				const newEndY = startPos.end.y + dragOffset.y;
				moveArrow(id, newStartX, newStartY, newEndX, newEndY, false);
			});
			
			$editorApi.save_snapshot();
			scheduleRender();
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
					selectedDiamonds.set([]);
					selectedLines.set([]);
					selectedArrows.set([]);
					if ($rectangles.length > 0) {
						const newestRect = $rectangles[$rectangles.length - 1];
						selectedRectangles.set([newestRect]);
					}
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
					selectedDiamonds.set([]);
					selectedLines.set([]);
					selectedArrows.set([]);
					if ($ellipses.length > 0) {
						const newestEllipse = $ellipses[$ellipses.length - 1];
						selectedEllipses.set([newestEllipse]);
					}
					activeTool.set('select' as Tool);
				}
			} else if ($activeTool === 'diamond') {
				const width = Math.abs(createCurrentPos.x - createStartPos.x);
				const height = Math.abs(createCurrentPos.y - createStartPos.y);
				
				if (width > threshold && height > threshold) {
					const x = Math.min(createStartPos.x, createCurrentPos.x);
					const y = Math.min(createStartPos.y, createCurrentPos.y);
					addDiamond(x, y, width, height);
					selectedRectangles.set([]);
					selectedEllipses.set([]);
					selectedLines.set([]);
					selectedArrows.set([]);
					if ($diamonds.length > 0) {
						const newestDiamond = $diamonds[$diamonds.length - 1];
						selectedDiamonds.set([newestDiamond]);
					}
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
						selectedDiamonds.set([]);
						selectedArrows.set([]);
						if ($lines.length > 0) {
							const newestLine = $lines[$lines.length - 1];
							selectedLines.set([newestLine]);
						}
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
						selectedDiamonds.set([]);
						selectedLines.set([]);
						if ($arrows.length > 0) {
							const newestArrow = $arrows[$arrows.length - 1];
							selectedArrows.set([newestArrow]);
						}
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
		dragOffset = { x: 0, y: 0 };
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
		
		const handles = getHandlePositions({ x: boxX, y: boxY, width: boxWidth, height: boxHeight }).slice(0, 4);
		
		handles.forEach((handle) => {
			ctx.beginPath();
			ctx.rect(handle.x - halfHandle, handle.y - halfHandle, handleSize, handleSize);
			ctx.fill();
			ctx.stroke();
		});
	}

	function renderSelectionOutline(
		ctx: CanvasRenderingContext2D,
		x: number,
		y: number,
		width: number,
		height: number,
		zoom: number,
		includePadding: boolean = true
	) {
		ctx.save();
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 1 / zoom;
		const padding = includePadding ? getSelectionPaddingValue(zoom) : 0;
		ctx.strokeRect(x - padding, y - padding, width + padding * 2, height + padding * 2);
		ctx.restore();
	}

	function calculateGroupBoundingBox(): { x: number; y: number; width: number; height: number } | null {
		const allSelectedShapes: Array<{ minX: number; minY: number; maxX: number; maxY: number }> = [];
		
		$selectedRectangles.forEach(rect => {
			const isDragged = isDragging && selectedShapesStartPositions.rectangles.has(rect.id);
			let x: number, y: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.rectangles.get(rect.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
			} else {
				x = rect.position.x;
				y = rect.position.y;
			}
			allSelectedShapes.push({
				minX: x,
				minY: y,
				maxX: x + rect.width,
				maxY: y + rect.height
			});
		});
		
		$selectedEllipses.forEach(ellipse => {
			const isDragged = isDragging && selectedShapesStartPositions.ellipses.has(ellipse.id);
			let x: number, y: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.ellipses.get(ellipse.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
			} else {
				x = ellipse.position.x;
				y = ellipse.position.y;
			}
			allSelectedShapes.push({
				minX: x - ellipse.radius_x,
				minY: y - ellipse.radius_y,
				maxX: x + ellipse.radius_x,
				maxY: y + ellipse.radius_y
			});
		});
		
		$selectedDiamonds.forEach(diamond => {
			const isDragged = isDragging && selectedShapesStartPositions.diamonds.has(diamond.id);
			let x: number, y: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.diamonds.get(diamond.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
			} else {
				x = diamond.position.x;
				y = diamond.position.y;
			}
			allSelectedShapes.push({
				minX: x,
				minY: y,
				maxX: x + diamond.width,
				maxY: y + diamond.height
			});
		});
		
		$selectedLines.forEach(line => {
			const isDragged = isDragging && selectedShapesStartPositions.lines.has(line.id);
			let startX: number, startY: number, endX: number, endY: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.lines.get(line.id)!;
				startX = startPos.start.x + dragOffset.x;
				startY = startPos.start.y + dragOffset.y;
				endX = startPos.end.x + dragOffset.x;
				endY = startPos.end.y + dragOffset.y;
			} else {
				startX = line.start.x;
				startY = line.start.y;
				endX = line.end.x;
				endY = line.end.y;
			}
			allSelectedShapes.push({
				minX: Math.min(startX, endX),
				minY: Math.min(startY, endY),
				maxX: Math.max(startX, endX),
				maxY: Math.max(startY, endY)
			});
		});
		
		$selectedArrows.forEach(arrow => {
			const isDragged = isDragging && selectedShapesStartPositions.arrows.has(arrow.id);
			let startX: number, startY: number, endX: number, endY: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.arrows.get(arrow.id)!;
				startX = startPos.start.x + dragOffset.x;
				startY = startPos.start.y + dragOffset.y;
				endX = startPos.end.x + dragOffset.x;
				endY = startPos.end.y + dragOffset.y;
			} else {
				startX = arrow.start.x;
				startY = arrow.start.y;
				endX = arrow.end.x;
				endY = arrow.end.y;
			}
			allSelectedShapes.push({
				minX: Math.min(startX, endX),
				minY: Math.min(startY, endY),
				maxX: Math.max(startX, endX),
				maxY: Math.max(startY, endY)
			});
		});
		
		if (allSelectedShapes.length === 0) return null;
		if (allSelectedShapes.length === 1) return null;
		
		const minX = Math.min(...allSelectedShapes.map(s => s.minX));
		const minY = Math.min(...allSelectedShapes.map(s => s.minY));
		const maxX = Math.max(...allSelectedShapes.map(s => s.maxX));
		const maxY = Math.max(...allSelectedShapes.map(s => s.maxY));
		
		return {
			x: minX,
			y: minY,
			width: maxX - minX,
			height: maxY - minY
		};
	}

function renderGroupBoundingBox(ctx: CanvasRenderingContext2D, box: BoundingBox, zoom: number) {
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 1 / zoom;
		ctx.setLineDash([5 / zoom, 5 / zoom]);
		ctx.strokeRect(box.x, box.y, box.width, box.height);
		ctx.setLineDash([]);
	renderGroupResizeHandles(ctx, box, zoom);
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
		const isCreatingDiamond = $activeTool === 'diamond' && isCreatingShape;
		const totalSelectionCount =
			$selectedRectangles.length +
			$selectedEllipses.length +
			$selectedDiamonds.length +
			$selectedLines.length +
			$selectedArrows.length;
		const showIndividualHandles = totalSelectionCount <= 1;
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$rectangles.forEach((rect) => {
			const isSelected = $selectedRectangles.some(selected => selected.id === rect.id);
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.rectangles.has(rect.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'rectangle' && resizePreview.id === rect.id;
			
			let renderX: number, renderY: number;
			if (isResized && resizePreview) {
				renderX = resizePreview.x;
				renderY = resizePreview.y;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.rectangles.get(rect.id)!;
				renderX = startPos.x + dragOffset.x;
				renderY = startPos.y + dragOffset.y;
			} else {
				renderX = rect.position.x;
				renderY = rect.position.y;
			}
			const renderWidth = isResized && resizePreview ? resizePreview.width : rect.width;
			const renderHeight = isResized && resizePreview ? resizePreview.height : rect.height;
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.strokeRect(renderX, renderY, renderWidth, renderHeight);
			
			if (isSelected) {
				if (showIndividualHandles) {
					renderResizeHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom);
				} else {
					renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom);
				}
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
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.ellipses.has(ellipse.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'ellipse' && resizePreview.id === ellipse.id;
			
			let renderX: number, renderY: number;
			if (isResized && resizePreview) {
				renderX = resizePreview.x;
				renderY = resizePreview.y;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.ellipses.get(ellipse.id)!;
				renderX = startPos.x + dragOffset.x;
				renderY = startPos.y + dragOffset.y;
			} else {
				renderX = ellipse.position.x;
				renderY = ellipse.position.y;
			}
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
				if (showIndividualHandles) {
					renderResizeHandles(renderCtx, x, y, width, height, $zoom);
				} else {
					renderSelectionOutline(renderCtx, x, y, width, height, $zoom);
				}
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
		
		renderCtx.restore();
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$diamonds.forEach((diamond) => {
			const isSelected = $selectedDiamonds.some(selected => selected.id === diamond.id);
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.diamonds.has(diamond.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'diamond' && resizePreview.id === diamond.id;
			
			let renderX: number, renderY: number;
			if (isResized && resizePreview) {
				renderX = resizePreview.x;
				renderY = resizePreview.y;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.diamonds.get(diamond.id)!;
				renderX = startPos.x + dragOffset.x;
				renderY = startPos.y + dragOffset.y;
			} else {
				renderX = diamond.position.x;
				renderY = diamond.position.y;
			}
			const renderWidth = isResized && resizePreview ? resizePreview.width : diamond.width;
			const renderHeight = isResized && resizePreview ? resizePreview.height : diamond.height;
			
			const centerX = renderX + renderWidth / 2;
			const centerY = renderY + renderHeight / 2;
			const halfWidth = renderWidth / 2;
			const halfHeight = renderHeight / 2;
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.beginPath();
			renderCtx.moveTo(centerX, centerY - halfHeight);
			renderCtx.lineTo(centerX + halfWidth, centerY);
			renderCtx.lineTo(centerX, centerY + halfHeight);
			renderCtx.lineTo(centerX - halfWidth, centerY);
			renderCtx.closePath();
			renderCtx.stroke();
			
			if (isSelected) {
				if (showIndividualHandles) {
					renderResizeHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom);
				} else {
					renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom);
				}
			}
		});
		
		if (isCreatingDiamond && previewRect && previewRect.width > 0 && previewRect.height > 0) {
			const centerX = previewRect.x + previewRect.width / 2;
			const centerY = previewRect.y + previewRect.height / 2;
			const halfWidth = previewRect.width / 2;
			const halfHeight = previewRect.height / 2;
			
			renderCtx.strokeStyle = '#000000';
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			renderCtx.moveTo(centerX, centerY - halfHeight);
			renderCtx.lineTo(centerX + halfWidth, centerY);
			renderCtx.lineTo(centerX, centerY + halfHeight);
			renderCtx.lineTo(centerX - halfWidth, centerY);
			renderCtx.closePath();
			renderCtx.stroke();
			renderCtx.globalAlpha = 1.0;
		}
		
		renderCtx.restore();
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		$lines.forEach((line: Line) => {
			const isSelected = $selectedLines.some(selected => selected.id === line.id);
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.lines.has(line.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'line' && resizePreview.id === line.id;
			
			let renderStartX: number, renderStartY: number, renderEndX: number, renderEndY: number;
			
			if (isResized && resizePreview) {
				renderStartX = resizePreview.x;
				renderStartY = resizePreview.y;
				renderEndX = resizePreview.x + resizePreview.width;
				renderEndY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.lines.get(line.id)!;
				renderStartX = startPos.start.x + dragOffset.x;
				renderStartY = startPos.start.y + dragOffset.y;
				renderEndX = startPos.end.x + dragOffset.x;
				renderEndY = startPos.end.y + dragOffset.y;
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
			
			if (isSelected && showIndividualHandles) {
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
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.arrows.has(arrow.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'arrow' && resizePreview.id === arrow.id;
			
			let renderStartX: number, renderStartY: number, renderEndX: number, renderEndY: number;
			
			if (isResized && resizePreview) {
				renderStartX = resizePreview.x;
				renderStartY = resizePreview.y;
				renderEndX = resizePreview.x + resizePreview.width;
				renderEndY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.arrows.get(arrow.id)!;
				renderStartX = startPos.start.x + dragOffset.x;
				renderStartY = startPos.start.y + dragOffset.y;
				renderEndX = startPos.end.x + dragOffset.x;
				renderEndY = startPos.end.y + dragOffset.y;
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
			const arrowLength = 15;
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
			
			if (isSelected && showIndividualHandles) {
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
			const arrowLength = 15;
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
		
		const groupBoundingBoxRaw =
			totalSelectionCount > 1 ? groupResizeCurrentBox ?? calculateGroupBoundingBox() : null;
		const groupBoundingBox =
			groupBoundingBoxRaw && totalSelectionCount > 1
				? expandBoundingBox(groupBoundingBoxRaw, getSelectionPaddingValue($zoom))
				: null;
		if (groupBoundingBox) {
			renderGroupBoundingBox(renderCtx, groupBoundingBox, $zoom);
		}
		
		if (isSelectingBox && selectionBoxStart && selectionBoxEnd) {
			const boxX = Math.min(selectionBoxStart.x, selectionBoxEnd.x);
			const boxY = Math.min(selectionBoxStart.y, selectionBoxEnd.y);
			const boxWidth = Math.abs(selectionBoxEnd.x - selectionBoxStart.x);
			const boxHeight = Math.abs(selectionBoxEnd.y - selectionBoxStart.y);
			
			renderCtx.strokeStyle = '#1e88e5';
			renderCtx.lineWidth = 1 / $zoom;
			renderCtx.setLineDash([5 / $zoom, 5 / $zoom]);
			renderCtx.strokeRect(boxX, boxY, boxWidth, boxHeight);
			renderCtx.setLineDash([]);
			
			renderCtx.fillStyle = 'rgba(30, 136, 229, 0.1)';
			renderCtx.fillRect(boxX, boxY, boxWidth, boxHeight);
		}
		
		renderCtx.restore();
	}

	function scheduleRender() {
		if (renderRequestId !== null) {
			return;
		}
		renderRequestId = requestAnimationFrame(() => {
			renderRequestId = null;
			render();
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
	$: renderDependencies = {
		viewportOffset: $viewportOffset,
		zoomValue: $zoom,
		rectangles: $rectangles,
		selectedRectangles: $selectedRectangles,
		ellipses: $ellipses,
		selectedEllipses: $selectedEllipses,
		lines: $lines,
		selectedLines: $selectedLines,
		arrows: $arrows,
		selectedArrows: $selectedArrows,
		diamonds: $diamonds,
		selectedDiamonds: $selectedDiamonds
	};

	$: if (ctx && canvas && !isCreatingShape && renderDependencies) {
		scheduleRender();
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
