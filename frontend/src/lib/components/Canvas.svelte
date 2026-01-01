<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		lines, selectedLines, arrows, selectedArrows,
		diamonds, selectedDiamonds,
		paths, selectedPaths,
		images, selectedImages,
		texts, selectedTexts,
		groups, selectedGroups,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Path, type Image, type Text, type Group
	} from '$lib/stores/editor';

	import { isPointInRectangle, isPointInEllipse, isPointOnLine, isPointOnPath, isPointInDiamond, isPointInImage, rectangleIntersectsBox, ellipseIntersectsBox, lineIntersectsBox, arrowIntersectsBox, diamondIntersectsBox, pathIntersectsBox, imageIntersectsBox, getPathBoundingBox } from '$lib/utils/geometry';
	import { screenToWorld } from '$lib/utils/viewport';
	import { 
		addRectangle, moveRectangle, resizeRectangle, setRectangleRotation,
		addEllipse, moveEllipse, resizeEllipse, setEllipseRotation,
		addDiamond, moveDiamond, resizeDiamond, setDiamondRotation,
		addLine, moveLine,
		addArrow, moveArrow,
		addPath, movePath, resizePath, setPathRotation, setPathPoints,
		moveImage, resizeImage, setImageRotation
	} from '$lib/utils/canvas-operations/index';
	import { updatePaths } from '$lib/utils/canvas-operations/path';
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
	// @ts-ignore
	import StylePanel from './StylePanel.svelte';
	import Sidebar from './Sidebar.svelte';

	let sidebarRef: Sidebar;
	import { activeTool, type Tool } from '$lib/stores/tools';
	import { theme } from '$lib/stores/theme';

	function getDefaultStrokeColor(): string {
		return $theme === 'dark' ? '#ffffff' : '#000000';
	}


	function getHandleFillColor(): string {
		return $theme === 'dark' ? '#1c1917' : '#ffffff';
	}

	function getHandleStrokeColor(): string {
		return '#1e88e5';
	}

	function getSelectionOutlineColor(): string {
		return '#1e88e5';
	}

	function adaptColorToTheme(color: string | null | undefined, defaultColor: string): string {
		if (!color) return defaultColor;
		const normalizedColor = color.toLowerCase().trim();
		if (normalizedColor === '#000000' || normalizedColor === 'black' || normalizedColor === 'rgb(0, 0, 0)') {
			return $theme === 'dark' ? '#ffffff' : '#000000';
		}
		if (normalizedColor === '#ffffff' || normalizedColor === 'white' || normalizedColor === 'rgb(255, 255, 255)') {
			return $theme === 'dark' ? '#ffffff' : '#000000';
		}
		return color;
	}

	type BoundingBox = { x: number; y: number; width: number; height: number; rawWidth?: number; rawHeight?: number; scaleX?: number; scaleY?: number };
	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let draggedShape: Rectangle | Ellipse | Line | Arrow | Diamond | Path | Image | null = null;
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
	let freehandPoints: Array<{ x: number; y: number }> = [];
	let isDrawingFreehand = false;
	let isErasing = false;
	let eraserRadius = 5;
	let eraserPosition: { x: number; y: number } | null = null;
	let eraserShadowPosition: { x: number; y: number } | null = null;
	let renderRequestId: number | null = null;
	let isResizing = false;
	let resizeHandleIndex: number | null = null;
	let resizeStartShape: Rectangle | Ellipse | Line | Arrow | Diamond | Image | Path | null = null;
	let resizeStartShapeType: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | 'image' | 'path' | null = null;
	let resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
	let resizeStartMousePos = { x: 0, y: 0 };
	let isShiftPressedDuringResize = false;
	let dragOffset = { x: 0, y: 0 };
	let resizePreview: { x: number; y: number; width: number; height: number; type: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | 'image' | 'path'; id: number } | null = null;
	let lastMouseWorldPos: { x: number; y: number } | null = null;
	let isSelectingBox = false;
	let selectionBoxStart: { x: number; y: number } | null = null;
	let selectionBoxEnd: { x: number; y: number } | null = null;
	let selectedShapesStartPositions: {
		rectangles: Map<number, { x: number; y: number; width: number; height: number; rotation: number }>;
		ellipses: Map<number, { x: number; y: number; radius_x: number; radius_y: number; rotation: number }>;
		diamonds: Map<number, { x: number; y: number; width: number; height: number; rotation: number }>;
		lines: Map<number, { start: { x: number; y: number }; end: { x: number; y: number }; rotation: number }>;
		arrows: Map<number, { start: { x: number; y: number }; end: { x: number; y: number }; rotation: number }>;
		texts: Map<number, { x: number; y: number; width: number; height: number; rotation: number }>;
		paths: Map<number, { points: Array<{ x: number; y: number }>; rotation: number }>;
		images: Map<number, { x: number; y: number; width: number; height: number; rotation: number }>;
	} = {
		rectangles: new Map(),
		ellipses: new Map(),
		diamonds: new Map(),
		lines: new Map(),
		arrows: new Map(),
		texts: new Map(),
		paths: new Map(),
		images: new Map()
	};

	function storeSelectedShapesStartPositions() {
		selectedShapesStartPositions.rectangles.clear();
		selectedShapesStartPositions.ellipses.clear();
		selectedShapesStartPositions.diamonds.clear();
		selectedShapesStartPositions.lines.clear();
		selectedShapesStartPositions.arrows.clear();
		selectedShapesStartPositions.texts.clear();
		selectedShapesStartPositions.paths.clear();
		selectedShapesStartPositions.images.clear();
		
	$selectedRectangles.forEach(rect => {
		selectedShapesStartPositions.rectangles.set(rect.id, { x: rect.position.x, y: rect.position.y, width: rect.width, height: rect.height, rotation: rect.rotation_angle ?? 0 });
	});
		
		$selectedEllipses.forEach(ellipse => {
		selectedShapesStartPositions.ellipses.set(ellipse.id, { x: ellipse.position.x, y: ellipse.position.y, radius_x: ellipse.radius_x, radius_y: ellipse.radius_y, rotation: ellipse.rotation_angle ?? 0 });
		});
		
		$selectedDiamonds.forEach(diamond => {
		selectedShapesStartPositions.diamonds.set(diamond.id, { x: diamond.position.x, y: diamond.position.y, width: diamond.width, height: diamond.height, rotation: diamond.rotation_angle ?? 0 });
		});
		
		$selectedLines.forEach(line => {
		selectedShapesStartPositions.lines.set(line.id, { start: { x: line.start.x, y: line.start.y }, end: { x: line.end.x, y: line.end.y }, rotation: line.rotation_angle ?? 0 });
		});
		
		$selectedArrows.forEach(arrow => {
		selectedShapesStartPositions.arrows.set(arrow.id, { start: { x: arrow.start.x, y: arrow.start.y }, end: { x: arrow.end.x, y: arrow.end.y }, rotation: arrow.rotation_angle ?? 0 });
		});

		$selectedTexts.forEach(text => {
		selectedShapesStartPositions.texts.set(text.id, { x: text.position.x, y: text.position.y, width: text.width, height: text.height, rotation: text.rotation_angle ?? 0 });
		});

		$selectedPaths.forEach(path => {
		selectedShapesStartPositions.paths.set(path.id, { points: path.points.map(p => ({ x: p.x, y: p.y })), rotation: path.rotation_angle ?? 0 });
		});

		$selectedImages.forEach(image => {
		selectedShapesStartPositions.images.set(image.id, { x: image.position.x, y: image.position.y, width: image.width, height: image.height, rotation: image.rotation_angle ?? 0 });
		});
	}
	let isGroupResizing = false;
	let groupResizeHandleIndex: number | null = null;
	let groupResizeStartBox: BoundingBox | null = null;
	let groupResizeCurrentBox: BoundingBox | null = null;
	let groupResizePadding = 0;
	let groupResizeStartMousePos = { x: 0, y: 0 };
	let isGroupRotating = false;
	let groupRotationState: { center: { x: number; y: number }; startAngle: number; mouseStartAngle: number } | null = null;
	const imageCache = new Map<number, HTMLImageElement>();
	const resizeCursors = ['nwse-resize', 'nesw-resize', 'nwse-resize', 'nesw-resize', 'ns-resize', 'ew-resize', 'ns-resize', 'ew-resize'];
	const ROTATION_HANDLE_DISTANCE = 18;
	const ROTATION_HANDLE_RADIUS = 5;
	const ROTATION_STEP = Math.PI / 12;
	type RotatableShapeType = 'rectangle' | 'ellipse' | 'diamond' | 'image' | 'path';
	let isRotating = false;
	let rotationState: { type: RotatableShapeType; id: number; center: { x: number; y: number }; startAngle: number; mouseStartAngle: number } | null = null;
	let rotationPreview: { type: RotatableShapeType; id: number; angle: number } | null = null;



    function groupSelectedShapes() {
        if (!$editorApi) return;
        const selectedIds: number[] = [];
        $selectedRectangles.forEach(r => selectedIds.push(r.id));
        $selectedEllipses.forEach(e => selectedIds.push(e.id));
        $selectedDiamonds.forEach(d => selectedIds.push(d.id));
        $selectedLines.forEach(l => selectedIds.push(l.id));
        $selectedArrows.forEach(a => selectedIds.push(a.id));
        $selectedPaths.forEach(p => selectedIds.push(p.id));
        $selectedImages.forEach(i => selectedIds.push(i.id));

        if (selectedIds.length < 2) return;

        const groupId = $editorApi.group_elements(selectedIds);
        const updatedGroups = Array.from($editorApi.get_groups() as Group[]);
        groups.set(updatedGroups);

        clearAllSelections();
        const newGroup = updatedGroups.find(g => g.id === Number(groupId));
        if (newGroup) {
            selectGroup(newGroup, false);
        }
    }

    function ungroupSelectedGroups() {
        if (!$editorApi || $selectedGroups.length === 0) return;

        $selectedGroups.forEach(group => {
            $editorApi!.ungroup_elements(BigInt(group.id));
        });

        const updatedGroups = Array.from($editorApi.get_groups() as Group[]);
        groups.set(updatedGroups);
        selectedGroups.set([]);
        
        clearAllSelections();
    }

    function bringToFront() {
        if (!$editorApi) return;
        
        $selectedRectangles.forEach(r => $editorApi!.bring_shape_to_front(BigInt(r.id)));
        $selectedEllipses.forEach(e => $editorApi!.bring_shape_to_front(BigInt(e.id)));
        $selectedDiamonds.forEach(d => $editorApi!.bring_shape_to_front(BigInt(d.id)));
        $selectedLines.forEach(l => $editorApi!.bring_shape_to_front(BigInt(l.id)));
        $selectedArrows.forEach(a => $editorApi!.bring_shape_to_front(BigInt(a.id)));
        $selectedPaths.forEach(p => $editorApi!.bring_shape_to_front(BigInt(p.id)));
        $selectedImages.forEach(i => $editorApi!.bring_shape_to_front(BigInt(i.id)));
        
        updateAllStoresAfterUndoRedo();
        scheduleRender();
    }

    function bringForward() {
        if (!$editorApi) return;
        
        $selectedRectangles.forEach(r => $editorApi!.bring_shape_forward(BigInt(r.id)));
        $selectedEllipses.forEach(e => $editorApi!.bring_shape_forward(BigInt(e.id)));
        $selectedDiamonds.forEach(d => $editorApi!.bring_shape_forward(BigInt(d.id)));
        $selectedLines.forEach(l => $editorApi!.bring_shape_forward(BigInt(l.id)));
        $selectedArrows.forEach(a => $editorApi!.bring_shape_forward(BigInt(a.id)));
        $selectedPaths.forEach(p => $editorApi!.bring_shape_forward(BigInt(p.id)));
        $selectedImages.forEach(i => $editorApi!.bring_shape_forward(BigInt(i.id)));
        
        updateAllStoresAfterUndoRedo();
        scheduleRender();
    }

    function sendBackward() {
        if (!$editorApi) return;
        
        $selectedRectangles.forEach(r => $editorApi!.send_shape_backward(BigInt(r.id)));
        $selectedEllipses.forEach(e => $editorApi!.send_shape_backward(BigInt(e.id)));
        $selectedDiamonds.forEach(d => $editorApi!.send_shape_backward(BigInt(d.id)));
        $selectedLines.forEach(l => $editorApi!.send_shape_backward(BigInt(l.id)));
        $selectedArrows.forEach(a => $editorApi!.send_shape_backward(BigInt(a.id)));
        $selectedPaths.forEach(p => $editorApi!.send_shape_backward(BigInt(p.id)));
        $selectedImages.forEach(i => $editorApi!.send_shape_backward(BigInt(i.id)));
        
        updateAllStoresAfterUndoRedo();
        scheduleRender();
    }

    function sendToBack() {
        if (!$editorApi) return;
        
        $selectedRectangles.forEach(r => $editorApi!.send_shape_to_back(BigInt(r.id)));
        $selectedEllipses.forEach(e => $editorApi!.send_shape_to_back(BigInt(e.id)));
        $selectedDiamonds.forEach(d => $editorApi!.send_shape_to_back(BigInt(d.id)));
        $selectedLines.forEach(l => $editorApi!.send_shape_to_back(BigInt(l.id)));
        $selectedArrows.forEach(a => $editorApi!.send_shape_to_back(BigInt(a.id)));
        $selectedPaths.forEach(p => $editorApi!.send_shape_to_back(BigInt(p.id)));
        $selectedImages.forEach(i => $editorApi!.send_shape_to_back(BigInt(i.id)));
        
        updateAllStoresAfterUndoRedo();
        scheduleRender();
    }



	function getAllShapeIdsInGroup(group: Group): number[] {
		let ids: number[] = [];
		for (const id of group.element_ids) {
			const subGroup = $groups.find(g => g.id === id);
			if (subGroup) {
				ids = ids.concat(getAllShapeIdsInGroup(subGroup));
			} else {
				ids.push(id);
			}
		}
		return ids;
	}

	function findGroupForShape(shapeId: number): Group | null {
		let currentGroupId: number | null = null;
		
		for (const group of $groups) {
			if (group.element_ids.includes(shapeId)) {
				currentGroupId = group.id;
				break;
			}
		}

		if (currentGroupId === null) return null;

		while (true) {
			let parentGroup: Group | null = null;
			for (const group of $groups) {
				if (group.element_ids.includes(currentGroupId)) {
					parentGroup = group;
					break;
				}
			}
			if (parentGroup) {
				currentGroupId = parentGroup.id;
			} else {
				break;
			}
		}

		return $groups.find(g => g.id === currentGroupId) || null;
	}

	function selectGroup(group: Group, isShiftPressed: boolean) {
		const isAlreadySelected = $selectedGroups.some(g => g.id === group.id);
		let newSelectedGroups: Group[] = [];

		if (isShiftPressed) {
			newSelectedGroups = isAlreadySelected
				? $selectedGroups.filter(g => g.id !== group.id)
				: [...$selectedGroups, group];
		} else {
			newSelectedGroups = [group];
		}

		selectedGroups.set(newSelectedGroups);

		const allShapeIds = new Set<number>();
		newSelectedGroups.forEach(g => {
			getAllShapeIdsInGroup(g).forEach(id => allShapeIds.add(id));
		});

        if (isShiftPressed) {
        } else {
             selectedRectangles.set([]);
             selectedEllipses.set([]);
             selectedDiamonds.set([]);
             selectedLines.set([]);
             selectedArrows.set([]);
             selectedPaths.set([]);
             selectedImages.set([]);
        }

        const rects: Rectangle[] = [];
        const ells: Ellipse[] = [];
        const diams: Diamond[] = [];
        const lns: Line[] = [];
        const arrs: Arrow[] = [];
        const pths: Path[] = [];
        const imgs: Image[] = [];

        $rectangles.forEach(r => { if (allShapeIds.has(r.id)) rects.push(r); });
        $ellipses.forEach(e => { if (allShapeIds.has(e.id)) ells.push(e); });
        $diamonds.forEach(d => { if (allShapeIds.has(d.id)) diams.push(d); });
        $lines.forEach(l => { if (allShapeIds.has(l.id)) lns.push(l); });
        $arrows.forEach(a => { if (allShapeIds.has(a.id)) arrs.push(a); });
        $paths.forEach(p => { if (allShapeIds.has(p.id)) pths.push(p); });
        $images.forEach(i => { if (allShapeIds.has(i.id)) imgs.push(i); });

        if (isShiftPressed) {
             selectedRectangles.update(s => [...s, ...rects].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedEllipses.update(s => [...s, ...ells].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedDiamonds.update(s => [...s, ...diams].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedLines.update(s => [...s, ...lns].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedArrows.update(s => [...s, ...arrs].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedPaths.update(s => [...s, ...pths].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
             selectedImages.update(s => [...s, ...imgs].filter((v, i, a) => a.findIndex(t => t.id === v.id) === i));
        } else {
             selectedRectangles.set(rects);
             selectedEllipses.set(ells);
             selectedDiamonds.set(diams);
             selectedLines.set(lns);
             selectedArrows.set(arrs);
             selectedPaths.set(pths);
             selectedImages.set(imgs);
        }
	}


	function handleKeyDown(event: KeyboardEvent) {

		if (event.key === 'Escape' && isRotating) {
			event.preventDefault();
			resetRotationState();
			scheduleRender();
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'g') {
			event.preventDefault();
			if (event.shiftKey) {
				ungroupSelectedGroups();
			} else {
				groupSelectedShapes();
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === ']' || event.key === '}')) {
			event.preventDefault();
			if (event.shiftKey) {
				bringToFront();
			} else {
				bringForward();
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === '[' || event.key === '{')) {
			event.preventDefault();
			if (event.shiftKey) {
				sendToBack();
			} else {
				sendBackward();
			}
			return;
		}

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
			const isPlus = event.key === '+' || event.key === '=';
			

			if (isPlus) {
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
			clearAllSelections();
			scheduleRender();
			if ($activeTool === 'freehand') {
				finishFreehandDrawing();
				activeTool.set('select');
				return;
			}
			if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond' || $activeTool === 'line' || $activeTool === 'arrow' || $activeTool === 'text' || $activeTool === 'eraser') {
				activeTool.set('select');
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'c' || event.key === 'C')) {
			event.preventDefault();
			if ($selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0 || $selectedTexts.length > 0) {
				copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds, $selectedImages, $selectedTexts, $selectedPaths);
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'd') {
			event.preventDefault();
			event.stopPropagation();
			if (!$editorApi) return;
			
			const hasSelection = $selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0 || $selectedPaths.length > 0 || $selectedImages.length > 0;
			if (!hasSelection) return;
			
			const selectedIds = new Set([
				...$selectedRectangles.map(r => r.id),
				...$selectedEllipses.map(e => e.id),
				...$selectedLines.map(l => l.id),
				...$selectedArrows.map(a => a.id),
				...$selectedDiamonds.map(d => d.id),
				...$selectedPaths.map(p => p.id),
				...$selectedImages.map(i => i.id)
			]);
			
			const allRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
			const allEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
			const allLines = Array.from($editorApi.get_lines() as Line[]);
			const allArrows = Array.from($editorApi.get_arrows() as Arrow[]);
			const allDiamonds = Array.from($editorApi.get_diamonds() as Diamond[]);
			const allTexts = [...$texts];
			const allPaths = Array.from($editorApi.get_paths() as Path[]);
			const allImages = Array.from($editorApi.get_images() as Image[]);
			
			const currentRectangles = allRectangles.filter(r => selectedIds.has(r.id));
			const currentEllipses = allEllipses.filter(e => selectedIds.has(e.id));
			const currentLines = allLines.filter(l => selectedIds.has(l.id));
			const currentArrows = allArrows.filter(a => selectedIds.has(a.id));
			const currentDiamonds = allDiamonds.filter(d => selectedIds.has(d.id));
			const currentTexts = allTexts.filter((t: Text) => selectedIds.has(t.id));
			const currentPaths = allPaths.filter(p => selectedIds.has(p.id));
			const currentImages = allImages.filter(i => selectedIds.has(i.id));
			
			copyToClipboard(currentRectangles, currentEllipses, currentLines, currentArrows, currentDiamonds, currentImages, currentTexts, currentPaths);
			const clipboard = getClipboard();
			
			const bounds: Array<{ minX: number; minY: number }> = [];
			clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
			clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
			clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
			clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
			clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
			clipboard.images.forEach(i => bounds.push({ minX: i.position.x, minY: i.position.y }));
			clipboard.paths.forEach(p => {
				if (p.points.length > 0) {
					const minX = Math.min(...p.points.map(pt => pt.x));
					const minY = Math.min(...p.points.map(pt => pt.y));
					bounds.push({ minX, minY });
				}
			});

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
			selectedLines.set([...$lines]);
			selectedArrows.set([...$arrows]);
			selectedDiamonds.set([...$diamonds]);
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

		if (!event.ctrlKey && !event.metaKey && !event.altKey && event.key.toLowerCase() === 'r') {
			if ($activeTool === 'select') {
				event.preventDefault();
				const delta = event.shiftKey ? -ROTATION_STEP : ROTATION_STEP;
				rotateSelectedShapes(delta);
			}
			return;
		}

		if (!$editorApi || event.key !== 'Delete') return;

		const hasSelectedRectangles = $selectedRectangles.length > 0;
		const hasSelectedEllipses = $selectedEllipses.length > 0;
		const hasSelectedDiamonds = $selectedDiamonds.length > 0;
		const hasSelectedLines = $selectedLines.length > 0;
		const hasSelectedArrows = $selectedArrows.length > 0;
		const hasSelectedPaths = $selectedPaths.length > 0;
		const hasSelectedImages = $selectedImages.length > 0;

		if (!hasSelectedRectangles && !hasSelectedEllipses && !hasSelectedDiamonds && !hasSelectedLines && !hasSelectedArrows && !hasSelectedPaths && !hasSelectedImages) return;

		event.preventDefault();
		
		const rectangleIds = hasSelectedRectangles ? $selectedRectangles.map(rect => rect.id) : [];
		const ellipseIds = hasSelectedEllipses ? $selectedEllipses.map(ellipse => ellipse.id) : [];
		const diamondIds = hasSelectedDiamonds ? $selectedDiamonds.map(diamond => diamond.id) : [];
		const lineIds = hasSelectedLines ? $selectedLines.map(line => line.id) : [];
		const arrowIds = hasSelectedArrows ? $selectedArrows.map(arrow => arrow.id) : [];
		const pathIds = hasSelectedPaths ? $selectedPaths.map(path => path.id) : [];
		const imageIds = hasSelectedImages ? $selectedImages.map(image => image.id) : [];
		
		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, [], pathIds, imageIds);
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.key === ' ') {
			isSpacePressed = false;
		}
	}

	function getResizeHandleAt(
		x: number,
		y: number,
		shape: Rectangle | Ellipse | Diamond | Image,
		shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'image',
		zoom: number,
		rotation: number = 0
	): number | null {
		const padding = getSelectionPaddingValue(zoom);
		const gap = 4 / zoom;
		
		let boxX: number, boxY: number, boxWidth: number, boxHeight: number;
		
		if (shapeType === 'rectangle') {
			const rect = shape as Rectangle;
			boxX = rect.position.x - padding - gap;
			boxY = rect.position.y - padding - gap;
			boxWidth = rect.width + (padding + gap) * 2;
			boxHeight = rect.height + (padding + gap) * 2;
		} else if (shapeType === 'diamond') {
			const diamond = shape as Diamond;
			boxX = diamond.position.x - padding - gap;
			boxY = diamond.position.y - padding - gap;
			boxWidth = diamond.width + (padding + gap) * 2;
			boxHeight = diamond.height + (padding + gap) * 2;
		} else if (shapeType === 'image') {
			const image = shape as Image;
			boxX = image.position.x - padding - gap;
			boxY = image.position.y - padding - gap;
			boxWidth = image.width + (padding + gap) * 2;
			boxHeight = image.height + (padding + gap) * 2;
		} else {
			const ellipse = shape as Ellipse;
			const x = ellipse.position.x - ellipse.radius_x;
			const y = ellipse.position.y - ellipse.radius_y;
			const width = ellipse.radius_x * 2;
			const height = ellipse.radius_y * 2;
			boxX = x - padding - gap;
			boxY = y - padding - gap;
			boxWidth = width + (padding + gap) * 2;
			boxHeight = height + (padding + gap) * 2;
		}
		
		const bounds = { x: boxX, y: boxY, width: boxWidth, height: boxHeight };
		let testPoint = { x, y };
		if (Math.abs(rotation) > 0.0001) {
			const center = { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
			testPoint = rotatePointAround({ x, y }, center, -rotation);
		}
		
		return hitTestEdges(
			testPoint.x,
			testPoint.y,
			bounds,
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

function getShapeBoundingBox(
	shape: Rectangle | Ellipse | Diamond | Image | Path,
	shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'image' | 'path',
	ctxOverride: CanvasRenderingContext2D | null = ctx
): { x: number; y: number; width: number; height: number } | null {
	if (shapeType === 'rectangle') {
		const rect = shape as Rectangle;
		return { x: rect.position.x, y: rect.position.y, width: rect.width, height: rect.height };
	}
	if (shapeType === 'diamond') {
		const diamond = shape as Diamond;
		return { x: diamond.position.x, y: diamond.position.y, width: diamond.width, height: diamond.height };
	}
	if (shapeType === 'ellipse') {
		const ellipse = shape as Ellipse;
		return {
			x: ellipse.position.x - ellipse.radius_x,
			y: ellipse.position.y - ellipse.radius_y,
			width: ellipse.radius_x * 2,
			height: ellipse.radius_y * 2
		};
	}
	if (shapeType === 'image') {
		const image = shape as Image;
		return { x: image.position.x, y: image.position.y, width: image.width, height: image.height };
	}
	if (shapeType === 'path') {
		const path = shape as Path;
		return getPathBoundingBox(path);
	}
	return null;
}

function getSelectionBoundsForShape(
	shape: Rectangle | Ellipse | Diamond | Image | Path,
	shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'image' | 'path',
	zoom: number,
	ctxOverride: CanvasRenderingContext2D | null = ctx
): { x: number; y: number; width: number; height: number } | null {
	const base = getShapeBoundingBox(shape, shapeType, ctxOverride);
	if (!base) return null;
	const padding = getSelectionPaddingValue(zoom);
	const gap = 4 / zoom;
	return {
		x: base.x - padding - gap,
		y: base.y - padding - gap,
		width: base.width + (padding + gap) * 2,
		height: base.height + (padding + gap) * 2
	};
}

function getShapeCenter(
	shape: Rectangle | Ellipse | Diamond | Image | Path,
	shapeType: RotatableShapeType,
	ctxOverride: CanvasRenderingContext2D | null = ctx
): { x: number; y: number } | null {
	const bounds = getShapeBoundingBox(shape, shapeType, ctxOverride);
	if (!bounds) return null;
	return { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
}

function getShapeRotation(shape: Rectangle | Ellipse | Diamond | Image | Path, shapeType: RotatableShapeType): number {
	if (shapeType === 'rectangle') {
		return (shape as Rectangle).rotation_angle ?? 0;
	}
	if (shapeType === 'ellipse') {
		return (shape as Ellipse).rotation_angle ?? 0;
	}
	if (shapeType === 'diamond') {
		return (shape as Diamond).rotation_angle ?? 0;
	}
	if (shapeType === 'image') {
		return (shape as Image).rotation_angle ?? 0;
	}
	if (shapeType === 'path') {
		return (shape as Path).rotation_angle ?? 0;
	}
	return 0;
}

	function getRotationHandlePoints(
		bounds: { x: number; y: number; width: number; height: number },
		zoom: number,
		rotation: number = 0
	) {
		const center = { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
		const halfHeight = bounds.height / 2;
		const offset = ROTATION_HANDLE_DISTANCE / zoom;
		const basePoint = { x: center.x, y: center.y - halfHeight };
		const handlePoint = { x: center.x, y: center.y - halfHeight - offset };
		const rotatedBase = rotatePointAround(basePoint, center, rotation);
		const rotatedHandle = rotatePointAround(handlePoint, center, rotation);
		return { base: rotatedBase, handle: rotatedHandle };
	}

	function renderRotationHandleFromBounds(
		ctx: CanvasRenderingContext2D,
		bounds: { x: number; y: number; width: number; height: number },
		zoom: number,
		rotation: number = 0
	) {
		const { handle } = getRotationHandlePoints(bounds, zoom, rotation);
		const radius = ROTATION_HANDLE_RADIUS / zoom;
		ctx.save();
		ctx.fillStyle = getHandleFillColor();
		ctx.beginPath();
		ctx.arc(handle.x, handle.y, radius, 0, Math.PI * 2);
		ctx.fill();
		ctx.strokeStyle = getHandleStrokeColor();
		ctx.lineWidth = 1 / zoom;
		ctx.stroke();
		ctx.restore();
	}

	function isPointOnRotationHandle(
		x: number,
		y: number,
		shape: Rectangle | Ellipse | Diamond | Image | Path,
		shapeType: RotatableShapeType,
		zoom: number,
		ctxOverride: CanvasRenderingContext2D | null = ctx
	): boolean {
		const bounds = getSelectionBoundsForShape(shape, shapeType, zoom, ctxOverride);
		if (!bounds) return false;
		const rotation = getRenderedRotation(shape, shapeType);
		const { handle } = getRotationHandlePoints(bounds, zoom, rotation);
		const radius = ROTATION_HANDLE_RADIUS / zoom;
		return Math.hypot(x - handle.x, y - handle.y) <= radius;
	}

function normalizeAngle(angle: number): number {
	const twoPi = Math.PI * 2;
	let result = angle % twoPi;
	if (result > Math.PI) result -= twoPi;
	if (result < -Math.PI) result += twoPi;
	return result;
}

function getRenderedRotation(
	shape: Rectangle | Ellipse | Diamond | Image | Path,
	shapeType: RotatableShapeType
): number {
	if (rotationPreview && rotationPreview.type === shapeType && rotationPreview.id === shape.id) {
		return rotationPreview.angle;
	}
	return getShapeRotation(shape, shapeType);
}

function applyRotationToShape(type: RotatableShapeType, id: number, angle: number, saveHistory: boolean) {
	const normalized = normalizeAngle(angle);
	if (type === 'rectangle') {
		setRectangleRotation(id, normalized, saveHistory);
		return;
	}
	if (type === 'ellipse') {
		setEllipseRotation(id, normalized, saveHistory);
		return;
	}
	if (type === 'diamond') {
		setDiamondRotation(id, normalized, saveHistory);
		return;
	}
	if (type === 'image') {
		setImageRotation(id, normalized, saveHistory);
		return;
	}
	if (type === 'path') {
		setPathRotation(id, normalized, saveHistory);
		return;
	}
}

function beginRotation(
	shape: Rectangle | Ellipse | Diamond | Image | Path,
	shapeType: RotatableShapeType,
	x: number,
	y: number
): boolean {
	if (isRotating || !canvas) return false;
	const center = getShapeCenter(shape, shapeType, ctx);
	if (!center) return false;
	const startAngle = getShapeRotation(shape, shapeType);
	const mouseAngle = Math.atan2(y - center.y, x - center.x);
	isRotating = true;
	rotationState = { type: shapeType, id: shape.id, center, startAngle, mouseStartAngle: mouseAngle };
	rotationPreview = { type: shapeType, id: shape.id, angle: startAngle };
	canvas.style.cursor = 'grabbing';
	return true;
}

function resetRotationState() {
	isRotating = false;
	rotationState = null;
	rotationPreview = null;
}

	function isPointOnGroupRotationHandle(
		x: number,
		y: number,
		box: BoundingBox,
		zoom: number
	): boolean {
		const gap = 4 / zoom;
		const outlineX = box.x - gap;
		const outlineY = box.y - gap;
		const outlineWidth = box.width + gap * 2;
		const outlineHeight = box.height + gap * 2;
		const { handle } = getRotationHandlePoints({ x: outlineX, y: outlineY, width: outlineWidth, height: outlineHeight }, zoom, 0);
		const radius = ROTATION_HANDLE_RADIUS / zoom;
		return Math.hypot(x - handle.x, y - handle.y) <= radius;
	}

	function beginGroupRotation(
		box: BoundingBox,
		x: number,
		y: number
	): boolean {
		if (isGroupRotating || !canvas) return false;
		const center = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
		const mouseAngle = Math.atan2(y - center.y, x - center.x);
		isGroupRotating = true;
		groupRotationState = { center, startAngle: 0, mouseStartAngle: mouseAngle };
		storeSelectedShapesStartPositions();
		canvas.style.cursor = 'grabbing';
		return true;
	}

	function resetGroupRotationState() {
		isGroupRotating = false;
		groupRotationState = null;
	}

	function rotateSelectedShapes(delta: number) {
		const targets: Array<{ type: RotatableShapeType; shape: Rectangle | Ellipse | Diamond | Image }> = [];
		$selectedRectangles.forEach(rect => targets.push({ type: 'rectangle', shape: rect }));
		$selectedEllipses.forEach(ellipse => targets.push({ type: 'ellipse', shape: ellipse }));
		$selectedDiamonds.forEach(diamond => targets.push({ type: 'diamond', shape: diamond }));
		$selectedImages.forEach(image => targets.push({ type: 'image', shape: image }));
		if (targets.length === 0) return;
		targets.forEach((item, index) => {
			const currentAngle = getShapeRotation(item.shape, item.type);
			const newAngle = normalizeAngle(currentAngle + delta);
			const isLast = index === targets.length - 1;
			applyRotationToShape(item.type, item.shape.id, newAngle, isLast);
		});
		scheduleRender();
	}

	function getSelectionPaddingValue(currentZoom: number): number {
		return 2 / currentZoom;
	}

	function rotatePointAround(
		point: { x: number; y: number },
		center: { x: number; y: number },
		angle: number
	): { x: number; y: number } {
		if (angle === 0) return { x: point.x, y: point.y };
		const cos = Math.cos(angle);
		const sin = Math.sin(angle);
		const dx = point.x - center.x;
		const dy = point.y - center.y;
		return {
			x: center.x + dx * cos - dy * sin,
			y: center.y + dx * sin + dy * cos
		};
	}

	function worldToScreen(
		worldX: number,
		worldY: number,
		offset: { x: number; y: number },
		zoomLevel: number
	): { x: number; y: number } {
		return {
			x: worldX * zoomLevel + offset.x,
			y: worldY * zoomLevel + offset.y
		};
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
			const handle = handles[i];
			if (
				Math.abs(x - handle.x) <= halfHandle &&
				Math.abs(y - handle.y) <= halfHandle
			) {
				return i;
			}
		}

		return null;
	}

	function hitTestEdges(
		x: number,
		y: number,
		box: { x: number; y: number; width: number; height: number },
		zoom: number
	): number | null {
		const edgeTolerance = 8 / zoom;
		const { x: boxX, y: boxY, width: boxWidth, height: boxHeight } = box;
		const boxRight = boxX + boxWidth;
		const boxBottom = boxY + boxHeight;
		
		const onTopEdge = Math.abs(y - boxY) <= edgeTolerance && x >= boxX && x <= boxRight;
		const onRightEdge = Math.abs(x - boxRight) <= edgeTolerance && y >= boxY && y <= boxBottom;
		const onBottomEdge = Math.abs(y - boxBottom) <= edgeTolerance && x >= boxX && x <= boxRight;
		const onLeftEdge = Math.abs(x - boxX) <= edgeTolerance && y >= boxY && y <= boxBottom;
		
		const onTopLeftCorner = Math.abs(x - boxX) <= edgeTolerance && Math.abs(y - boxY) <= edgeTolerance;
		const onTopRightCorner = Math.abs(x - boxRight) <= edgeTolerance && Math.abs(y - boxY) <= edgeTolerance;
		const onBottomRightCorner = Math.abs(x - boxRight) <= edgeTolerance && Math.abs(y - boxBottom) <= edgeTolerance;
		const onBottomLeftCorner = Math.abs(x - boxX) <= edgeTolerance && Math.abs(y - boxBottom) <= edgeTolerance;
		
		if (onTopLeftCorner) return 0;
		if (onTopRightCorner) return 1;
		if (onBottomRightCorner) return 2;
		if (onBottomLeftCorner) return 3;
		if (onTopEdge) return 4;
		if (onRightEdge) return 5;
		if (onBottomEdge) return 6;
		if (onLeftEdge) return 7;
		
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

	function getPathResizeHandleAt(x: number, y: number, path: Path, zoom: number): number | null {
		const pathBounds = getPathBoundingBox(path);
		if (!pathBounds) return null;
		
		const padding = getSelectionPaddingValue(zoom);
		const gap = 4 / zoom;
		const boxX = pathBounds.x - padding - gap;
		const boxY = pathBounds.y - padding - gap;
		const boxWidth = pathBounds.width + (padding + gap) * 2;
		const boxHeight = pathBounds.height + (padding + gap) * 2;
		
		return hitTestEdges(x, y, { x: boxX, y: boxY, width: boxWidth, height: boxHeight }, zoom);
	}



	function getGroupResizeHandleAt(x: number, y: number, box: BoundingBox, zoom: number): number | null {
		const gap = 4 / zoom;
		const outlineX = box.x - gap;
		const outlineY = box.y - gap;
		const outlineWidth = box.width + gap * 2;
		const outlineHeight = box.height + gap * 2;
		return hitTestEdges(x, y, { x: outlineX, y: outlineY, width: outlineWidth, height: outlineHeight }, zoom);
	}

	function renderGroupResizeHandles(ctx: CanvasRenderingContext2D, box: BoundingBox, zoom: number) {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const gap = 4 / zoom;
		
		const outlineX = box.x - gap;
		const outlineY = box.y - gap;
		const outlineWidth = box.width + gap * 2;
		const outlineHeight = box.height + gap * 2;

		ctx.fillStyle = getHandleFillColor();
		ctx.strokeStyle = getHandleStrokeColor();
		ctx.lineWidth = 2 / zoom;

		const cornerHandles = [
			{ x: outlineX, y: outlineY },
			{ x: outlineX + outlineWidth, y: outlineY },
			{ x: outlineX + outlineWidth, y: outlineY + outlineHeight },
			{ x: outlineX, y: outlineY + outlineHeight }
		];

		cornerHandles.forEach(handle => {
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

		let newLeft = startLeft;
		let newRight = startRight;
		let newTop = startTop;
		let newBottom = startBottom;

		switch (groupResizeHandleIndex) {
			case 0:
				newLeft = targetX;
				newTop = targetY;
				break;
			case 1:
				newRight = targetX;
				newTop = targetY;
				break;
			case 2:
				newRight = targetX;
				newBottom = targetY;
				break;
			case 3:
				newLeft = targetX;
				newBottom = targetY;
				break;
			case 4:
				newTop = targetY;
				break;
			case 5:
				newRight = targetX;
				break;
			case 6:
				newBottom = targetY;
				break;
			case 7:
				newLeft = targetX;
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
		const rawScaleX = startBox.width === 0
			? (Math.sign(targetBox.rawWidth ?? targetBox.width) || 1)
			: (targetBox.rawWidth ?? targetBox.width) / startBox.width;
		const rawScaleY = startBox.height === 0
			? (Math.sign(targetBox.rawHeight ?? targetBox.height) || 1)
			: (targetBox.rawHeight ?? targetBox.height) / startBox.height;
		
		const absScaleX = Math.abs(rawScaleX);
		const absScaleY = Math.abs(rawScaleY);
		const isFlippedX = rawScaleX < 0;
		const isFlippedY = rawScaleY < 0;

		const originX = isFlippedX ? targetBox.x + targetBox.width : targetBox.x;
		const originY = isFlippedY ? targetBox.y + targetBox.height : targetBox.y;

		selectedShapesStartPositions.rectangles.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newWidth = startPos.width * absScaleX;
			const newHeight = startPos.height * absScaleY;
			
			let newX: number;
			let newY: number;
			if (isFlippedX) {
				newX = originX - relativeX * absScaleX - newWidth;
			} else {
				newX = originX + relativeX * absScaleX;
			}
			if (isFlippedY) {
				newY = originY - relativeY * absScaleY - newHeight;
			} else {
				newY = originY + relativeY * absScaleY;
			}
			
			moveRectangle(id, newX, newY, saveHistory);
			resizeRectangle(id, newWidth, newHeight, saveHistory);
		});

		selectedShapesStartPositions.ellipses.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newRadiusX = startPos.radius_x * absScaleX;
			const newRadiusY = startPos.radius_y * absScaleY;
			
			let newX: number;
			let newY: number;
			if (isFlippedX) {
				newX = originX - relativeX * absScaleX;
			} else {
				newX = originX + relativeX * absScaleX;
			}
			if (isFlippedY) {
				newY = originY - relativeY * absScaleY;
			} else {
				newY = originY + relativeY * absScaleY;
			}
			
			moveEllipse(id, newX, newY, saveHistory);
			resizeEllipse(id, newRadiusX, newRadiusY, saveHistory);
		});

		selectedShapesStartPositions.diamonds.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			const newWidth = startPos.width * absScaleX;
			const newHeight = startPos.height * absScaleY;
			
			let newX: number;
			let newY: number;
			if (isFlippedX) {
				newX = originX - relativeX * absScaleX - newWidth;
			} else {
				newX = originX + relativeX * absScaleX;
			}
			if (isFlippedY) {
				newY = originY - relativeY * absScaleY - newHeight;
			} else {
				newY = originY + relativeY * absScaleY;
			}
			
			moveDiamond(id, newX, newY, saveHistory);
			resizeDiamond(id, newWidth, newHeight, saveHistory);
		});

		selectedShapesStartPositions.lines.forEach((startPos, id) => {
			const startRelativeX = startPos.start.x - startBox.x;
			const startRelativeY = startPos.start.y - startBox.y;
			const endRelativeX = startPos.end.x - startBox.x;
			const endRelativeY = startPos.end.y - startBox.y;
			
			let newStartX: number;
			let newStartY: number;
			let newEndX: number;
			let newEndY: number;
			
			if (isFlippedX) {
				newStartX = originX - startRelativeX * absScaleX;
				newEndX = originX - endRelativeX * absScaleX;
			} else {
				newStartX = originX + startRelativeX * absScaleX;
				newEndX = originX + endRelativeX * absScaleX;
			}
			if (isFlippedY) {
				newStartY = originY - startRelativeY * absScaleY;
				newEndY = originY - endRelativeY * absScaleY;
			} else {
				newStartY = originY + startRelativeY * absScaleY;
				newEndY = originY + endRelativeY * absScaleY;
			}
			
			moveLine(id, newStartX, newStartY, newEndX, newEndY, saveHistory);
		});

		selectedShapesStartPositions.arrows.forEach((startPos, id) => {
			const startRelativeX = startPos.start.x - startBox.x;
			const startRelativeY = startPos.start.y - startBox.y;
			const endRelativeX = startPos.end.x - startBox.x;
			const endRelativeY = startPos.end.y - startBox.y;
			
			let newStartX: number;
			let newStartY: number;
			let newEndX: number;
			let newEndY: number;
			
			if (isFlippedX) {
				newStartX = originX - startRelativeX * absScaleX;
				newEndX = originX - endRelativeX * absScaleX;
			} else {
				newStartX = originX + startRelativeX * absScaleX;
				newEndX = originX + endRelativeX * absScaleX;
			}
			if (isFlippedY) {
				newStartY = originY - startRelativeY * absScaleY;
				newEndY = originY - endRelativeY * absScaleY;
			} else {
				newStartY = originY + startRelativeY * absScaleY;
				newEndY = originY + endRelativeY * absScaleY;
			}
			
			moveArrow(id, newStartX, newStartY, newEndX, newEndY, saveHistory);
		});

		selectedShapesStartPositions.images.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
			let newX: number;
			let newY: number;
			if (isFlippedX) {
				newX = originX - relativeX * absScaleX - startPos.width * absScaleX;
			} else {
				newX = originX + relativeX * absScaleX;
			}
			if (isFlippedY) {
				newY = originY - relativeY * absScaleY - startPos.height * absScaleY;
			} else {
				newY = originY + relativeY * absScaleY;
			}

			const newWidth = startPos.width * absScaleX;
			const newHeight = startPos.height * absScaleY;
			moveImage(id, newX, newY, saveHistory);
			resizeImage(id, Math.max(1, newWidth), Math.max(1, newHeight), saveHistory);
		});

		selectedShapesStartPositions.paths.forEach((startPos, id) => {
			if (startPos.points.length === 0) return;
			
			let minX = startPos.points[0].x;
			let minY = startPos.points[0].y;
			let maxX = startPos.points[0].x;
			let maxY = startPos.points[0].y;
			
			for (const point of startPos.points) {
				minX = Math.min(minX, point.x);
				minY = Math.min(minY, point.y);
				maxX = Math.max(maxX, point.x);
				maxY = Math.max(maxY, point.y);
			}
			
			const oldWidth = maxX - minX;
			const oldHeight = maxY - minY;
			
			if (oldWidth === 0 || oldHeight === 0) return;
			
			const relativeX = minX - startBox.x;
			const relativeY = minY - startBox.y;
			let newX: number;
			let newY: number;
			if (isFlippedX) {
				newX = originX - relativeX * absScaleX - oldWidth * absScaleX;
			} else {
				newX = originX + relativeX * absScaleX;
			}
			if (isFlippedY) {
				newY = originY - relativeY * absScaleY - oldHeight * absScaleY;
			} else {
				newY = originY + relativeY * absScaleY;
			}
			
			const newWidth = oldWidth * absScaleX;
			const newHeight = oldHeight * absScaleY;
			resizePath(id, newX, newY, newWidth, newHeight, saveHistory);
		});
	}

	function handleShapeClick(
		shape: Rectangle | Ellipse | Diamond | Line | Arrow | Text | Path | Image,
		shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'line' | 'arrow' | 'text' | 'path' | 'image',
		isShiftPressed: boolean,
		x: number,
		y: number
	) {
		const group = findGroupForShape(shape.id);
		if (group) {
			selectGroup(group, isShiftPressed);
			draggedShape = shape;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
			return;
		}

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
				selectedTexts.set([]);
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
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
				selectedTexts.set([]);
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedDiamond;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'ellipse') {
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
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedEllipse;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'line') {
			const clickedLine = shape as Line;
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
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedLine;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'arrow') {
			const clickedArrow = shape as Arrow;
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
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedArrow;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'text') {
			const clickedText = shape as Text;
			const index = $selectedTexts.findIndex(t => t.id === clickedText.id);
			const isAlreadySelected = index >= 0;

			if (isShiftPressed) {
				selectedTexts.set(
					isAlreadySelected
						? $selectedTexts.filter(t => t.id !== clickedText.id)
						: [...$selectedTexts, clickedText]
				);
			} else if (!isAlreadySelected) {
				selectedTexts.set([clickedText]);
				selectedRectangles.set([]);
				selectedEllipses.set([]);
				selectedDiamonds.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedText;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'path') {
			const clickedPath = shape as Path;
			const index = $selectedPaths.findIndex(p => p.id === clickedPath.id);
			const isAlreadySelected = index >= 0;
			
			if (isShiftPressed) {
				selectedPaths.set(
					isAlreadySelected
						? $selectedPaths.filter(p => p.id !== clickedPath.id)
						: [...$selectedPaths, clickedPath]
				);
			} else if (!isAlreadySelected) {
				selectedPaths.set([clickedPath]);
				selectedRectangles.set([]);
				selectedEllipses.set([]);
				selectedDiamonds.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
				selectedImages.set([]);
				selectedGroups.set([]);
			}

			draggedShape = clickedPath;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		} else if (shapeType === 'image') {
			const clickedImage = shape as Image;
			const index = $selectedImages.findIndex(i => i.id === clickedImage.id);
			const isAlreadySelected = index >= 0;
			
			if (isShiftPressed) {
				selectedImages.set(
					isAlreadySelected
						? $selectedImages.filter(i => i.id !== clickedImage.id)
						: [...$selectedImages, clickedImage]
				);
			} else {
				if (!isAlreadySelected) {
					selectedImages.set([clickedImage]);
					selectedRectangles.set([]);
					selectedEllipses.set([]);
					selectedDiamonds.set([]);
					selectedLines.set([]);
					selectedArrows.set([]);
					selectedPaths.set([]);
					selectedGroups.set([]);
				}
			}

			draggedShape = clickedImage;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		}
	}
	
	function finishFreehandDrawing() {
		if (isDrawingFreehand) {
			if (freehandPoints.length > 1) {
				addPath(freehandPoints);
				selectedRectangles.set([]);
				selectedEllipses.set([]);
				selectedDiamonds.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
				selectedPaths.set([]);
				updatePaths();
			}
			isDrawingFreehand = false;
			freehandPoints = [];
			scheduleRender();
		}
	}

	function performErase(x: number, y: number) {
		if (!$editorApi) return;

		const shapesToDelete: {
			rectangles: number[];
			ellipses: number[];
			diamonds: number[];
			lines: number[];
			arrows: number[];
			images: number[];
			paths: number[];
		} = {
			rectangles: [],
			ellipses: [],
			diamonds: [],
			lines: [],
			arrows: [],
			images: [],
			paths: []
		};

		$paths.forEach(path => {
			if (path.points.length === 0) return;

			for (let i = 0; i < path.points.length; i++) {
				const point = path.points[i];
				const dx = point.x - x;
				const dy = point.y - y;
				const distance = Math.sqrt(dx * dx + dy * dy);

				if (distance <= eraserRadius) {
					shapesToDelete.paths.push(path.id);
					break;
				}

				if (i > 0) {
					const prevPoint = path.points[i - 1];
					const segDx = prevPoint.x - point.x;
					const segDy = prevPoint.y - point.y;
					const segLength = Math.sqrt(segDx * segDx + segDy * segDy);

					if (segLength > 0) {
						const t = Math.max(0, Math.min(1, ((x - prevPoint.x) * segDx + (y - prevPoint.y) * segDy) / (segLength * segLength)));
						const closestX = prevPoint.x + t * segDx;
						const closestY = prevPoint.y + t * segDy;
						const closestDx = closestX - x;
						const closestDy = closestY - y;
						const closestDistance = Math.sqrt(closestDx * closestDx + closestDy * closestDy);

						if (closestDistance <= eraserRadius) {
							shapesToDelete.paths.push(path.id);
							break;
						}
					}
				}
			}
		});

		$rectangles.forEach(rect => {
			const centerX = rect.position.x + rect.width / 2;
			const centerY = rect.position.y + rect.height / 2;
			const dx = centerX - x;
			const dy = centerY - y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			const maxDistance = Math.sqrt(rect.width * rect.width + rect.height * rect.height) / 2;
			
			if (distance < eraserRadius + maxDistance) {
				shapesToDelete.rectangles.push(rect.id);
			}
		});

		$ellipses.forEach(ellipse => {
			const dx = ellipse.position.x - x;
			const dy = ellipse.position.y - y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			const maxRadius = Math.max(ellipse.radius_x, ellipse.radius_y);
			
			if (distance < eraserRadius + maxRadius) {
				shapesToDelete.ellipses.push(ellipse.id);
			}
		});

		$diamonds.forEach(diamond => {
			const centerX = diamond.position.x + diamond.width / 2;
			const centerY = diamond.position.y + diamond.height / 2;
			const dx = centerX - x;
			const dy = centerY - y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			const maxDistance = Math.sqrt(diamond.width * diamond.width + diamond.height * diamond.height) / 2;
			
			if (distance < eraserRadius + maxDistance) {
				shapesToDelete.diamonds.push(diamond.id);
			}
		});

		$lines.forEach(line => {
			const dx = line.end.x - line.start.x;
			const dy = line.end.y - line.start.y;
			const length = Math.sqrt(dx * dx + dy * dy);

			if (length > 0) {
				const t = Math.max(0, Math.min(1, ((x - line.start.x) * dx + (y - line.start.y) * dy) / (length * length)));
				const closestX = line.start.x + t * dx;
				const closestY = line.start.y + t * dy;
				const distX = closestX - x;
				const distY = closestY - y;
				const distance = Math.sqrt(distX * distX + distY * distY);

				if (distance <= eraserRadius) {
					shapesToDelete.lines.push(line.id);
				}
			} else {
				const distX = line.start.x - x;
				const distY = line.start.y - y;
				const distance = Math.sqrt(distX * distX + distY * distY);

				if (distance <= eraserRadius) {
					shapesToDelete.lines.push(line.id);
				}
			}
		});

		$arrows.forEach(arrow => {
			const dx = arrow.end.x - arrow.start.x;
			const dy = arrow.end.y - arrow.start.y;
			const length = Math.sqrt(dx * dx + dy * dy);

			if (length > 0) {
				const t = Math.max(0, Math.min(1, ((x - arrow.start.x) * dx + (y - arrow.start.y) * dy) / (length * length)));
				const closestX = arrow.start.x + t * dx;
				const closestY = arrow.start.y + t * dy;
				const distX = closestX - x;
				const distY = closestY - y;
				const distance = Math.sqrt(distX * distX + distY * distY);

				if (distance <= eraserRadius) {
					shapesToDelete.arrows.push(arrow.id);
				}
			} else {
				const distX = arrow.start.x - x;
				const distY = arrow.start.y - y;
				const distance = Math.sqrt(distX * distX + distY * distY);

				if (distance <= eraserRadius) {
					shapesToDelete.arrows.push(arrow.id);
				}
			}
		});

		$images.forEach(image => {
			const centerX = image.position.x + image.width / 2;
			const centerY = image.position.y + image.height / 2;
			const dx = centerX - x;
			const dy = centerY - y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			const maxDistance = Math.sqrt(image.width * image.width + image.height * image.height) / 2;
			
			if (distance < eraserRadius + maxDistance) {
				shapesToDelete.images.push(image.id);
			}
		});

		if (shapesToDelete.rectangles.length > 0 || shapesToDelete.ellipses.length > 0 ||
			shapesToDelete.diamonds.length > 0 || shapesToDelete.lines.length > 0 ||
			shapesToDelete.arrows.length > 0 ||
			shapesToDelete.images.length > 0 || shapesToDelete.paths.length > 0) {

			deleteShapes(
				shapesToDelete.rectangles,
				shapesToDelete.ellipses,
				shapesToDelete.lines,
				shapesToDelete.arrows,
				shapesToDelete.diamonds,
				[],
				shapesToDelete.paths,
				shapesToDelete.images
			);
			
			updatePaths();
		}
	}
	
	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		event.preventDefault();
		canvas.focus({ preventScroll: true });

		if (sidebarRef) {
			sidebarRef.closeSidebar();
		}

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
			const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length + $selectedTexts.length + $selectedPaths.length + $selectedImages.length;
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

			if (visualGroupBox && isPointOnGroupRotationHandle(x, y, visualGroupBox, $zoom)) {
				if (beginGroupRotation(visualGroupBox, x, y)) {
					return;
				}
			}

			if (allowIndividualHandles) {
				for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
					if (isPointOnRotationHandle(x, y, $selectedRectangles[i], 'rectangle', $zoom)) {
						if (beginRotation($selectedRectangles[i], 'rectangle', x, y)) {
							return;
						}
					}
					const handleIndex = getResizeHandleAt(
						x,
						y,
						$selectedRectangles[i],
						'rectangle',
						$zoom,
						getRenderedRotation($selectedRectangles[i], 'rectangle')
					);
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
					if (isPointOnRotationHandle(x, y, $selectedEllipses[i], 'ellipse', $zoom)) {
						if (beginRotation($selectedEllipses[i], 'ellipse', x, y)) {
							return;
						}
					}
					const handleIndex = getResizeHandleAt(
						x,
						y,
						$selectedEllipses[i],
						'ellipse',
						$zoom,
						getRenderedRotation($selectedEllipses[i], 'ellipse')
					);
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
					if (isPointOnRotationHandle(x, y, $selectedDiamonds[i], 'diamond', $zoom)) {
						if (beginRotation($selectedDiamonds[i], 'diamond', x, y)) {
							return;
						}
					}
					const handleIndex = getResizeHandleAt(
						x,
						y,
						$selectedDiamonds[i],
						'diamond',
						$zoom,
						getRenderedRotation($selectedDiamonds[i], 'diamond')
					);
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

				for (let i = $selectedImages.length - 1; i >= 0; i--) {
					if (isPointOnRotationHandle(x, y, $selectedImages[i], 'image', $zoom)) {
						if (beginRotation($selectedImages[i], 'image', x, y)) {
							return;
						}
					}
					const handleIndex = getResizeHandleAt(
						x,
						y,
						$selectedImages[i],
						'image',
						$zoom,
						getRenderedRotation($selectedImages[i], 'image')
					);
					if (handleIndex !== null) {
						isResizing = true;
						resizeHandleIndex = handleIndex;
						resizeStartShape = $selectedImages[i];
						resizeStartShapeType = 'image';
						const image = $selectedImages[i];
						resizeStartPos = {
							x: image.position.x,
							y: image.position.y,
							width: image.width,
							height: image.height
						};
						resizeStartMousePos = { x, y };
						isShiftPressedDuringResize = isShiftPressed;
						return;
					}
				}

				for (let i = $selectedPaths.length - 1; i >= 0; i--) {
					if (isPointOnRotationHandle(x, y, $selectedPaths[i], 'path', $zoom)) {
						if (beginRotation($selectedPaths[i], 'path', x, y)) {
							return;
						}
					}
					const handleIndex = getPathResizeHandleAt(x, y, $selectedPaths[i], $zoom);
					if (handleIndex !== null) {
						isResizing = true;
						resizeHandleIndex = handleIndex;
						resizeStartShape = $selectedPaths[i];
						resizeStartShapeType = 'path';
						const pathBounds = getPathBoundingBox($selectedPaths[i]);
						if (pathBounds) {
							resizeStartPos = {
								x: pathBounds.x,
								y: pathBounds.y,
								width: pathBounds.width,
								height: pathBounds.height
							};
						}
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
					
					let adjustedX = x;
					let adjustedY = y;
					if (groupHandleIndex === 0 || groupHandleIndex === 3 || groupHandleIndex === 7) {
						adjustedX = x - selectionPaddingValue;
					}
					if (groupHandleIndex === 1 || groupHandleIndex === 2 || groupHandleIndex === 5) {
						adjustedX = x + selectionPaddingValue;
					}
					if (groupHandleIndex === 0 || groupHandleIndex === 1 || groupHandleIndex === 4) {
						adjustedY = y - selectionPaddingValue;
					}
					if (groupHandleIndex === 2 || groupHandleIndex === 3 || groupHandleIndex === 6) {
						adjustedY = y + selectionPaddingValue;
					}
					groupResizeStartMousePos = { x: adjustedX, y: adjustedY };
					
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
					handleShapeClick($lines[i], 'line', isShiftPressed, x, y);
					return;
				}
			}

			for (let i = $arrows.length - 1; i >= 0; i--) {
				if (isPointOnLine(x, y, $arrows[i], 5 / $zoom)) {
					handleShapeClick($arrows[i], 'arrow', isShiftPressed, x, y);
					return;
				}
			}

			for (let i = $paths.length - 1; i >= 0; i--) {
				const path = $paths[i];
				const pathBounds = getPathBoundingBox(path);
				if (pathBounds && x >= pathBounds.x && x <= pathBounds.x + pathBounds.width && y >= pathBounds.y && y <= pathBounds.y + pathBounds.height) {
					handleShapeClick(path, 'path', isShiftPressed, x, y);
					return;
				}
			}

		for (let i = $texts.length - 1; i >= 0; i--) {
			const text: Text = $texts[i];
			if (x >= text.position.x && x <= text.position.x + text.width &&
				y >= text.position.y && y <= text.position.y + text.height) {
				handleShapeClick(text, 'text', isShiftPressed, x, y);
				return;
			}
		}

			for (let i = $images.length - 1; i >= 0; i--) {
				if (isPointInImage(x, y, $images[i])) {
					handleShapeClick($images[i], 'image', isShiftPressed, x, y);
					return;
				}
			}

			if (!isShiftPressed && totalSelectedCount > 1 && visualGroupBox) {
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
					const firstSelectedPath = $selectedPaths[0];
					const firstSelectedImage = $selectedImages[0];
					
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
					} else if (firstSelectedPath) {
						draggedShape = firstSelectedPath;
					} else if (firstSelectedImage) {
						draggedShape = firstSelectedImage;
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
			resetRotationState();
			clearAllSelections();
		} else if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond') {
			resetRotationState();
			clearAllSelections();
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			createStartPos = { x, y };
			createCurrentPos = { x, y };
			scheduleRender();
		} else if ($activeTool === 'line') {
			resetRotationState();
			clearAllSelections();
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			lineStart = { x, y };
			lineEnd = { x, y };
			scheduleRender();
		} else if ($activeTool === 'arrow') {
			resetRotationState();
			clearAllSelections();
			isCreatingShape = true;
			isShiftPressedDuringCreation = isShiftPressed;
			arrowStart = { x, y };
			arrowEnd = { x, y };
			scheduleRender();
		} else if ($activeTool === 'freehand') {
			finishFreehandDrawing();
			resetRotationState();
			clearAllSelections();
			isDrawingFreehand = true;
			freehandPoints = [{ x, y }];
			scheduleRender();
		} else if ($activeTool === 'text') {
			resetRotationState();
			clearAllSelections();
			const newId = $editorApi.add_text(x, y, 100, 30, 'Text');
			const allTexts = Array.from($editorApi.get_texts() as Text[]);
			const newText = allTexts.find(t => t.id === Number(newId));
			if (newText) {
				selectedTexts.set([newText]);
			}
			activeTool.set('select');
			updateStores();
		} else if ($activeTool === 'eraser') {
			resetRotationState();
			clearAllSelections();
			isErasing = true;
			eraserPosition = { x, y };
			eraserShadowPosition = { x, y };
			performErase(x, y);
			scheduleRender();
		}
	}

	function enterTextEditingMode(text: Text) {
		if (!canvas) return;

		const canvasRect = canvas.getBoundingClientRect();
		const textScreenX = (text.position.x * $zoom) + $viewportOffset.x + canvasRect.left;
		const textScreenY = (text.position.y * $zoom) + $viewportOffset.y + canvasRect.top;
		const textScreenWidth = text.width * $zoom;
		const textScreenHeight = text.height * $zoom;

		const input = document.createElement('input');
		input.type = 'text';
		input.value = text.content;
		input.style.position = 'fixed';
		input.style.left = `${textScreenX}px`;
		input.style.top = `${textScreenY}px`;
		input.style.width = `${textScreenWidth}px`;
		input.style.height = `${textScreenHeight}px`;
		input.style.fontSize = `${(text.font_size || 16) * $zoom}px`;
		input.style.fontFamily = text.font_family || 'Arial';
		input.style.fontWeight = text.font_weight || 'normal';
		input.style.textAlign = text.text_align || 'left';
		input.style.color = text.color || getDefaultStrokeColor();
		input.style.background = 'transparent';
		input.style.border = '1px dashed #007acc';
		input.style.outline = 'none';
		input.style.padding = '2px';
		input.style.zIndex = '9999';

		document.body.appendChild(input);
		input.focus();
		input.select();

		const finishEditing = () => {
			const newContent = input.value.trim();
			if (newContent !== text.content) {
				$editorApi.set_text_content(BigInt(text.id), newContent, true);
				updateStores();
			}

			if (document.body.contains(input)) {
				document.body.removeChild(input);
			}
		};

		input.onblur = finishEditing;
		input.onkeydown = (e) => {
			if (e.key === 'Enter') {
				finishEditing();
			} else if (e.key === 'Escape') {
				if (document.body.contains(input)) {
					document.body.removeChild(input);
				}
			}
		};
	}

	function updateStores() {
		if (!$editorApi) return;
		const api = get(editorApi);

		const selectedRectIds = new Set($selectedRectangles.map(r => r.id));
		const selectedEllipseIds = new Set($selectedEllipses.map(e => e.id));
		const selectedLineIds = new Set($selectedLines.map(l => l.id));
		const selectedArrowIds = new Set($selectedArrows.map(a => a.id));
		const selectedDiamondIds = new Set($selectedDiamonds.map(d => d.id));
		const selectedImageIds = new Set($selectedImages.map(i => i.id));
		const selectedPathIds = new Set($selectedPaths.map(p => p.id));
		const selectedTextIds = new Set($selectedTexts.map(t => t.id));

		const allRectangles = api.get_rectangles() as Rectangle[];
		const allEllipses = api.get_ellipses() as Ellipse[];
		const allLines = api.get_lines() as Line[];
		const allArrows = api.get_arrows() as Arrow[];
		const allDiamonds = api.get_diamonds() as Diamond[];
		const allImages = api.get_images() as Image[];
		const allPaths = api.get_paths() as Path[];
		const allTexts = api.get_texts() as Text[];

		rectangles.set(allRectangles);
		ellipses.set(allEllipses);
		lines.set(allLines);
		arrows.set(allArrows);
		diamonds.set(allDiamonds);
		images.set(allImages);
		paths.set(allPaths);
		texts.set(allTexts);

		selectedRectangles.set(allRectangles.filter(r => selectedRectIds.has(r.id)));
		selectedEllipses.set(allEllipses.filter(e => selectedEllipseIds.has(e.id)));
		selectedLines.set(allLines.filter(l => selectedLineIds.has(l.id)));
		selectedArrows.set(allArrows.filter(a => selectedArrowIds.has(a.id)));
		selectedDiamonds.set(allDiamonds.filter(d => selectedDiamondIds.has(d.id)));
		selectedImages.set(allImages.filter(i => selectedImageIds.has(i.id)));
		selectedPaths.set(allPaths.filter(p => selectedPathIds.has(p.id)));
		selectedTexts.set(allTexts.filter(t => selectedTextIds.has(t.id)));
	}

	function rotateGroup(delta: number, saveHistory: boolean) {
		if (!groupRotationState) return;
		const center = groupRotationState.center;

		selectedShapesStartPositions.rectangles.forEach((startPos, id) => {
			const rotatedPos = rotatePointAround({ x: startPos.x + startPos.width / 2, y: startPos.y + startPos.height / 2 }, center, delta);
			const newX = rotatedPos.x - startPos.width / 2;
			const newY = rotatedPos.y - startPos.height / 2;
			const newRotation = normalizeAngle(startPos.rotation + delta);
			moveRectangle(id, newX, newY, saveHistory);
			setRectangleRotation(id, newRotation, saveHistory);
		});

		selectedShapesStartPositions.ellipses.forEach((startPos, id) => {
			const rotatedPos = rotatePointAround({ x: startPos.x, y: startPos.y }, center, delta);
			const newRotation = normalizeAngle(startPos.rotation + delta);
			moveEllipse(id, rotatedPos.x, rotatedPos.y, saveHistory);
			setEllipseRotation(id, newRotation, saveHistory);
		});

		selectedShapesStartPositions.diamonds.forEach((startPos, id) => {
			const rotatedPos = rotatePointAround({ x: startPos.x + startPos.width / 2, y: startPos.y + startPos.height / 2 }, center, delta);
			const newX = rotatedPos.x - startPos.width / 2;
			const newY = rotatedPos.y - startPos.height / 2;
			const newRotation = normalizeAngle(startPos.rotation + delta);
			moveDiamond(id, newX, newY, saveHistory);
			setDiamondRotation(id, newRotation, saveHistory);
		});

		selectedShapesStartPositions.lines.forEach((startPos, id) => {
			const newStart = rotatePointAround(startPos.start, center, delta);
			const newEnd = rotatePointAround(startPos.end, center, delta);
			moveLine(id, newStart.x, newStart.y, newEnd.x, newEnd.y, saveHistory);
		});

		selectedShapesStartPositions.arrows.forEach((startPos, id) => {
			const newStart = rotatePointAround(startPos.start, center, delta);
			const newEnd = rotatePointAround(startPos.end, center, delta);
			moveArrow(id, newStart.x, newStart.y, newEnd.x, newEnd.y, saveHistory);
		});

		selectedShapesStartPositions.paths.forEach((startPos, id) => {
			if (startPos.points.length === 0) return;
			
			const pathBounds = getPathBoundingBox({ points: startPos.points });
			if (!pathBounds) return;
			
			const centerX = pathBounds.x + pathBounds.width / 2;
			const centerY = pathBounds.y + pathBounds.height / 2;
			const pathCenter = { x: centerX, y: centerY };
			
			const rotatedCenter = rotatePointAround(pathCenter, center, delta);
			const offsetX = rotatedCenter.x - centerX;
			const offsetY = rotatedCenter.y - centerY;
			
			const currentPath = $paths.find(p => p.id === id);
			if (!currentPath) return;
			
			const currentBounds = getPathBoundingBox({ points: currentPath.points });
			if (!currentBounds) return;
			
			const currentCenterX = currentBounds.x + currentBounds.width / 2;
			const currentCenterY = currentBounds.y + currentBounds.height / 2;
			
			const actualOffsetX = rotatedCenter.x - currentCenterX;
			const actualOffsetY = rotatedCenter.y - currentCenterY;
			
			const newRotation = normalizeAngle(startPos.rotation + delta);
			movePath(id, actualOffsetX, actualOffsetY, false);
			setPathRotation(id, newRotation, false);
		});

		selectedShapesStartPositions.images.forEach((startPos, id) => {
			const rotatedPos = rotatePointAround({ x: startPos.x + startPos.width / 2, y: startPos.y + startPos.height / 2 }, center, delta);
			const newX = rotatedPos.x - startPos.width / 2;
			const newY = rotatedPos.y - startPos.height / 2;
			const newRotation = normalizeAngle(startPos.rotation + delta);
			moveImage(id, newX, newY, saveHistory);
			setImageRotation(id, newRotation, saveHistory);
		});
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const worldPos = screenToWorld(screenX, screenY, $viewportOffset, $zoom);
		lastMouseWorldPos = { x: worldPos.x, y: worldPos.y };

		const { x, y } = worldPos;

		if ($activeTool === 'freehand' && isDrawingFreehand) {
			if (event.buttons === 0) {
				finishFreehandDrawing();
				return;
			}
			const lastPoint = freehandPoints[freehandPoints.length - 1];
			const dx = x - lastPoint.x;
			const dy = y - lastPoint.y;
			const distance = Math.sqrt(dx * dx + dy * dy);
			if (distance > 2) {
				freehandPoints.push({ x, y });
				scheduleRender();
			}
			canvas.style.cursor = 'crosshair';
			return;
		}

		if ($activeTool === 'eraser') {
			if (isErasing && event.buttons !== 0) {
				eraserPosition = { x, y };
				if (eraserShadowPosition) {
					const dx = x - eraserShadowPosition.x;
					const dy = y - eraserShadowPosition.y;
					const distance = Math.sqrt(dx * dx + dy * dy);
					if (distance > 1) {
						eraserShadowPosition.x += dx * 0.3;
						eraserShadowPosition.y += dy * 0.3;
					}
				} else {
					eraserShadowPosition = { x, y };
				}

				performErase(x, y);

				scheduleRender();
			} else {
				eraserPosition = { x, y };
				eraserShadowPosition = { x, y };
				scheduleRender();
			}
			canvas.style.cursor = 'none';
			return;
		}

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

		if (isGroupRotating && groupRotationState) {
			const currentAngle = Math.atan2(y - groupRotationState.center.y, x - groupRotationState.center.x);
			const delta = currentAngle - groupRotationState.mouseStartAngle;
			rotateGroup(delta, false);
			if (canvas) {
				canvas.style.cursor = 'grabbing';
			}
			scheduleRender();
			return;
		}

		if (isRotating && rotationState) {
			const currentAngle = Math.atan2(y - rotationState.center.y, x - rotationState.center.x);
			const delta = currentAngle - rotationState.mouseStartAngle;
			const nextAngle = normalizeAngle(rotationState.startAngle + delta);
			rotationPreview = { type: rotationState.type, id: rotationState.id, angle: nextAngle };
			if (canvas) {
				canvas.style.cursor = 'grabbing';
			}
			scheduleRender();
			return;
		}
		
		if (isDragging && draggedShape) {
			canvas.style.cursor = 'move';
			const deltaX = x - dragStartPos.x;
			const deltaY = y - dragStartPos.y;
			
			dragOffset = { x: deltaX, y: deltaY };
			if (ctx && canvas) {
				render();
			}
			return;
		}
		
		if (isGroupResizing && groupResizeHandleIndex !== null) {
			const padding = groupResizePadding;
			let adjustedX = x;
			let adjustedY = y;
			
			if (groupResizeHandleIndex === 0 || groupResizeHandleIndex === 3 || groupResizeHandleIndex === 7) {
				adjustedX = x - padding;
			}
			if (groupResizeHandleIndex === 1 || groupResizeHandleIndex === 2 || groupResizeHandleIndex === 5) {
				adjustedX = x + padding;
			}
			if (groupResizeHandleIndex === 0 || groupResizeHandleIndex === 1 || groupResizeHandleIndex === 4) {
				adjustedY = y - padding;
			}
			if (groupResizeHandleIndex === 2 || groupResizeHandleIndex === 3 || groupResizeHandleIndex === 6) {
				adjustedY = y + padding;
			}
			
			const newBox = computeGroupResizeBox(adjustedX, adjustedY);
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

				if (isShiftPressedDuringResize && adjustsHorizontal && adjustsVertical && resizeStartPos.height !== 0) {
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
			} else if (resizeStartShapeType === 'image') {
				const image = resizeStartShape as Image;
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

				const newWidth = Math.max(1, finalRight - finalLeft);
				const newHeight = Math.max(1, finalBottom - finalTop);

				resizePreview = { x: finalLeft, y: finalTop, width: newWidth, height: newHeight, type: 'image', id: image.id };
				scheduleRender();
			} else if (resizeStartShapeType === 'path') {
				const path = resizeStartShape as Path;
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

				const newWidth = Math.max(1, finalRight - finalLeft);
				const newHeight = Math.max(1, finalBottom - finalTop);

				resizePreview = { x: finalLeft, y: finalTop, width: newWidth, height: newHeight, type: 'path', id: path.id };
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
		} else if ($activeTool === 'freehand') {
			canvas.style.cursor = 'crosshair';
		} else if ($activeTool === 'select') {
			if (isResizing) {
				canvas.style.cursor = resizeHandleIndex !== null ? resizeCursors[resizeHandleIndex] : 'default';
			} else if (isDragging && draggedShape) {
				canvas.style.cursor = 'move';
			} else {
				const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length + $selectedPaths.length + $selectedImages.length;
				const selectionPaddingValue = getSelectionPaddingValue($zoom);
				const rawGroupBoxForCursor =
					totalSelectedCount > 1 ? groupResizeCurrentBox ?? calculateGroupBoundingBox() : null;
				const visualGroupBoxForCursor = rawGroupBoxForCursor
					? expandBoundingBox(rawGroupBoxForCursor, selectionPaddingValue)
					: null;

				if (visualGroupBoxForCursor && isPointOnGroupRotationHandle(x, y, visualGroupBoxForCursor, $zoom)) {
					canvas.style.cursor = 'grab';
					return;
				}

				if (totalSelectedCount <= 1) {
					for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
						if (isPointOnRotationHandle(x, y, $selectedRectangles[i], 'rectangle', $zoom)) {
							canvas.style.cursor = 'grab';
							return;
						}
					}
					for (let i = $selectedEllipses.length - 1; i >= 0; i--) {
						if (isPointOnRotationHandle(x, y, $selectedEllipses[i], 'ellipse', $zoom)) {
							canvas.style.cursor = 'grab';
							return;
						}
					}
					for (let i = $selectedDiamonds.length - 1; i >= 0; i--) {
						if (isPointOnRotationHandle(x, y, $selectedDiamonds[i], 'diamond', $zoom)) {
							canvas.style.cursor = 'grab';
							return;
						}
					}
					for (let i = $selectedPaths.length - 1; i >= 0; i--) {
						if (isPointOnRotationHandle(x, y, $selectedPaths[i], 'path', $zoom)) {
							canvas.style.cursor = 'grab';
							return;
						}
					}
					for (let i = $selectedRectangles.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedRectangles[i],
							'rectangle',
							$zoom,
							getRenderedRotation($selectedRectangles[i], 'rectangle')
						);
						if (handleIndex !== null) {
							canvas.style.cursor = resizeCursors[handleIndex];
							return;
						}
					}
					for (let i = $selectedEllipses.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedEllipses[i],
							'ellipse',
							$zoom,
							getRenderedRotation($selectedEllipses[i], 'ellipse')
						);
						if (handleIndex !== null) {
							canvas.style.cursor = resizeCursors[handleIndex];
							return;
						}
					}
					
					for (let i = $selectedDiamonds.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedDiamonds[i],
							'diamond',
							$zoom,
							getRenderedRotation($selectedDiamonds[i], 'diamond')
						);
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
					
					for (let i = $selectedImages.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedImages[i],
							'image',
							$zoom,
							getRenderedRotation($selectedImages[i], 'image')
						);
						if (handleIndex !== null) {
							canvas.style.cursor = resizeCursors[handleIndex];
							return;
						}
					}
					
					for (let i = $selectedPaths.length - 1; i >= 0; i--) {
						const handleIndex = getPathResizeHandleAt(x, y, $selectedPaths[i], $zoom);
						if (handleIndex !== null) {
							canvas.style.cursor = resizeCursors[handleIndex];
							return;
						}
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
			for (let i = $paths.length - 1; i >= 0; i--) {
				const path = $paths[i];
				const isSelected = $selectedPaths.some(selected => selected.id === path.id);
				if (isSelected) {
					const pathBounds = getPathBoundingBox(path);
					if (pathBounds && x >= pathBounds.x && x <= pathBounds.x + pathBounds.width && y >= pathBounds.y && y <= pathBounds.y + pathBounds.height) {
						canvas.style.cursor = 'move';
						return;
					}
				} else if (isPointOnPath(x, y, path)) {
					canvas.style.cursor = 'move';
					return;
				}
			}
			for (let i = $images.length - 1; i >= 0; i--) {
				const image = $images[i];
				const isSelected = $selectedImages.some(selected => selected.id === image.id);
				if (isSelected) {
					if (x >= image.position.x && x <= image.position.x + image.width && y >= image.position.y && y <= image.position.y + image.height) {
						canvas.style.cursor = 'move';
						return;
					}
				} else if (isPointInImage(x, y, image)) {
					canvas.style.cursor = 'move';
					return;
				}
			}
			canvas.style.cursor = 'default';
		}
		}

		if (isSelectingBox && selectionBoxStart) {
			selectionBoxEnd = { x, y };
			scheduleRender();
		}
	}

	function handleCanvasDoubleClick(event: MouseEvent) {
		if (!canvas || !ctx) return;
		event.preventDefault();

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset, $zoom);

		for (let i = $texts.length - 1; i >= 0; i--) {
			const text = $texts[i];
			if (x >= text.position.x && x <= text.position.x + text.width &&
				y >= text.position.y && y <= text.position.y + text.height) {
				enterTextEditingMode(text);
				return;
			}
		}
	}
	
	function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
		}

		if (isGroupRotating) {
			if (groupRotationState && lastMouseWorldPos) {
				const currentAngle = Math.atan2(lastMouseWorldPos.y - groupRotationState.center.y, lastMouseWorldPos.x - groupRotationState.center.x);
				const delta = currentAngle - groupRotationState.mouseStartAngle;
				rotateGroup(delta, true);
			}
			resetGroupRotationState();
			scheduleRender();
		}

		if (isRotating) {
			if (rotationState && rotationPreview) {
				applyRotationToShape(rotationState.type, rotationState.id, rotationPreview.angle, true);
			}
			resetRotationState();
			scheduleRender();
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
			} else if (resizePreview.type === 'image') {
				moveImage(resizePreview.id, resizePreview.x, resizePreview.y, true);
				resizeImage(resizePreview.id, resizePreview.width, resizePreview.height, false);
			} else if (resizePreview.type === 'path') {
				resizePath(resizePreview.id, resizePreview.x, resizePreview.y, resizePreview.width, resizePreview.height, true);
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
				
				const selectedPathsArray: Path[] = [];
				$paths.forEach(path => {
					if (pathIntersectsBox(path, box)) {
						selectedPathsArray.push(path);
					}
				});

				const selectedImagesArray: Image[] = [];
				$images.forEach(image => {
					if (imageIntersectsBox(image, box)) {
						selectedImagesArray.push(image);
					}
				});
				
				selectedRectangles.set(selectedRects);
				selectedEllipses.set(selectedElls);
				selectedDiamonds.set(selectedDias);
				selectedLines.set(selectedLinesArray);
				selectedArrows.set(selectedArrs);
				selectedPaths.set(selectedPathsArray);
				selectedImages.set(selectedImagesArray);
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
			groupResizeStartMousePos = { x: 0, y: 0 };
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
			
			selectedShapesStartPositions.paths.forEach((startPos, id) => {
				movePath(id, dragOffset.x, dragOffset.y, false);
			});

			selectedShapesStartPositions.images.forEach((startPos, id) => {
				const newX = startPos.x + dragOffset.x;
				const newY = startPos.y + dragOffset.y;
				moveImage(id, newX, newY, false);
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
			
			finishFreehandDrawing();
			
			isCreatingShape = false;
		}

		if (isErasing) {
			if ($editorApi) {
				$editorApi.save_snapshot();
			}
			isErasing = false;
			scheduleRender();
		}
		
		createStartPos = { x: 0, y: 0 };
		createCurrentPos = { x: 0, y: 0 };
		lineStart = null;
		lineEnd = null;
		arrowStart = null;
		arrowEnd = null;
		scheduleRender();
		
		isDragging = false;
		draggedShape = null;
		dragOffset = { x: 0, y: 0 };
		canvas?.focus();
	}
	
	function handleMouseLeave() {
		finishFreehandDrawing();
		if (isPanning) {
			isPanning = false;
		}
	}
	
	function getSelectionOutlineBounds(
		x: number,
		y: number,
		width: number,
		height: number,
		zoom: number,
		includePadding: boolean = true
	) {
		const padding = includePadding ? getSelectionPaddingValue(zoom) : 0;
		const gap = 4 / zoom;
		return {
			x: x - padding - gap,
			y: y - padding - gap,
			width: width + (padding + gap) * 2,
			height: height + (padding + gap) * 2
		};
	}

	function renderSelectionOutline(
		ctx: CanvasRenderingContext2D,
		x: number,
		y: number,
		width: number,
		height: number,
		zoom: number,
		includePadding: boolean = true,
		rotation: number = 0
	) {
		const bounds = getSelectionOutlineBounds(x, y, width, height, zoom, includePadding);
		ctx.save();
		ctx.strokeStyle = getSelectionOutlineColor();
		ctx.lineWidth = 1 / zoom;
		const centerX = bounds.x + bounds.width / 2;
		const centerY = bounds.y + bounds.height / 2;
		ctx.translate(centerX, centerY);
		ctx.rotate(rotation);
		ctx.strokeRect(-bounds.width / 2, -bounds.height / 2, bounds.width, bounds.height);
		ctx.restore();
	}

	function renderCornerHandles(
		ctx: CanvasRenderingContext2D,
		x: number,
		y: number,
		width: number,
		height: number,
		zoom: number,
		includePadding: boolean = true,
		rotation: number = 0
	) {
		const handleSize = 8 / zoom;
		const halfHandle = handleSize / 2;
		const bounds = getSelectionOutlineBounds(x, y, width, height, zoom, includePadding);
		const halfWidth = bounds.width / 2;
		const halfHeight = bounds.height / 2;
		
		ctx.fillStyle = getHandleFillColor();
		ctx.strokeStyle = getHandleStrokeColor();
		ctx.lineWidth = 2 / zoom;
		
		const cornerHandles = [
			{ x: -halfWidth, y: -halfHeight },
			{ x: halfWidth, y: -halfHeight },
			{ x: halfWidth, y: halfHeight },
			{ x: -halfWidth, y: halfHeight }
		];
		
		ctx.save();
		ctx.translate(bounds.x + halfWidth, bounds.y + halfHeight);
		ctx.rotate(rotation);
		cornerHandles.forEach((handle) => {
			ctx.beginPath();
			ctx.rect(handle.x - halfHandle, handle.y - halfHandle, handleSize, handleSize);
			ctx.fill();
			ctx.stroke();
		});
		ctx.restore();
	}

	function calculateGroupBoundingBox(): { x: number; y: number; width: number; height: number } | null {
		const allSelectedShapes: Array<{ minX: number; minY: number; maxX: number; maxY: number }> = [];
		
		$selectedRectangles.forEach(rect => {
			const isDragged = isDragging && selectedShapesStartPositions.rectangles.has(rect.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'rectangle' && resizePreview.id === rect.id;
			let x: number, y: number, width: number, height: number;
			if (isResized && resizePreview) {
				x = resizePreview.x;
				y = resizePreview.y;
				width = resizePreview.width;
				height = resizePreview.height;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.rectangles.get(rect.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
				width = rect.width;
				height = rect.height;
			} else {
				x = rect.position.x;
				y = rect.position.y;
				width = rect.width;
				height = rect.height;
			}
			allSelectedShapes.push({
				minX: x,
				minY: y,
				maxX: x + width,
				maxY: y + height
			});
		});
		
		$selectedEllipses.forEach(ellipse => {
			const isDragged = isDragging && selectedShapesStartPositions.ellipses.has(ellipse.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'ellipse' && resizePreview.id === ellipse.id;
			let x: number, y: number, radiusX: number, radiusY: number;
			if (isResized && resizePreview) {
				x = resizePreview.x;
				y = resizePreview.y;
				radiusX = resizePreview.width / 2;
				radiusY = resizePreview.height / 2;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.ellipses.get(ellipse.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
				radiusX = ellipse.radius_x;
				radiusY = ellipse.radius_y;
			} else {
				x = ellipse.position.x;
				y = ellipse.position.y;
				radiusX = ellipse.radius_x;
				radiusY = ellipse.radius_y;
			}
			allSelectedShapes.push({
				minX: x - radiusX,
				minY: y - radiusY,
				maxX: x + radiusX,
				maxY: y + radiusY
			});
		});
		
		$selectedDiamonds.forEach(diamond => {
			const isDragged = isDragging && selectedShapesStartPositions.diamonds.has(diamond.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'diamond' && resizePreview.id === diamond.id;
			let x: number, y: number, width: number, height: number;
			if (isResized && resizePreview) {
				x = resizePreview.x;
				y = resizePreview.y;
				width = resizePreview.width;
				height = resizePreview.height;
			} else if (isDragged) {
				const startPos = selectedShapesStartPositions.diamonds.get(diamond.id)!;
				x = startPos.x + dragOffset.x;
				y = startPos.y + dragOffset.y;
				width = diamond.width;
				height = diamond.height;
			} else {
				x = diamond.position.x;
				y = diamond.position.y;
				width = diamond.width;
				height = diamond.height;
			}
			allSelectedShapes.push({
				minX: x,
				minY: y,
				maxX: x + width,
				maxY: y + height
			});
		});
		
		$selectedLines.forEach(line => {
			const isDragged = isDragging && selectedShapesStartPositions.lines.has(line.id);
			const isResized = isResizing && resizePreview && resizePreview.type === 'line' && resizePreview.id === line.id;
			let startX: number, startY: number, endX: number, endY: number;
			if (isResized && resizePreview) {
				startX = resizePreview.x;
				startY = resizePreview.y;
				endX = resizePreview.x + resizePreview.width;
				endY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
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
			const isResized = isResizing && resizePreview && resizePreview.type === 'arrow' && resizePreview.id === arrow.id;
			let startX: number, startY: number, endX: number, endY: number;
			if (isResized && resizePreview) {
				startX = resizePreview.x;
				startY = resizePreview.y;
				endX = resizePreview.x + resizePreview.width;
				endY = resizePreview.y + resizePreview.height;
			} else if (isDragged) {
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
		
		$selectedPaths.forEach(path => {
			const isDragged = isDragging && selectedShapesStartPositions.paths.has(path.id);
			const pathBounds = getPathBoundingBox(path);
			if (!pathBounds) return;
			
			let minX = pathBounds.x;
			let minY = pathBounds.y;
			let maxX = pathBounds.x + pathBounds.width;
			let maxY = pathBounds.y + pathBounds.height;
			
			if (isDragged) {
				const startPos = selectedShapesStartPositions.paths.get(path.id)!;
				const offsetX = dragOffset.x;
				const offsetY = dragOffset.y;
				minX = pathBounds.x + offsetX;
				minY = pathBounds.y + offsetY;
				maxX = pathBounds.x + pathBounds.width + offsetX;
				maxY = pathBounds.y + pathBounds.height + offsetY;
			}
			
			allSelectedShapes.push({
				minX,
				minY,
				maxX,
				maxY
			});
		});

		$selectedImages.forEach(image => {
			const isDragged = isDragging && selectedShapesStartPositions.images.has(image.id);
			let minX = image.position.x;
			let minY = image.position.y;
			let maxX = image.position.x + image.width;
			let maxY = image.position.y + image.height;
			
			if (isDragged) {
				const startPos = selectedShapesStartPositions.images.get(image.id)!;
				minX = startPos.x + dragOffset.x;
				minY = startPos.y + dragOffset.y;
				maxX = startPos.x + startPos.width + dragOffset.x;
				maxY = startPos.y + startPos.height + dragOffset.y;
			}
			
			allSelectedShapes.push({
				minX,
				minY,
				maxX,
				maxY
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

	function calculateGroupBoundingBoxForGroup(group: Group): { x: number; y: number; width: number; height: number } | null {
		const allShapeBounds: Array<{ minX: number; minY: number; maxX: number; maxY: number }> = [];
		const shapeIds = getAllShapeIdsInGroup(group);
		
		$rectangles.forEach(rect => {
			if (shapeIds.includes(rect.id)) {
				allShapeBounds.push({
					minX: rect.position.x,
					minY: rect.position.y,
					maxX: rect.position.x + rect.width,
					maxY: rect.position.y + rect.height
				});
			}
		});
		
		$ellipses.forEach(ellipse => {
			if (shapeIds.includes(ellipse.id)) {
				allShapeBounds.push({
					minX: ellipse.position.x - ellipse.radius_x,
					minY: ellipse.position.y - ellipse.radius_y,
					maxX: ellipse.position.x + ellipse.radius_x,
					maxY: ellipse.position.y + ellipse.radius_y
				});
			}
		});
		
		$diamonds.forEach(diamond => {
			if (shapeIds.includes(diamond.id)) {
				allShapeBounds.push({
					minX: diamond.position.x,
					minY: diamond.position.y,
					maxX: diamond.position.x + diamond.width,
					maxY: diamond.position.y + diamond.height
				});
			}
		});
		
		$lines.forEach(line => {
			if (shapeIds.includes(line.id)) {
				allShapeBounds.push({
					minX: Math.min(line.start.x, line.end.x),
					minY: Math.min(line.start.y, line.end.y),
					maxX: Math.max(line.start.x, line.end.x),
					maxY: Math.max(line.start.y, line.end.y)
				});
			}
		});
		
		$arrows.forEach(arrow => {
			if (shapeIds.includes(arrow.id)) {
				allShapeBounds.push({
					minX: Math.min(arrow.start.x, arrow.end.x),
					minY: Math.min(arrow.start.y, arrow.end.y),
					maxX: Math.max(arrow.start.x, arrow.end.x),
					maxY: Math.max(arrow.start.y, arrow.end.y)
				});
			}
		});
		
		$paths.forEach(path => {
			if (shapeIds.includes(path.id)) {
				const pathBounds = getPathBoundingBox(path);
				if (pathBounds) {
					allShapeBounds.push({
						minX: pathBounds.x,
						minY: pathBounds.y,
						maxX: pathBounds.x + pathBounds.width,
						maxY: pathBounds.y + pathBounds.height
					});
				}
			}
		});

		$images.forEach(image => {
			if (shapeIds.includes(image.id)) {
				allShapeBounds.push({
					minX: image.position.x,
					minY: image.position.y,
					maxX: image.position.x + image.width,
					maxY: image.position.y + image.height
				});
			}
		});
		
		if (allShapeBounds.length === 0) return null;
		
		const minX = Math.min(...allShapeBounds.map(s => s.minX));
		const minY = Math.min(...allShapeBounds.map(s => s.minY));
		const maxX = Math.max(...allShapeBounds.map(s => s.maxX));
		const maxY = Math.max(...allShapeBounds.map(s => s.maxY));
		
		return {
			x: minX,
			y: minY,
			width: maxX - minX,
			height: maxY - minY
		};
	}

	function renderUnselectedGroupIndicator(ctx: CanvasRenderingContext2D, box: BoundingBox, zoom: number, groupId: number) {
		ctx.save();
		const hue = (groupId * 137.508) % 360;
		const indicatorColor = `hsla(${hue}, 50%, ${$theme === 'dark' ? '60%' : '50%'}, 0.3)`;
		ctx.strokeStyle = indicatorColor;
		ctx.lineWidth = 1.5 / zoom;
		const gap = 4 / zoom;
		ctx.setLineDash([4 / zoom, 4 / zoom]);
		ctx.strokeRect(box.x - gap, box.y - gap, box.width + gap * 2, box.height + gap * 2);
		ctx.setLineDash([]);
		ctx.restore();
	}

	function renderGroupBoundingBox(ctx: CanvasRenderingContext2D, box: BoundingBox, zoom: number, isGroupSelection: boolean = false) {
		ctx.save();
		ctx.strokeStyle = getSelectionOutlineColor();
		ctx.lineWidth = 1 / zoom;
		const gap = 4 / zoom;
		ctx.setLineDash([2.5 / zoom, 2.5 / zoom]);
		ctx.strokeRect(box.x - gap, box.y - gap, box.width + gap * 2, box.height + gap * 2);
		ctx.setLineDash([]);
		ctx.restore();
		renderGroupResizeHandles(ctx, box, zoom);
		
		const outlineX = box.x - gap;
		const outlineY = box.y - gap;
		const outlineWidth = box.width + gap * 2;
		const outlineHeight = box.height + gap * 2;
		renderRotationHandleFromBounds(ctx, { x: outlineX, y: outlineY, width: outlineWidth, height: outlineHeight }, zoom, 0);
	}

	function render() {
		if (!ctx || !canvas) return;
		const renderCtx = ctx;
		
		renderCtx.clearRect(0, 0, canvas.width, canvas.height);
		renderCtx.fillStyle = $theme === 'dark' ? '#1c1917' : '#fafaf9';
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
			$selectedArrows.length +
			$selectedTexts.length +
			$selectedPaths.length +
			$selectedImages.length;
		
		const selectedGroupChildIds = new Set<number>();
		if ($selectedGroups.length > 0) {
			$selectedGroups.forEach(g => {
				getAllShapeIdsInGroup(g).forEach(id => selectedGroupChildIds.add(id));
			});
		}

		const showIndividualHandles = totalSelectionCount <= 1 && selectedGroupChildIds.size === 0;

		const allShapes = [
			...$rectangles.map(r => ({ type: 'rectangle', data: r })),
			...$ellipses.map(e => ({ type: 'ellipse', data: e })),
			...$diamonds.map(d => ({ type: 'diamond', data: d })),
			...$lines.map(l => ({ type: 'line', data: l })),
			...$arrows.map(a => ({ type: 'arrow', data: a })),
			...$texts.map(t => ({ type: 'text', data: t })),
			...$paths.map(p => ({ type: 'path', data: p })),
			...$images.map(i => ({ type: 'image', data: i }))
		];

		allShapes.sort((a, b) => (a.data.z_index || 0) - (b.data.z_index || 0));
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);
		
		allShapes.forEach(item => {
			if (item.type === 'rectangle') {
				const rect = item.data as Rectangle;
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
				
				const strokeColor = adaptColorToTheme(rect.stroke_color, getDefaultStrokeColor());
				const fillColor = rect.fill_color ? adaptColorToTheme(rect.fill_color, rect.fill_color) : null;
				const lineWidth = rect.line_width || 2;
				const dashPattern = rect.dash_pattern || 'solid';
				const borderRadius = rect.border_radius || 0;
				const rotation = getRenderedRotation(rect, 'rectangle');
				
				renderCtx.lineWidth = lineWidth;
				
				if (dashPattern === 'dashed') {
					renderCtx.setLineDash([8 / $zoom, 4 / $zoom]);
				} else if (dashPattern === 'dotted') {
					renderCtx.setLineDash([2 / $zoom, 2 / $zoom]);
				} else {
					renderCtx.setLineDash([]);
				}
				
				renderCtx.save();
				const centerX = renderX + renderWidth / 2;
				const centerY = renderY + renderHeight / 2;
				renderCtx.translate(centerX, centerY);
				renderCtx.rotate(rotation);
				
				const x = -renderWidth / 2;
				const y = -renderHeight / 2;
				const w = renderWidth;
				const h = renderHeight;
				
				if (borderRadius > 0) {
					const r = Math.min(borderRadius, w / 2, h / 2);
					renderCtx.beginPath();
					renderCtx.moveTo(x + r, y);
					renderCtx.lineTo(x + w - r, y);
					renderCtx.quadraticCurveTo(x + w, y, x + w, y + r);
					renderCtx.lineTo(x + w, y + h - r);
					renderCtx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
					renderCtx.lineTo(x + r, y + h);
					renderCtx.quadraticCurveTo(x, y + h, x, y + h - r);
					renderCtx.lineTo(x, y + r);
					renderCtx.quadraticCurveTo(x, y, x + r, y);
					renderCtx.closePath();
				} else {
					renderCtx.beginPath();
					renderCtx.rect(x, y, w, h);
					renderCtx.closePath();
				}
				
				if (fillColor) {
					renderCtx.fillStyle = fillColor;
					renderCtx.fill();
				}
				renderCtx.strokeStyle = strokeColor;
				renderCtx.stroke();
				renderCtx.restore();
				renderCtx.setLineDash([]);
				
				if (isSelected && !selectedGroupChildIds.has(rect.id)) {
					const outlineBounds = getSelectionOutlineBounds(renderX, renderY, renderWidth, renderHeight, $zoom, true);
					renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
					if (showIndividualHandles) {
						renderCornerHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
						renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
					}
				}
			} else if (item.type === 'text') {
				const text = item.data as Text;
				const isSelected = $selectedTexts.some(selected => selected.id === text.id);
				const isDragged = isDragging && isSelected && selectedShapesStartPositions.texts?.has(text.id);

				let renderX: number, renderY: number;
				if (isDragged) {
					const startPos = selectedShapesStartPositions.texts?.get(text.id)!;
					renderX = startPos.x + dragOffset.x;
					renderY = startPos.y + dragOffset.y;
				} else {
					renderX = text.position.x;
					renderY = text.position.y;
				}

				const renderWidth = text.width;
				const renderHeight = text.height;
				const fontSize = text.font_size || 16;
				const fontFamily = text.font_family || 'Arial';
				const fontWeight = text.font_weight || 'normal';
				const textAlign = text.text_align || 'left';
				const color = adaptColorToTheme(text.color, getDefaultStrokeColor());
				const rotation = text.rotation_angle || 0;

				renderCtx.save();
				renderCtx.translate(renderX + renderWidth / 2, renderY + renderHeight / 2);
				renderCtx.rotate(rotation);

				renderCtx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
				renderCtx.fillStyle = color;
				renderCtx.textAlign = textAlign;
				renderCtx.textBaseline = 'middle';

				const textX = textAlign === 'center' ? 0 : textAlign === 'right' ? renderWidth / 2 : -renderWidth / 2;
				const textY = 0;

				renderCtx.fillText(text.content, textX, textY);

				if (isSelected) {
					const outlineBounds = {
						x: -renderWidth / 2,
						y: -renderHeight / 2,
						width: renderWidth,
						height: renderHeight
					};
					renderSelectionOutline(renderCtx, outlineBounds.x, outlineBounds.y, outlineBounds.width, outlineBounds.height, $zoom, true, 0);
					if (showIndividualHandles) {
						renderCornerHandles(renderCtx, outlineBounds.x, outlineBounds.y, outlineBounds.width, outlineBounds.height, $zoom, true, 0);
						renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, 0);
					}
				}

				renderCtx.restore();
			} else if (item.type === 'ellipse') {
				const ellipse = item.data as Ellipse;
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
				
				const strokeColor = adaptColorToTheme(ellipse.stroke_color, getDefaultStrokeColor());
				const fillColor = ellipse.fill_color ? adaptColorToTheme(ellipse.fill_color, ellipse.fill_color) : null;
				const lineWidth = ellipse.line_width || 2;
				const dashPattern = ellipse.dash_pattern || 'solid';
				const rotation = getRenderedRotation(ellipse, 'ellipse');
				
				renderCtx.lineWidth = lineWidth;
				
				if (dashPattern === 'dashed') {
					renderCtx.setLineDash([8 / $zoom, 4 / $zoom]);
				} else if (dashPattern === 'dotted') {
					renderCtx.setLineDash([2 / $zoom, 2 / $zoom]);
				} else {
					renderCtx.setLineDash([]);
				}
				
				renderCtx.save();
				renderCtx.translate(renderX, renderY);
				renderCtx.rotate(rotation);
				renderCtx.beginPath();
				renderCtx.ellipse(0, 0, renderRadiusX, renderRadiusY, 0, 0, 2 * Math.PI);
				
				if (fillColor) {
					renderCtx.fillStyle = fillColor;
					renderCtx.fill();
				}
				
				renderCtx.strokeStyle = strokeColor;
				renderCtx.stroke();
				renderCtx.restore();
				renderCtx.setLineDash([]);
				
				if (isSelected && !selectedGroupChildIds.has(ellipse.id)) {
					const x = renderX - renderRadiusX;
					const y = renderY - renderRadiusY;
					const width = renderRadiusX * 2;
					const height = renderRadiusY * 2;
					const outlineBounds = getSelectionOutlineBounds(x, y, width, height, $zoom, true);
					renderSelectionOutline(renderCtx, x, y, width, height, $zoom, true, rotation);
					if (showIndividualHandles) {
						renderCornerHandles(renderCtx, x, y, width, height, $zoom, true, rotation);
						renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
					}
				}
			} else if (item.type === 'diamond') {
				const diamond = item.data as Diamond;
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
				
				const strokeColor = adaptColorToTheme(diamond.stroke_color, getDefaultStrokeColor());
				const fillColor = diamond.fill_color ? adaptColorToTheme(diamond.fill_color, diamond.fill_color) : null;
				const lineWidth = diamond.line_width || 2;
				const dashPattern = diamond.dash_pattern || 'solid';
				const rotation = getRenderedRotation(diamond, 'diamond');
				
				renderCtx.lineWidth = lineWidth;
				
				if (dashPattern === 'dashed') {
					renderCtx.setLineDash([8 / $zoom, 4 / $zoom]);
				} else if (dashPattern === 'dotted') {
					renderCtx.setLineDash([2 / $zoom, 2 / $zoom]);
				} else {
					renderCtx.setLineDash([]);
				}
				
				const borderRadius = diamond.border_radius || 0;
				renderCtx.save();
				renderCtx.translate(centerX, centerY);
				renderCtx.rotate(rotation);
				
				if (borderRadius > 0) {
					const r = Math.min(borderRadius, halfWidth * 0.3, halfHeight * 0.3);
					const edgeLength = Math.sqrt(halfWidth * halfWidth + halfHeight * halfHeight);
					const ratio = r / edgeLength;
					
					const topCornerX = 0;
					const topCornerY = -halfHeight;
					const rightCornerX = halfWidth;
					const rightCornerY = 0;
					const bottomCornerX = 0;
					const bottomCornerY = halfHeight;
					const leftCornerX = -halfWidth;
					const leftCornerY = 0;
					
					const topStartX = topCornerX - ratio * halfWidth;
					const topStartY = topCornerY + ratio * halfHeight;
					const topEndX = topCornerX + ratio * halfWidth;
					const topEndY = topCornerY + ratio * halfHeight;
					
					const rightStartX = rightCornerX - ratio * halfWidth;
					const rightStartY = rightCornerY - ratio * halfHeight;
					const rightEndX = rightCornerX - ratio * halfWidth;
					const rightEndY = rightCornerY + ratio * halfHeight;
					
					const bottomStartX = bottomCornerX + ratio * halfWidth;
					const bottomStartY = bottomCornerY - ratio * halfHeight;
					const bottomEndX = bottomCornerX - ratio * halfWidth;
					const bottomEndY = bottomCornerY - ratio * halfHeight;
					
					const leftStartX = leftCornerX + ratio * halfWidth;
					const leftStartY = leftCornerY + ratio * halfHeight;
					const leftEndX = leftCornerX + ratio * halfWidth;
					const leftEndY = leftCornerY - ratio * halfHeight;
					
					renderCtx.beginPath();
					renderCtx.moveTo(topStartX, topStartY);
					renderCtx.quadraticCurveTo(topCornerX, topCornerY, topEndX, topEndY);
					renderCtx.lineTo(rightStartX, rightStartY);
					renderCtx.quadraticCurveTo(rightCornerX, rightCornerY, rightEndX, rightEndY);
					renderCtx.lineTo(bottomStartX, bottomStartY);
					renderCtx.quadraticCurveTo(bottomCornerX, bottomCornerY, bottomEndX, bottomEndY);
					renderCtx.lineTo(leftStartX, leftStartY);
					renderCtx.quadraticCurveTo(leftCornerX, leftCornerY, leftEndX, leftEndY);
					renderCtx.closePath();
				} else {
					renderCtx.beginPath();
					renderCtx.moveTo(0, -halfHeight);
					renderCtx.lineTo(halfWidth, 0);
					renderCtx.lineTo(0, halfHeight);
					renderCtx.lineTo(-halfWidth, 0);
					renderCtx.closePath();
				}
				
				if (fillColor) {
					renderCtx.fillStyle = fillColor;
					renderCtx.fill();
				}
				
				renderCtx.strokeStyle = strokeColor;
				renderCtx.stroke();
				renderCtx.restore();
				renderCtx.setLineDash([]);
				
				if (isSelected && !selectedGroupChildIds.has(diamond.id)) {
					const outlineBounds = getSelectionOutlineBounds(renderX, renderY, renderWidth, renderHeight, $zoom, true);
					renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
					if (showIndividualHandles) {
						renderCornerHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
						renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
					}
				}
			} else if (item.type === 'line') {
				const line = item.data as Line;
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
				
				const strokeColor = adaptColorToTheme(line.stroke_color, getDefaultStrokeColor());
				const lineWidth = line.line_width || 2;
				const dashPattern = line.dash_pattern || 'solid';
				
				renderCtx.strokeStyle = strokeColor;
				renderCtx.lineWidth = lineWidth;
				
				if (dashPattern === 'dashed') {
					renderCtx.setLineDash([8 / $zoom, 4 / $zoom]);
				} else if (dashPattern === 'dotted') {
					renderCtx.setLineDash([2 / $zoom, 2 / $zoom]);
				} else {
					renderCtx.setLineDash([]);
				}
				
				renderCtx.beginPath();
				renderCtx.moveTo(renderStartX, renderStartY);
				renderCtx.lineTo(renderEndX, renderEndY);
				renderCtx.stroke();
				renderCtx.setLineDash([]);
				
				if (isSelected && showIndividualHandles && !selectedGroupChildIds.has(line.id)) {
					const handleSize = 8 / $zoom;
					const halfHandle = handleSize / 2;
					
					renderCtx.fillStyle = getHandleFillColor();
					renderCtx.strokeStyle = getHandleStrokeColor();
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
			} else if (item.type === 'arrow') {
				const arrow = item.data as Arrow;
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
				
				const strokeColor = adaptColorToTheme(arrow.stroke_color, getDefaultStrokeColor());
				const lineWidth = arrow.line_width || 2;
				const dashPattern = arrow.dash_pattern || 'solid';
				
				renderCtx.strokeStyle = strokeColor;
				renderCtx.lineWidth = lineWidth;
				
				if (dashPattern === 'dashed') {
					renderCtx.setLineDash([8 / $zoom, 4 / $zoom]);
				} else if (dashPattern === 'dotted') {
					renderCtx.setLineDash([2 / $zoom, 2 / $zoom]);
				} else {
					renderCtx.setLineDash([]);
				}
				
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
				renderCtx.setLineDash([]);
				
				if (isSelected && showIndividualHandles && !selectedGroupChildIds.has(arrow.id)) {
					const handleSize = 8 / $zoom;
					const halfHandle = handleSize / 2;
					
					renderCtx.fillStyle = getHandleFillColor();
					renderCtx.strokeStyle = getHandleStrokeColor();
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
			} else if (item.type === 'path') {
				const path = item.data as Path;
				const isSelected = $selectedPaths.some(selected => selected.id === path.id);
				const isDragged = isDragging && isSelected && selectedShapesStartPositions.paths.has(path.id);
				const isResized = isResizing && resizePreview && resizePreview.type === 'path' && resizePreview.id === path.id;
				const isRotated = isRotating && rotationPreview && rotationPreview.type === 'path' && rotationPreview.id === path.id;
				const rotation = isRotated ? rotationPreview!.angle : (path.rotation_angle ?? 0);
				const strokeColor = adaptColorToTheme(path.stroke_color, getDefaultStrokeColor());
				const lineWidth = path.line_width || 2;
				
				if (path.points.length > 0) {
					let points: Array<{ x: number; y: number }>;
					if (isResized && resizePreview && resizePreview.type === 'path') {
						const pathBounds = getPathBoundingBox(path);
						const preview = resizePreview;
						if (pathBounds && pathBounds.width > 0 && pathBounds.height > 0) {
							const scaleX = preview.width / pathBounds.width;
							const scaleY = preview.height / pathBounds.height;
							points = path.points.map(p => ({
								x: preview.x + (p.x - pathBounds.x) * scaleX,
								y: preview.y + (p.y - pathBounds.y) * scaleY
							}));
						} else {
							points = path.points;
						}
					} else {
						points = isDragged 
							? path.points.map(p => ({ x: p.x + dragOffset.x, y: p.y + dragOffset.y }))
							: path.points;
					}
					
					const pathBounds = getPathBoundingBox({ points });
					if (pathBounds) {
						const centerX = pathBounds.x + pathBounds.width / 2;
						const centerY = pathBounds.y + pathBounds.height / 2;
						
						renderCtx.save();
						renderCtx.translate(centerX, centerY);
						renderCtx.rotate(rotation);
						
						const relativePoints = points.map(p => ({
							x: p.x - centerX,
							y: p.y - centerY
						}));
						
						renderCtx.strokeStyle = strokeColor;
						renderCtx.lineWidth = lineWidth;
						renderCtx.lineCap = 'round';
						renderCtx.lineJoin = 'round';
						renderCtx.beginPath();
						
						if (relativePoints.length === 1) {
							renderCtx.moveTo(relativePoints[0].x, relativePoints[0].y);
							renderCtx.lineTo(relativePoints[0].x, relativePoints[0].y);
						} else if (relativePoints.length === 2) {
							renderCtx.moveTo(relativePoints[0].x, relativePoints[0].y);
							renderCtx.lineTo(relativePoints[1].x, relativePoints[1].y);
						} else if (relativePoints.length > 2) {
							renderCtx.moveTo(relativePoints[0].x, relativePoints[0].y);
							
							for (let i = 0; i < relativePoints.length - 1; i++) {
								const current = relativePoints[i];
								const next = relativePoints[i + 1];
								const midX = (current.x + next.x) / 2;
								const midY = (current.y + next.y) / 2;
								
								if (i === 0) {
									renderCtx.quadraticCurveTo(current.x, current.y, midX, midY);
								} else {
									renderCtx.quadraticCurveTo(current.x, current.y, midX, midY);
								}
							}
							
							const last = relativePoints[relativePoints.length - 1];
							const secondLast = relativePoints[relativePoints.length - 2];
							renderCtx.quadraticCurveTo(secondLast.x, secondLast.y, last.x, last.y);
						}
						
						renderCtx.stroke();
						renderCtx.restore();
						
						if (isSelected && !selectedGroupChildIds.has(path.id)) {
							const minX = pathBounds.x;
							const minY = pathBounds.y;
							const width = pathBounds.width;
							const height = pathBounds.height;
							
							const outlineBounds = getSelectionOutlineBounds(minX, minY, width, height, $zoom, true);
							renderSelectionOutline(renderCtx, minX, minY, width, height, $zoom, true, rotation);
							if (showIndividualHandles) {
								renderCornerHandles(renderCtx, minX, minY, width, height, $zoom, true, rotation);
								renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
							}
						}
					}
				}
			} else if (item.type === 'image') {
				const image = item.data as Image;
				const isSelected = $selectedImages.some(selected => selected.id === image.id);
				const isDragged = isDragging && isSelected && selectedShapesStartPositions.images.has(image.id);
				const isResized = isResizing && resizePreview && resizePreview.type === 'image' && resizePreview.id === image.id;
				
				let renderX: number, renderY: number;
				if (isDragged) {
					const startPos = selectedShapesStartPositions.images.get(image.id)!;
					renderX = startPos.x + dragOffset.x;
					renderY = startPos.y + dragOffset.y;
				} else {
					renderX = image.position.x;
					renderY = image.position.y;
				}
				
				let renderWidth = image.width;
				let renderHeight = image.height;
				if (isResized && resizePreview) {
					renderX = resizePreview.x;
					renderY = resizePreview.y;
					renderWidth = resizePreview.width;
					renderHeight = resizePreview.height;
				}
				const rotation = image.rotation_angle ?? 0;
				
				let img = imageCache.get(image.id);
				if (!img || img.src !== image.image_data) {
					img = new Image();
					img.onload = () => {
						scheduleRender();
					};
					img.src = image.image_data;
					imageCache.set(image.id, img);
				}
				
				if (img.complete && img.naturalWidth > 0) {
					renderCtx.save();
					const centerX = renderX + renderWidth / 2;
					const centerY = renderY + renderHeight / 2;
					renderCtx.translate(centerX, centerY);
					renderCtx.rotate(rotation);
					renderCtx.drawImage(img, -renderWidth / 2, -renderHeight / 2, renderWidth, renderHeight);
					renderCtx.restore();
				}
				
				if (isSelected && !selectedGroupChildIds.has(image.id)) {
					const outlineBounds = getSelectionOutlineBounds(renderX, renderY, renderWidth, renderHeight, $zoom, true);
					renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
					if (showIndividualHandles) {
						renderCornerHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
						renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
					}
				}
			}
		});
		
		if (isCreatingRectangle && previewRect && previewRect.width > 0 && previewRect.height > 0) {
			renderCtx.strokeStyle = getDefaultStrokeColor();
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.strokeRect(previewRect.x, previewRect.y, previewRect.width, previewRect.height);
			renderCtx.globalAlpha = 1.0;
		}
		
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
				renderCtx.strokeStyle = getDefaultStrokeColor();
				renderCtx.lineWidth = 2;
				renderCtx.globalAlpha = 0.5;
				renderCtx.beginPath();
				renderCtx.ellipse(centerX, centerY, radius_x, radius_y, 0, 0, 2 * Math.PI);
				renderCtx.stroke();
				renderCtx.globalAlpha = 1.0;
			}
		}
		
		if (isCreatingDiamond && previewRect && previewRect.width > 0 && previewRect.height > 0) {
			const centerX = previewRect.x + previewRect.width / 2;
			const centerY = previewRect.y + previewRect.height / 2;
			const halfWidth = previewRect.width / 2;
			const halfHeight = previewRect.height / 2;
			
			renderCtx.strokeStyle = getDefaultStrokeColor();
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
		
		if (isCreatingShape && $activeTool === 'line' && lineStart && lineEnd) {
			renderCtx.strokeStyle = getDefaultStrokeColor();
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			renderCtx.moveTo(lineStart.x, lineStart.y);
			renderCtx.lineTo(lineEnd.x, lineEnd.y);
			renderCtx.stroke();
			renderCtx.globalAlpha = 1.0;
		}
		
		if (isCreatingShape && $activeTool === 'arrow' && arrowStart && arrowEnd) {
			renderCtx.strokeStyle = getDefaultStrokeColor();
			renderCtx.lineWidth = 2;
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			renderCtx.moveTo(arrowStart.x, arrowStart.y);
			renderCtx.lineTo(arrowEnd.x, arrowEnd.y);
			renderCtx.stroke();
			renderCtx.globalAlpha = 1.0;
		}

		if (isDrawingFreehand && freehandPoints.length > 0) {
			renderCtx.strokeStyle = getDefaultStrokeColor();
			renderCtx.lineWidth = 2;
			renderCtx.lineCap = 'round';
			renderCtx.lineJoin = 'round';
			renderCtx.globalAlpha = 0.5;
			renderCtx.beginPath();
			
			if (freehandPoints.length === 1) {
				renderCtx.moveTo(freehandPoints[0].x, freehandPoints[0].y);
				renderCtx.lineTo(freehandPoints[0].x, freehandPoints[0].y);
			} else if (freehandPoints.length === 2) {
				renderCtx.moveTo(freehandPoints[0].x, freehandPoints[0].y);
				renderCtx.lineTo(freehandPoints[1].x, freehandPoints[1].y);
			} else if (freehandPoints.length > 2) {
				renderCtx.moveTo(freehandPoints[0].x, freehandPoints[0].y);
				
				for (let i = 0; i < freehandPoints.length - 1; i++) {
					const current = freehandPoints[i];
					const next = freehandPoints[i + 1];
					const midX = (current.x + next.x) / 2;
					const midY = (current.y + next.y) / 2;
					
					if (i === 0) {
						renderCtx.quadraticCurveTo(current.x, current.y, midX, midY);
					} else {
						renderCtx.quadraticCurveTo(current.x, current.y, midX, midY);
					}
				}
				
				const last = freehandPoints[freehandPoints.length - 1];
				const secondLast = freehandPoints[freehandPoints.length - 2];
				renderCtx.quadraticCurveTo(secondLast.x, secondLast.y, last.x, last.y);
			}
			
			renderCtx.stroke();
			renderCtx.globalAlpha = 1.0;
		}
		
		renderCtx.restore();
		
		renderCtx.save();
		renderCtx.translate($viewportOffset.x, $viewportOffset.y);
		renderCtx.scale($zoom, $zoom);

		const groupBoundingBoxRaw =
			totalSelectionCount > 1 
				? (isGroupResizing ? calculateGroupBoundingBox() : (groupResizeCurrentBox ?? calculateGroupBoundingBox()))
				: null;
		const groupBoundingBox =
			groupBoundingBoxRaw && totalSelectionCount > 1
				? expandBoundingBox(groupBoundingBoxRaw, getSelectionPaddingValue($zoom))
				: null;
		const isGroupSelection = $selectedGroups.length > 0;
		if (groupBoundingBox && !isGroupRotating) {
			renderGroupBoundingBox(renderCtx, groupBoundingBox, $zoom, isGroupSelection);
		}

		
		if (isSelectingBox && selectionBoxStart && selectionBoxEnd) {
			const boxX = Math.min(selectionBoxStart.x, selectionBoxEnd.x);
			const boxY = Math.min(selectionBoxStart.y, selectionBoxEnd.y);
			const boxWidth = Math.abs(selectionBoxEnd.x - selectionBoxStart.x);
			const boxHeight = Math.abs(selectionBoxEnd.y - selectionBoxStart.y);
			
			renderCtx.strokeStyle = getSelectionOutlineColor();
			renderCtx.lineWidth = 1 / $zoom;
			renderCtx.setLineDash([5 / $zoom, 5 / $zoom]);
			renderCtx.strokeRect(boxX, boxY, boxWidth, boxHeight);
			renderCtx.setLineDash([]);
		}
		
		renderCtx.restore();
		
		if ($activeTool === 'eraser') {
			const currentPos = eraserPosition || lastMouseWorldPos;
			if (currentPos) {
				const screenPos = worldToScreen(currentPos.x, currentPos.y, $viewportOffset, $zoom);
				const screenRadius = eraserRadius;
				
				if (eraserShadowPosition) {
					const shadowScreenPos = worldToScreen(eraserShadowPosition.x, eraserShadowPosition.y, $viewportOffset, $zoom);
					renderCtx.save();
					renderCtx.globalAlpha = 0.2;
					renderCtx.fillStyle = '#000000';
					renderCtx.beginPath();
					renderCtx.arc(shadowScreenPos.x, shadowScreenPos.y, screenRadius, 0, 2 * Math.PI);
					renderCtx.fill();
					renderCtx.restore();
				}
				
				renderCtx.save();
				renderCtx.strokeStyle = $theme === 'dark' ? '#ffffff' : '#000000';
				renderCtx.lineWidth = 2;
				renderCtx.globalAlpha = 0.8;
				renderCtx.beginPath();
				renderCtx.arc(screenPos.x, screenPos.y, screenRadius, 0, 2 * Math.PI);
				renderCtx.stroke();
				renderCtx.restore();
			}
		}
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
		const dpr = window.devicePixelRatio || 1;
		canvas.width = window.innerWidth * dpr;
		canvas.height = window.innerHeight * dpr;
		ctx = canvas.getContext('2d');
		if (ctx) ctx.scale(dpr, dpr);
	}

	$: {
		if ($activeTool && $activeTool !== 'select' && isDragging && !draggedShape) {
			isDragging = false;
			dragOffset = { x: 0, y: 0 };
		}
	}


	onMount(() => {
		initCanvas();
		window.addEventListener('keydown', handleKeyDown);
		window.addEventListener('keyup', handleKeyUp);
		
		const handleResize = () => {
			if (canvas) {
				const dpr = window.devicePixelRatio || 1;
				canvas.width = window.innerWidth * dpr;
				canvas.height = window.innerHeight * dpr;
				if (ctx) ctx.scale(dpr, dpr);
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
	
	$: {
		const currentImageIds = new Set($images.map(img => img.id));
		for (const [id, img] of imageCache.entries()) {
			if (!currentImageIds.has(id)) {
				imageCache.delete(id);
			}
		}
		for (const image of $images) {
			const cached = imageCache.get(image.id);
			if (!cached || cached.src !== image.image_data) {
				const img = new Image();
				img.onload = () => {
					scheduleRender();
				};
				img.src = image.image_data;
				imageCache.set(image.id, img);
			}
		}
	}
	
	$: if (ctx && canvas && !isCreatingShape) {
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
		$diamonds;
		$selectedDiamonds;
		$paths;
		$selectedPaths;
		$images;
		$selectedImages;
		$groups;
		$selectedGroups;
		$theme;
		scheduleRender();
	}
</script>

<div class="relative w-full h-full bg-stone-50 dark:bg-stone-900">
	<Toolbar />
	<Sidebar bind:this={sidebarRef} bind:canvas bind:ctx />
	<ZoomControls />
	<UndoRedoControls />
	<StylePanel />
	<canvas
		on:mousedown={handleMouseDown}
		on:mousemove={handleMouseMove}
		on:mouseup={handleMouseUp}
		on:mouseleave={handleMouseLeave}
		on:dblclick={handleCanvasDoubleClick}
			on:wheel={(e) => handleViewportScroll(e, canvas!)}
		bind:this={canvas}
			class="w-full h-full bg-stone-50 dark:bg-stone-900"
		tabindex="0"
	></canvas>
</div>

