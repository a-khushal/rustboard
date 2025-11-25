<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { 
		rectangles, selectedRectangles, ellipses, selectedEllipses,
		lines, selectedLines, arrows, selectedArrows,
		diamonds, selectedDiamonds, texts, selectedTexts,
		editorApi, viewportOffset, zoom, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Text
	} from '$lib/stores/editor';
	import { isPointInRectangle, isPointInEllipse, isPointOnLine, isPointInDiamond, isPointInText, rectangleIntersectsBox, ellipseIntersectsBox, lineIntersectsBox, arrowIntersectsBox, diamondIntersectsBox, textIntersectsBox, measureMultilineText, getFontForSize, DEFAULT_TEXT_FONT_SIZE, TEXT_HORIZONTAL_PADDING, TEXT_VERTICAL_PADDING } from '$lib/utils/geometry';
		import { screenToWorld } from '$lib/utils/viewport';
import { 
		addRectangle, moveRectangle, resizeRectangle, setRectangleRotation,
		addEllipse, moveEllipse, resizeEllipse, setEllipseRotation,
		addDiamond, moveDiamond, resizeDiamond, setDiamondRotation,
		addLine, moveLine,
		addArrow, moveArrow,
		addText, moveText, setTextFontSize, setTextBoxWidth, updateTextContent, deleteTextById, setTextRotation
	} from '$lib/utils/canvas-operations/index';
	import { updateTexts } from '$lib/utils/canvas-operations/texts';
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
	import StylePanel from './StylePanel.svelte';
	import { activeTool, type Tool } from '$lib/stores/tools';

	type BoundingBox = { x: number; y: number; width: number; height: number; rawWidth?: number; rawHeight?: number; scaleX?: number; scaleY?: number };
	let canvas: HTMLCanvasElement | undefined;
	let ctx: CanvasRenderingContext2D | null = null;
	let isDragging = false;
	let dragStartPos = { x: 0, y: 0 };
	let draggedShape: Rectangle | Ellipse | Line | Arrow | Diamond | Text | null = null;
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
	let resizeStartShape: Rectangle | Ellipse | Line | Arrow | Diamond | Text | null = null;
	let resizeStartShapeType: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | 'text' | null = null;
	let resizeStartPos = { x: 0, y: 0, width: 0, height: 0 };
	let resizeStartMousePos = { x: 0, y: 0 };
	let isShiftPressedDuringResize = false;
	let dragOffset = { x: 0, y: 0 };
	let resizePreview: { x: number; y: number; width: number; height: number; type: 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | 'text'; id: number; fontSize?: number; baseline?: number } | null = null;
	let resizeStartTextAscent = 0;
	let resizeStartOriginalText: string | null = null;
	let resizeStartBoxWidth: number | null = null;
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
		texts: Map<number, { x: number; y: number; text: string; fontSize: number; rotation: number }>;
	} = {
		rectangles: new Map(),
		ellipses: new Map(),
		diamonds: new Map(),
		lines: new Map(),
		arrows: new Map(),
		texts: new Map()
	};
	let isGroupResizing = false;
	let groupResizeHandleIndex: number | null = null;
	let groupResizeStartBox: BoundingBox | null = null;
	let groupResizeCurrentBox: BoundingBox | null = null;
	let groupResizePadding = 0;
	let groupResizeStartMousePos = { x: 0, y: 0 };
	let renderDependencies: Record<string, unknown> | null = null;
	const resizeCursors = ['nwse-resize', 'nesw-resize', 'nwse-resize', 'nesw-resize', 'ns-resize', 'ew-resize', 'ns-resize', 'ew-resize'];
	const ROTATION_HANDLE_DISTANCE = 18;
	const ROTATION_HANDLE_RADIUS = 7;
	const ROTATION_STEP = Math.PI / 12;
	type RotatableShapeType = 'rectangle' | 'ellipse' | 'diamond' | 'text';
	let isRotating = false;
	let rotationState: { type: RotatableShapeType; id: number; center: { x: number; y: number }; startAngle: number; mouseStartAngle: number } | null = null;
	let rotationPreview: { type: RotatableShapeType; id: number; angle: number } | null = null;

	let isTypingText = false;
	let typingTextId: number | null = null;
	let typingOriginalValue = '';
	let typingValue = '';
	let typingWorldPos: { x: number; y: number } | null = null;
	let typingScreenPos = { x: 0, y: 0 };
	let typingDirty = false;
	let typingFontSize = DEFAULT_TEXT_FONT_SIZE;
	let typingTextColor: string | null = null;
let typingBoxWidth: number | null = null;
	let textInputRef: HTMLTextAreaElement | null = null;
	let typingLayout = measureMultilineText('', DEFAULT_TEXT_FONT_SIZE);

	function getTextContentWidthFromBoxWidth(boxWidth: number | null): number | undefined {
		if (!boxWidth || !isFinite(boxWidth)) return undefined;
		const contentWidth = boxWidth - TEXT_HORIZONTAL_PADDING * 2;
		return contentWidth > 0 ? contentWidth : undefined;
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (isTypingText) {
			if (event.key === 'Escape') {
				event.preventDefault();
				cancelTypingText();
				return;
			}
			if ((event.key === 'Enter' || event.key === 'NumpadEnter') && (event.metaKey || event.ctrlKey)) {
				event.preventDefault();
				commitTypingText();
				return;
			}
			if (event.key === 'Enter' || event.key === 'NumpadEnter') {
				event.preventDefault();
				insertTextAtCursor('\n');
				return;
			}
			return;
		}

		if (event.key === 'Escape' && isRotating) {
			event.preventDefault();
			resetRotationState();
			scheduleRender();
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
			
			if (isTypingText && typingTextId !== null) {
				const delta = isPlus ? 2 : -2;
				const newSize = Math.max(4, typingFontSize + delta);
				typingFontSize = newSize;
				const typingContentWidth = getTextContentWidthFromBoxWidth(typingBoxWidth);
				typingLayout = measureMultilineText(typingValue, typingFontSize, ctx ?? undefined, typingContentWidth);
				setTextFontSize(typingTextId, newSize, false);
				return;
			}

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
			if ($activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond' || $activeTool === 'line' || $activeTool === 'arrow' || $activeTool === 'text') {
				activeTool.set('select');
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && (event.key === 'c' || event.key === 'C')) {
			event.preventDefault();
			if ($selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0) {
				copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds, $selectedTexts);
			}
			return;
		}

		if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'd') {
			event.preventDefault();
			event.stopPropagation();
			if (!$editorApi) return;
			
			const hasSelection = $selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0;
			if (!hasSelection) return;
			
			copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds, $selectedTexts);
			const clipboard = getClipboard();
			
			const bounds: Array<{ minX: number; minY: number }> = [];
			clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
			clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
			clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
			clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
			clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
			clipboard.texts.forEach(t => bounds.push({ minX: t.position.x, minY: t.position.y }));

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
			selectedTexts.set([...$texts]);
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
		const hasSelectedTexts = $selectedTexts.length > 0;

		if (!hasSelectedRectangles && !hasSelectedEllipses && !hasSelectedDiamonds && !hasSelectedLines && !hasSelectedArrows && !hasSelectedTexts) return;

		event.preventDefault();
		
		const rectangleIds = hasSelectedRectangles ? $selectedRectangles.map(rect => rect.id) : [];
		const ellipseIds = hasSelectedEllipses ? $selectedEllipses.map(ellipse => ellipse.id) : [];
		const diamondIds = hasSelectedDiamonds ? $selectedDiamonds.map(diamond => diamond.id) : [];
		const lineIds = hasSelectedLines ? $selectedLines.map(line => line.id) : [];
		const arrowIds = hasSelectedArrows ? $selectedArrows.map(arrow => arrow.id) : [];
		const textIds = hasSelectedTexts ? $selectedTexts.map(text => text.id) : [];
		
		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, textIds);
	}

	function handleKeyUp(event: KeyboardEvent) {
		if (event.key === ' ') {
			isSpacePressed = false;
		}
	}

	function getResizeHandleAt(
		x: number,
		y: number,
		shape: Rectangle | Ellipse | Diamond | Text,
		shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'text',
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
		} else if (shapeType === 'text') {
			const text = shape as Text;
			const contentWidth = getTextContentWidthFromBoxWidth(text.boxWidth ?? null);
			const layout = measureMultilineText(
				text.text,
				text.fontSize ?? DEFAULT_TEXT_FONT_SIZE,
				ctx ?? undefined,
				contentWidth
			);
			const horizontalPadding = TEXT_HORIZONTAL_PADDING;
			const verticalPadding = TEXT_VERTICAL_PADDING;
			const selectionWidth = text.boxWidth ?? (layout.width + horizontalPadding * 2);
			const selectionHeight = layout.height + verticalPadding * 2;
			boxX = text.position.x - horizontalPadding - gap;
			boxY = text.position.y - layout.ascent - verticalPadding - gap;
			boxWidth = selectionWidth + gap * 2;
			boxHeight = selectionHeight + gap * 2;
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

	async function startTypingSession(
		id: number,
		initialValue: string,
		originalValue: string,
		worldPosition: { x: number; y: number },
		selectAll: boolean,
		fontSize: number,
		boxWidth: number | null = null
	) {
		const savedViewportOffset = { ...$viewportOffset };
		const savedScrollX = window.scrollX;
		const savedScrollY = window.scrollY;
		
		const restoreViewport = () => {
			viewportOffset.set(savedViewportOffset);
			window.scrollTo(savedScrollX, savedScrollY);
		};
		
		typingTextId = id;
		typingValue = initialValue;
		typingOriginalValue = originalValue;
		typingWorldPos = { ...worldPosition };
		typingDirty = initialValue !== originalValue;
		typingFontSize = fontSize;
		typingBoxWidth = boxWidth;
		const typingContentWidth = getTextContentWidthFromBoxWidth(typingBoxWidth);
		typingLayout = measureMultilineText(initialValue || '', typingFontSize, ctx ?? undefined, typingContentWidth);
		isTypingText = true;
		
		restoreViewport();
		
		await tick();
		restoreViewport();
		
		if (textInputRef) {
			restoreViewport();
			textInputRef.focus({ preventScroll: true });
			restoreViewport();
			
			if (selectAll) {
				textInputRef.select();
			} else {
				const caretPos = typingValue.length;
				textInputRef.setSelectionRange(caretPos, caretPos);
			}
			
			restoreViewport();
			
			const checkInterval = setInterval(() => {
				if (!textInputRef || !isTypingText) {
					clearInterval(checkInterval);
					return;
				}
				if (window.scrollX !== savedScrollX || window.scrollY !== savedScrollY) {
					window.scrollTo(savedScrollX, savedScrollY);
				}
				if ($viewportOffset.x !== savedViewportOffset.x || $viewportOffset.y !== savedViewportOffset.y) {
					viewportOffset.set(savedViewportOffset);
				}
			}, 10);
			
			setTimeout(() => clearInterval(checkInterval), 500);
		}
	}

	function resetTypingState() {
		isTypingText = false;
		typingTextId = null;
		typingOriginalValue = '';
		typingValue = '';
		typingWorldPos = null;
		typingDirty = false;
		typingFontSize = DEFAULT_TEXT_FONT_SIZE;
		typingTextColor = null;
		typingBoxWidth = null;
		typingLayout = measureMultilineText('', typingFontSize, ctx ?? undefined);
	}

	function handleTextInputInput(event: Event) {
		if (!isTypingText || typingTextId === null) return;
		const target = event.target as HTMLInputElement | HTMLTextAreaElement;
		const newValue = target.value;
		typingValue = newValue;
		typingDirty = newValue !== typingOriginalValue;
		updateTextContent(typingTextId, newValue, false);
		
		if (typingTextColor && typingTextColor !== '#000000' && $editorApi) {
			const updatedTexts = $texts;
			const updatedText = updatedTexts.find(t => t.id === typingTextId);
			if (updatedText && (!updatedText.text_color || updatedText.text_color === '#000000')) {
				$editorApi.set_text_color(BigInt(typingTextId), typingTextColor, false);
				updateTexts();
			}
		}
	}

	function insertTextAtCursor(value: string) {
		if (!textInputRef || typingTextId === null) return;
		const start = textInputRef.selectionStart ?? typingValue.length;
		const end = textInputRef.selectionEnd ?? typingValue.length;
		const newValue = typingValue.slice(0, start) + value + typingValue.slice(end);
		typingValue = newValue;
		textInputRef.value = newValue;
		typingDirty = newValue !== typingOriginalValue;
		updateTextContent(typingTextId, newValue, false);
		const cursor = start + value.length;
		setTimeout(() => {
			textInputRef?.setSelectionRange(cursor, cursor);
		}, 0);
	}

	function handleTextInputKeyDown(event: KeyboardEvent) {
		if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
			event.preventDefault();
			commitTypingText();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			cancelTypingText();
		} else if ((event.ctrlKey || event.metaKey) && (event.key === '+' || event.key === '=' || event.key === '-')) {
			event.preventDefault();
			const isPlus = event.key === '+' || event.key === '=';
			if (isPlus) {
				zoomIn();
			} else {
				zoomOut();
			}
		}
	}

	function commitTypingText() {
		if (!isTypingText || typingTextId === null) return;
		const currentId = typingTextId;
		const finalValue = typingValue;
		const originalValue = typingOriginalValue;
		const wasDirty = typingDirty;

		if (finalValue.replace(/\s+/g, '') === '') {
			const saveHistory = originalValue.trim().length > 0;
			deleteTextById(currentId, saveHistory);
			resetTypingState();
			return;
		}

		if (wasDirty) {
			updateTextContent(currentId, finalValue, false);
			if ($editorApi) {
				$editorApi.save_snapshot();
			}
		}

		resetTypingState();
	}

	function cancelTypingText() {
		if (!isTypingText || typingTextId === null) return;
		const currentId = typingTextId;
		if (typingOriginalValue.trim() === '') {
			deleteTextById(currentId, false);
		} else if (typingDirty) {
			updateTextContent(currentId, typingOriginalValue, false);
		}
		resetTypingState();
	}

	function startTypingExistingText(text: Text) {
		resetRotationState();
		clearAllSelections();
		typingTextColor = text.text_color || '#000000';
		
		if (typingTextColor && typingTextColor !== '#000000' && $editorApi) {
			$editorApi.set_text_color(BigInt(text.id), typingTextColor, false);
			updateTexts();
		}
		
		startTypingSession(
			text.id,
			text.text,
			text.text,
			{ x: text.position.x, y: text.position.y },
			true,
			text.fontSize ?? DEFAULT_TEXT_FONT_SIZE,
			text.boxWidth ?? null
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
	shape: Rectangle | Ellipse | Diamond | Text,
	shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'text',
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
	const text = shape as Text;
	const fontSize = text.fontSize ?? DEFAULT_TEXT_FONT_SIZE;
	const contentWidth = getTextContentWidthFromBoxWidth(text.boxWidth ?? null);
	const layout = measureMultilineText(text.text, fontSize, ctxOverride ?? undefined, contentWidth);
	const horizontalPadding = TEXT_HORIZONTAL_PADDING;
	const verticalPadding = TEXT_VERTICAL_PADDING;
	const width = (text.boxWidth ?? layout.width + horizontalPadding * 2);
	const height = layout.height + verticalPadding * 2;
	const x = text.position.x - horizontalPadding;
	const y = text.position.y - layout.ascent - verticalPadding;
	return { x, y, width, height };
}

function getSelectionBoundsForShape(
	shape: Rectangle | Ellipse | Diamond | Text,
	shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'text',
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
	shape: Rectangle | Ellipse | Diamond | Text,
	shapeType: RotatableShapeType,
	ctxOverride: CanvasRenderingContext2D | null = ctx
): { x: number; y: number } | null {
	const bounds = getShapeBoundingBox(shape, shapeType, ctxOverride);
	if (!bounds) return null;
	return { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
}

function getShapeRotation(shape: Rectangle | Ellipse | Diamond | Text, shapeType: RotatableShapeType): number {
	if (shapeType === 'rectangle') {
		return (shape as Rectangle).rotation_angle ?? 0;
	}
	if (shapeType === 'ellipse') {
		return (shape as Ellipse).rotation_angle ?? 0;
	}
	if (shapeType === 'diamond') {
		return (shape as Diamond).rotation_angle ?? 0;
	}
	return (shape as Text).rotation_angle ?? 0;
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
		ctx.fillStyle = '#ffffff';
		ctx.beginPath();
		ctx.arc(handle.x, handle.y, radius, 0, Math.PI * 2);
		ctx.fill();
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 1 / zoom;
		ctx.stroke();
		ctx.restore();
	}

	function isPointOnRotationHandle(
		x: number,
		y: number,
		shape: Rectangle | Ellipse | Diamond | Text,
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
	shape: Rectangle | Ellipse | Diamond | Text,
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
	setTextRotation(id, normalized, saveHistory);
}

function beginRotation(
	shape: Rectangle | Ellipse | Diamond | Text,
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

function rotateSelectedShapes(delta: number) {
	const targets: Array<{ type: RotatableShapeType; shape: Rectangle | Ellipse | Diamond | Text }> = [];
	$selectedRectangles.forEach(rect => targets.push({ type: 'rectangle', shape: rect }));
	$selectedEllipses.forEach(ellipse => targets.push({ type: 'ellipse', shape: ellipse }));
	$selectedDiamonds.forEach(diamond => targets.push({ type: 'diamond', shape: diamond }));
	$selectedTexts.forEach(text => targets.push({ type: 'text', shape: text }));
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

	function storeSelectedShapesStartPositions() {
		selectedShapesStartPositions.rectangles.clear();
		selectedShapesStartPositions.ellipses.clear();
		selectedShapesStartPositions.diamonds.clear();
		selectedShapesStartPositions.lines.clear();
		selectedShapesStartPositions.arrows.clear();
		selectedShapesStartPositions.texts.clear();
		
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
		selectedShapesStartPositions.texts.set(text.id, { x: text.position.x, y: text.position.y, text: text.text, fontSize: text.fontSize ?? DEFAULT_TEXT_FONT_SIZE, rotation: text.rotation_angle ?? 0 });
		});
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

		ctx.fillStyle = '#ffffff';
		ctx.strokeStyle = '#1e88e5';
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

		selectedShapesStartPositions.texts.forEach((startPos, id) => {
			const relativeX = startPos.x - startBox.x;
			const relativeY = startPos.y - startBox.y;
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

			const newFontSize = (startPos.fontSize ?? DEFAULT_TEXT_FONT_SIZE) * Math.max(absScaleX, absScaleY);
			moveText(id, newX, newY, saveHistory);
			setTextFontSize(id, Math.max(4, newFontSize), saveHistory);
		});
	}

	function handleShapeClick(
		shape: Rectangle | Ellipse | Diamond | Text,
		shapeType: 'rectangle' | 'ellipse' | 'diamond' | 'text',
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
				selectedTexts.set([]);
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
				selectedTexts.set([]);
			}

			draggedShape = clickedEllipse;
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
			}

			draggedShape = clickedText;
			dragStartPos = { x, y };
			dragOffset = { x: 0, y: 0 };
			storeSelectedShapesStartPositions();
			isDragging = true;
		}
	}

	function handleMouseDown(event: MouseEvent) {
		if (!canvas) return;

		event.preventDefault();
		canvas.focus({ preventScroll: true });

		if (isTypingText) {
			commitTypingText();
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

		if (event.detail === 2 && ctx) {
			event.preventDefault();
			for (let i = $texts.length - 1; i >= 0; i--) {
				if (isPointInText(x, y, $texts[i], ctx)) {
					startTypingExistingText($texts[i]);
					return;
				}
			}
		}

		const isShiftPressed = event.shiftKey;
		resizeStartBoxWidth = null;
		
		if ($activeTool === 'select') {
			const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length + $selectedTexts.length;
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

				if (allowIndividualHandles && ctx) {
					for (let i = $selectedTexts.length - 1; i >= 0; i--) {
						if (isPointOnRotationHandle(x, y, $selectedTexts[i], 'text', $zoom, ctx)) {
							if (beginRotation($selectedTexts[i], 'text', x, y)) {
								return;
							}
						}
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedTexts[i],
							'text',
							$zoom,
							getRenderedRotation($selectedTexts[i], 'text')
						);
						if (handleIndex !== null) {
							isResizing = true;
							resizeHandleIndex = handleIndex;
							resizeStartShape = $selectedTexts[i];
							resizeStartShapeType = 'text';
							const fontSize = $selectedTexts[i].fontSize ?? DEFAULT_TEXT_FONT_SIZE;
							const layout = measureMultilineText($selectedTexts[i].text, fontSize, ctx);
							const width = layout.width;
							const height = layout.height;
							resizeStartPos = {
								x: $selectedTexts[i].position.x,
								y: $selectedTexts[i].position.y,
								width,
								height
							};
							resizeStartTextAscent = layout.ascent;
							const currentText = $selectedTexts[i].text;
							const lines = currentText.split('\n');
							resizeStartOriginalText = lines.join(' ');
							const storedBoxWidth = $selectedTexts[i].boxWidth;
							const currentBoxWidth = storedBoxWidth ?? (layout.width + TEXT_HORIZONTAL_PADDING * 2);
							resizeStartBoxWidth = currentBoxWidth;
							resizeStartMousePos = { x, y };
							isShiftPressedDuringResize = isShiftPressed;
							return;
						}
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
						selectedTexts.set([]);
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
						selectedTexts.set([]);
					}
					
					draggedShape = clickedArrow;
					dragStartPos = { x, y };
					dragOffset = { x: 0, y: 0 };
					storeSelectedShapesStartPositions();
				isDragging = true;
					return;
				}
			}

			for (let i = $texts.length - 1; i >= 0; i--) {
				if (ctx && isPointInText(x, y, $texts[i], ctx)) {
					handleShapeClick($texts[i], 'text', isShiftPressed, x, y);
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
					const firstSelectedText = $selectedTexts[0];
					
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
					} else if (firstSelectedText) {
						draggedShape = firstSelectedText;
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
		} else if ($activeTool === 'text') {
			resetRotationState();
			clearAllSelections();
			const newId = addText(x, y, '', false);
			if (newId !== null) {
				startTypingSession(newId, '', '', { x, y }, false, DEFAULT_TEXT_FONT_SIZE, null);
			}
			activeTool.set('select');
			scheduleRender();
		}
	}

	function handleMouseMove(event: MouseEvent) {
		if (!canvas) return;
		if (isTypingText) return;

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
			} else if (resizeStartShapeType === 'text') {
			const text = resizeStartShape as Text;
			const isCornerHandle = resizeHandleIndex === 0 || resizeHandleIndex === 1 || resizeHandleIndex === 2 || resizeHandleIndex === 3;
			const isSideHandle = resizeHandleIndex === 5 || resizeHandleIndex === 7;
			const affectsLeft = resizeHandleIndex === 0 || resizeHandleIndex === 3 || resizeHandleIndex === 7;
			const affectsRight = resizeHandleIndex === 1 || resizeHandleIndex === 2 || resizeHandleIndex === 5;
			const affectsTop = resizeHandleIndex === 0 || resizeHandleIndex === 1 || resizeHandleIndex === 4;
			const affectsBottom = resizeHandleIndex === 2 || resizeHandleIndex === 3 || resizeHandleIndex === 6;
			const adjustsHorizontal = affectsLeft || affectsRight;
			const adjustsVertical = affectsTop || affectsBottom;

			const horizontalPadding = TEXT_HORIZONTAL_PADDING;
			const startTextX = resizeStartPos.x;
			const startTextY = resizeStartPos.y;
			const startBoxLeft = startTextX - horizontalPadding;
			const startBoxWidth = resizeStartBoxWidth ?? (resizeStartPos.width + horizontalPadding * 2);
			const startBoxRight = startBoxLeft + startBoxWidth;
			const startBoxTop = startTextY - resizeStartTextAscent;
			const startBoxHeight = resizeStartPos.height;
			const startBoxBottom = startBoxTop + startBoxHeight;

			const minWidth = horizontalPadding * 2 + 10;
			const minHeight = 10;

			let left = startBoxLeft;
			let right = startBoxRight;
			let top = startBoxTop;
			let bottom = startBoxBottom;

			if (isSideHandle) {
				if (affectsLeft) {
					left = Math.min(startBoxLeft + deltaX, startBoxRight - minWidth);
					right = startBoxRight;
				} else if (affectsRight) {
					left = startBoxLeft;
					right = Math.max(startBoxRight + deltaX, startBoxLeft + minWidth);
				}
				top = startBoxTop;
				bottom = startBoxBottom;
			} else {
				if (affectsLeft) left = startBoxLeft + deltaX;
				if (affectsRight) right = startBoxRight + deltaX;
				if (affectsTop) top = startBoxTop + deltaY;
				if (affectsBottom) bottom = startBoxBottom + deltaY;

				if (isShiftPressedDuringResize && isCornerHandle && adjustsHorizontal && adjustsVertical && startBoxHeight !== 0) {
					const aspectRatio = startBoxWidth / startBoxHeight;
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
			}

			let finalLeft = Math.min(left, right);
			let finalRight = Math.max(left, right);
			let finalTop = Math.min(top, bottom);
			let finalBottom = Math.max(top, bottom);

			const draggedLeft = affectsLeft && !affectsRight;
			const draggedRight = affectsRight && !affectsLeft;
			const draggedTop = affectsTop && !affectsBottom;
			const draggedBottom = affectsBottom && !affectsTop;

			let newWidth = finalRight - finalLeft;
			if (newWidth < minWidth) {
				if (draggedLeft) {
					finalLeft = finalRight - minWidth;
				} else if (draggedRight) {
					finalRight = finalLeft + minWidth;
				} else {
					finalRight = finalLeft + minWidth;
				}
				newWidth = minWidth;
			}

			let newHeight = finalBottom - finalTop;
			if (!isSideHandle && newHeight < minHeight) {
				if (draggedTop) {
					finalTop = finalBottom - minHeight;
				} else if (draggedBottom) {
					finalBottom = finalTop + minHeight;
				} else {
					finalBottom = finalTop + minHeight;
				}
				newHeight = minHeight;
			}

			const newTextX = finalLeft + horizontalPadding;

			if (isSideHandle) {
				const fontSize = text.fontSize ?? DEFAULT_TEXT_FONT_SIZE;
				const textWidth = Math.max(10, newWidth - horizontalPadding * 2);
				const textToWrap = resizeStartOriginalText ?? text.text;

				const unwrappedLayout = measureMultilineText(textToWrap, fontSize, ctx ?? undefined);
				const needsWrapping = unwrappedLayout.width > textWidth;

				const previewLayout = needsWrapping
					? measureMultilineText(textToWrap, fontSize, ctx ?? undefined, textWidth)
					: unwrappedLayout;
				const newBaseline = finalTop + previewLayout.ascent;

				resizePreview = {
					x: newTextX,
					y: finalTop,
					width: newWidth,
					height: previewLayout.height,
					type: 'text',
					id: text.id,
					fontSize: fontSize,
					baseline: newBaseline
				};
			} else {
				const startWidth = Math.max(1, resizeStartPos.width);
				const startHeight = Math.max(1, resizeStartPos.height);
				const widthScale = newWidth / startWidth;
				const heightScale = newHeight / startHeight;
				const scale = Math.max(Math.abs(widthScale), Math.abs(heightScale));
				const newFontSize = Math.max(4, (text.fontSize ?? DEFAULT_TEXT_FONT_SIZE) * (isFinite(scale) ? scale : 1));
				const previewLayout = measureMultilineText(text.text, newFontSize, ctx ?? undefined);
				const newBaseline = finalTop + previewLayout.ascent;

				resizePreview = {
					x: newTextX,
					y: finalTop,
					width: newWidth,
					height: newHeight,
					type: 'text',
					id: text.id,
					fontSize: newFontSize,
					baseline: newBaseline
				};
			}
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
		} else if ($activeTool === 'text') {
			canvas.style.cursor = 'text';
		} else if ($activeTool === 'select') {
			if (isResizing) {
				canvas.style.cursor = resizeHandleIndex !== null ? resizeCursors[resizeHandleIndex] : 'default';
			} else if (isDragging && draggedShape) {
				canvas.style.cursor = 'move';
			} else {
				const totalSelectedCount = $selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + $selectedLines.length + $selectedArrows.length + $selectedTexts.length;
				const selectionPaddingValue = getSelectionPaddingValue($zoom);
				const rawGroupBoxForCursor =
					totalSelectedCount > 1 ? groupResizeCurrentBox ?? calculateGroupBoundingBox() : null;
				const visualGroupBoxForCursor = rawGroupBoxForCursor
					? expandBoundingBox(rawGroupBoxForCursor, selectionPaddingValue)
					: null;

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
					if (ctx) {
						for (let i = $selectedTexts.length - 1; i >= 0; i--) {
							if (isPointOnRotationHandle(x, y, $selectedTexts[i], 'text', $zoom, ctx)) {
								canvas.style.cursor = 'grab';
								return;
							}
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
					
					for (let i = $selectedTexts.length - 1; i >= 0; i--) {
						const handleIndex = getResizeHandleAt(
							x,
							y,
							$selectedTexts[i],
							'text',
							$zoom,
							getRenderedRotation($selectedTexts[i], 'text')
						);
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
			if (ctx) {
				for (let i = $texts.length - 1; i >= 0; i--) {
					if (isPointInText(x, y, $texts[i], ctx)) {
						canvas.style.cursor = 'move';
						return;
					}
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
		if (isTypingText) return;
		event.preventDefault();

		const rect = canvas.getBoundingClientRect();
		const screenX = event.clientX - rect.left;
		const screenY = event.clientY - rect.top;
		const { x, y } = screenToWorld(screenX, screenY, $viewportOffset, $zoom);

		for (let i = $texts.length - 1; i >= 0; i--) {
			if (isPointInText(x, y, $texts[i], ctx)) {
				startTypingExistingText($texts[i]);
				return;
			}
		}
	}
	
	function handleMouseUp() {
		if (isPanning) {
			isPanning = false;
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
			} else if (resizePreview.type === 'text') {
				const preview = resizePreview;
				const targetY = preview.baseline ?? resizeStartPos.y;
				const text = $texts.find(t => t.id === preview.id);
				let shouldDeferSnapshot = false;
				if (text && ctx) {
					const originalFontSize = text.fontSize ?? DEFAULT_TEXT_FONT_SIZE;
					const isSideResize = preview.fontSize === originalFontSize && preview.width !== undefined;
					
					if (isSideResize && preview.width) {
						const horizontalPadding = TEXT_HORIZONTAL_PADDING;
						const textWidth = Math.max(10, preview.width - horizontalPadding * 2);
						const textToWrap = resizeStartOriginalText ?? text.text;
						
						const unwrappedLayout = measureMultilineText(textToWrap, originalFontSize, ctx);
						const needsWrapping = unwrappedLayout.width > textWidth;
						
						const finalText = needsWrapping 
							? measureMultilineText(textToWrap, originalFontSize, ctx, textWidth).lines.join('\n')
							: textToWrap;
						
						if (finalText !== text.text) {
							updateTextContent(preview.id, finalText, false);
						}
						setTextBoxWidth(preview.id, preview.width, false);
						resizeStartOriginalText = null;
						shouldDeferSnapshot = true;
					} else if (preview.fontSize && preview.fontSize !== originalFontSize) {
						setTextFontSize(preview.id, preview.fontSize, false);
					}
				}
				moveText(preview.id, preview.x, targetY, !shouldDeferSnapshot);
				if (shouldDeferSnapshot) {
					$editorApi.save_snapshot();
				}
			}
			resizePreview = null;
			resizeStartBoxWidth = null;
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
				const selectedTextsArray: Text[] = [];
				
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
				
				$texts.forEach(text => {
					if (textIntersectsBox(text, box, ctx ?? undefined)) {
						selectedTextsArray.push(text);
					}
				});
				
				selectedRectangles.set(selectedRects);
				selectedEllipses.set(selectedElls);
				selectedDiamonds.set(selectedDias);
				selectedLines.set(selectedLinesArray);
				selectedArrows.set(selectedArrs);
				selectedTexts.set(selectedTextsArray);
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
			resizeStartTextAscent = 0;
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
			
			selectedShapesStartPositions.texts.forEach((startPos, id) => {
				const newX = startPos.x + dragOffset.x;
				const newY = startPos.y + dragOffset.y;
				moveText(id, newX, newY, false);
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
		const gap = 2 / zoom;
		
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
		
		const handles = getHandlePositions({ x: boxX, y: boxY, width: boxWidth, height: boxHeight });
		
		// Only render corner handles (indices 0-3), but keep all handles for hit-testing
		handles.slice(0, 4).forEach((handle) => {
			ctx.beginPath();
			ctx.rect(handle.x - halfHandle, handle.y - halfHandle, handleSize, handleSize);
			ctx.fill();
			ctx.stroke();
		});
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
		ctx.strokeStyle = '#1e88e5';
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
		
		ctx.fillStyle = '#ffffff';
		ctx.strokeStyle = '#1e88e5';
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

	function renderTextHandles(
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
		
		ctx.fillStyle = '#ffffff';
		ctx.strokeStyle = '#1e88e5';
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
		
		if (ctx) {
			$selectedTexts.forEach(text => {
				const isDragged = isDragging && selectedShapesStartPositions.texts.has(text.id);
				let x: number, y: number;
				if (isDragged) {
					const startPos = selectedShapesStartPositions.texts.get(text.id)!;
					x = startPos.x + dragOffset.x;
					y = startPos.y + dragOffset.y;
				} else {
					x = text.position.x;
					y = text.position.y;
				}
				const contentWidth = getTextContentWidthFromBoxWidth(text.boxWidth ?? null);
				const layout = measureMultilineText(
					text.text,
					text.fontSize ?? DEFAULT_TEXT_FONT_SIZE,
					ctx ?? undefined,
					contentWidth
				);
				const horizontalPadding = TEXT_HORIZONTAL_PADDING;
				const verticalPadding = TEXT_VERTICAL_PADDING;
				const boxX = x - horizontalPadding;
				const boxY = y - layout.ascent - verticalPadding;
				const boxWidth = (text.boxWidth ?? (layout.width + horizontalPadding * 2));
				const boxHeight = layout.height + verticalPadding * 2;
				allSelectedShapes.push({
					minX: boxX,
					minY: boxY,
					maxX: boxX + boxWidth,
					maxY: boxY + boxHeight
				});
			});
		}
		
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
		ctx.save();
		ctx.strokeStyle = '#1e88e5';
		ctx.lineWidth = 1 / zoom;
		const gap = 4 / zoom;
		ctx.setLineDash([2.5 / zoom, 2.5 / zoom]);
		ctx.strokeRect(box.x - gap, box.y - gap, box.width + gap * 2, box.height + gap * 2);
		ctx.setLineDash([]);
		ctx.restore();
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
			$selectedArrows.length +
			$selectedTexts.length;
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
			
			const strokeColor = rect.stroke_color || '#000000';
			const fillColor = rect.fill_color;
			const lineWidth = rect.line_width || 2;
			const rotation = getRenderedRotation(rect, 'rectangle');
			
			renderCtx.lineWidth = lineWidth;
			renderCtx.save();
			const centerX = renderX + renderWidth / 2;
			const centerY = renderY + renderHeight / 2;
			renderCtx.translate(centerX, centerY);
			renderCtx.rotate(rotation);
			if (fillColor) {
				renderCtx.fillStyle = fillColor;
				renderCtx.fillRect(-renderWidth / 2, -renderHeight / 2, renderWidth, renderHeight);
			}
			renderCtx.strokeStyle = strokeColor;
			renderCtx.strokeRect(-renderWidth / 2, -renderHeight / 2, renderWidth, renderHeight);
			renderCtx.restore();
			
			if (isSelected) {
				const outlineBounds = getSelectionOutlineBounds(renderX, renderY, renderWidth, renderHeight, $zoom, true);
				renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
				if (showIndividualHandles) {
					renderCornerHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
					renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
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
			
			const strokeColor = ellipse.stroke_color || '#000000';
			const fillColor = ellipse.fill_color;
			const lineWidth = ellipse.line_width || 2;
			const rotation = getRenderedRotation(ellipse, 'ellipse');
			
			renderCtx.lineWidth = lineWidth;
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
			
			if (isSelected) {
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
			
			const strokeColor = diamond.stroke_color || '#000000';
			const fillColor = diamond.fill_color;
			const lineWidth = diamond.line_width || 2;
			const rotation = getRenderedRotation(diamond, 'diamond');
			
			renderCtx.lineWidth = lineWidth;
			renderCtx.save();
			renderCtx.translate(centerX, centerY);
			renderCtx.rotate(rotation);
			renderCtx.beginPath();
			renderCtx.moveTo(0, -halfHeight);
			renderCtx.lineTo(halfWidth, 0);
			renderCtx.lineTo(0, halfHeight);
			renderCtx.lineTo(-halfWidth, 0);
			renderCtx.closePath();
			
			if (fillColor) {
				renderCtx.fillStyle = fillColor;
				renderCtx.fill();
			}
			
			renderCtx.strokeStyle = strokeColor;
			renderCtx.stroke();
			renderCtx.restore();
			
			if (isSelected) {
				const outlineBounds = getSelectionOutlineBounds(renderX, renderY, renderWidth, renderHeight, $zoom, true);
				renderSelectionOutline(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
				if (showIndividualHandles) {
					renderCornerHandles(renderCtx, renderX, renderY, renderWidth, renderHeight, $zoom, true, rotation);
					renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
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
			
			const strokeColor = line.stroke_color || '#000000';
			const lineWidth = line.line_width || 2;
			
			renderCtx.strokeStyle = strokeColor;
			renderCtx.lineWidth = lineWidth;
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
			
			const strokeColor = arrow.stroke_color || '#000000';
			const lineWidth = arrow.line_width || 2;
			
			renderCtx.strokeStyle = strokeColor;
			renderCtx.lineWidth = lineWidth;
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
		
		$texts.forEach((text: Text) => {
			if (isTypingText && text.id === typingTextId) return;
			const isSelected = $selectedTexts.some(selected => selected.id === text.id);
			const isDragged = isDragging && isSelected && selectedShapesStartPositions.texts.has(text.id);
			const isResizedText = isResizing && resizePreview && resizePreview.type === 'text' && resizePreview.id === text.id && resizePreview.fontSize;
			
			let renderX: number, renderY: number;
			if (isDragged) {
				const startPos = selectedShapesStartPositions.texts.get(text.id)!;
				renderX = startPos.x + dragOffset.x;
				renderY = startPos.y + dragOffset.y;
			} else {
				renderX = text.position.x;
				renderY = text.position.y;
			}
			if (isResizedText) {
				renderX = resizePreview!.x;
				renderY = resizePreview!.baseline ?? renderY;
			}

			const baseFontSize = text.fontSize ?? DEFAULT_TEXT_FONT_SIZE;
			const fontSize = isResizedText ? resizePreview!.fontSize! : baseFontSize;
			const textColor = text.text_color || '#000000';
			renderCtx.fillStyle = textColor;
			renderCtx.font = getFontForSize(fontSize);
			const rotation = getRenderedRotation(text, 'text');
			
			let layout;
			let constrainedWidth: number | undefined = undefined;
			const storedBoxWidth = text.boxWidth ?? null;
			if (isResizedText && resizePreview!.width) {
				constrainedWidth = resizePreview!.width;
				const textWidth = Math.max(10, resizePreview!.width - TEXT_HORIZONTAL_PADDING * 2);
				const unwrappedLayout = measureMultilineText(text.text, fontSize, renderCtx);
				const needsWrapping = unwrappedLayout.width > textWidth;
				layout = needsWrapping 
					? measureMultilineText(text.text, fontSize, renderCtx, textWidth)
					: unwrappedLayout;
			} else {
				const contentWidth = getTextContentWidthFromBoxWidth(storedBoxWidth);
				if (contentWidth) {
					constrainedWidth = storedBoxWidth ?? undefined;
					layout = measureMultilineText(text.text, fontSize, renderCtx, contentWidth);
				} else {
					layout = measureMultilineText(text.text, fontSize, renderCtx);
				}
			}

			const horizontalPadding = TEXT_HORIZONTAL_PADDING;
			const verticalPadding = TEXT_VERTICAL_PADDING;
			const textTop = renderY - layout.ascent;
			const textBottom = renderY + layout.descent + (layout.lines.length - 1) * layout.lineHeight;
			const boxX = renderX - horizontalPadding;
			const boxY = textTop - verticalPadding;
			const selectionWidth = constrainedWidth ? constrainedWidth : (layout.width + horizontalPadding * 2);
			const selectionHeight = (textBottom - textTop) + verticalPadding * 2;
			const originX = boxX + selectionWidth / 2;
			const originY = boxY + selectionHeight / 2;

			renderCtx.save();
			renderCtx.translate(originX, originY);
			renderCtx.rotate(rotation);
			const textStartX = -selectionWidth / 2 + horizontalPadding;
			const textStartY = -selectionHeight / 2 + verticalPadding + layout.ascent;
			layout.lines.forEach((line, index) => {
				const baselineY = textStartY + index * layout.lineHeight;
				renderCtx.fillText(line || ' ', textStartX, baselineY);
			});
			renderCtx.restore();
			
			if (isSelected) {
				const outlineBounds = getSelectionOutlineBounds(boxX, boxY, selectionWidth, selectionHeight, $zoom, false);
				renderSelectionOutline(renderCtx, boxX, boxY, selectionWidth, selectionHeight, $zoom, false, rotation);
				if (showIndividualHandles) {
					renderTextHandles(renderCtx, boxX, boxY, selectionWidth, selectionHeight, $zoom, false, rotation);
					renderRotationHandleFromBounds(renderCtx, outlineBounds, $zoom, rotation);
				}
			}
		});

		const groupBoundingBoxRaw =
			totalSelectionCount > 1 
				? (isGroupResizing ? calculateGroupBoundingBox() : (groupResizeCurrentBox ?? calculateGroupBoundingBox()))
				: null;
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
		const dpr = window.devicePixelRatio || 1;
		canvas.width = window.innerWidth * dpr;
		canvas.height = window.innerHeight * dpr;
		ctx = canvas.getContext('2d');
		if (ctx) ctx.scale(dpr, dpr);
	}

	$: if (isTypingText) {
		const typingContentWidth = getTextContentWidthFromBoxWidth(typingBoxWidth);
		typingLayout = measureMultilineText(typingValue || '', typingFontSize, ctx ?? undefined, typingContentWidth);
	}

	$: if (isTypingText && typingWorldPos) {
		typingScreenPos = worldToScreen(typingWorldPos.x, typingWorldPos.y, $viewportOffset, $zoom);
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
		selectedDiamonds: $selectedDiamonds,
		texts: $texts,
		selectedTexts: $selectedTexts
	};

	$: if (ctx && canvas && !isCreatingShape && renderDependencies) {
		scheduleRender();
	}
</script>

<div class="relative w-full h-full bg-stone-50">
	<Toolbar />
	<ZoomControls />
	<UndoRedoControls />
	<StylePanel />
	{#if isTypingText && typingWorldPos}
		<textarea
			bind:this={textInputRef}
			class="absolute z-50 bg-transparent border-none outline-none p-0 m-0 resize-none caret-black whitespace-pre overflow-hidden appearance-none pointer-events-auto"
			style={`left:${typingScreenPos.x}px; top:${typingScreenPos.y - typingLayout.ascent * $zoom}px; font-size:${typingFontSize * $zoom}px; font-family:'Lucida Console', monospace; line-height:${typingLayout.lineHeight * $zoom}px; width:${Math.max(getTextContentWidthFromBoxWidth(typingBoxWidth) ?? typingLayout.width, 2) * $zoom}px; height:${typingLayout.height * $zoom}px; contain: layout style paint; color:${typingTextColor || '#000000'};`}
			spellcheck="false"
			autocomplete="off"
			autocapitalize="off"
			tabindex="-1"
			rows="1"
			bind:value={typingValue}
			on:input={handleTextInputInput}
			on:keydown={handleTextInputKeyDown}
			on:blur={commitTypingText}
		></textarea>
	{/if}
<canvas
	on:mousedown={handleMouseDown}
	on:mousemove={handleMouseMove}
	on:mouseup={handleMouseUp}
	on:dblclick={handleCanvasDoubleClick}
		on:wheel={(e) => handleViewportScroll(e, canvas!)}
	bind:this={canvas}
		class="w-full h-full bg-stone-50"
	tabindex="0"
></canvas>
</div>
