import { derived, get } from 'svelte/store';
import {
	rectangles,
	ellipses,
	lines,
	arrows,
	diamonds,
	selectedRectangles,
	selectedEllipses,
	selectedLines,
	selectedArrows,
	selectedDiamonds,
	editorApi,
	paths,
	images,
	selectedPaths,
	selectedImages
} from '$lib/stores/editor';

export type SelectionSnapshot = {
	rectangles: number[];
	ellipses: number[];
	diamonds: number[];
	lines: number[];
	arrows: number[];
	paths: number[];
	images: number[];
	historyIndex: number;
};

const MAX_HISTORY = 200;

let initialized = false;
let isSuppressed = false;
let selectionStack: SelectionSnapshot[] = [];
let selectionIndex = -1;
let unsubscribe: (() => void) | null = null;
let pendingSnapshot: SelectionSnapshot | null = null;
let flushScheduled = false;

function snapshotEquals(a: SelectionSnapshot, b: SelectionSnapshot): boolean {
	return (
		a.historyIndex === b.historyIndex &&
		arraysEqual(a.rectangles, b.rectangles) &&
		arraysEqual(a.ellipses, b.ellipses) &&
		arraysEqual(a.diamonds, b.diamonds) &&
		arraysEqual(a.lines, b.lines) &&
		arraysEqual(a.arrows, b.arrows) &&
		arraysEqual(a.paths, b.paths) &&
		arraysEqual(a.images, b.images)
	);
}

function arraysEqual(a: number[], b: number[]): boolean {
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i += 1) {
		if (a[i] !== b[i]) return false;
	}
	return true;
}

function currentHistoryIndex(): number {
	const api = get(editorApi);
	if (api && typeof api.history_index === 'function') {
		return Number(api.history_index());
	}
	return 0;
}

export function getCurrentSelectionSnapshot(): SelectionSnapshot {
	return {
		rectangles: get(selectedRectangles).map((rect) => rect.id),
		ellipses: get(selectedEllipses).map((ellipse) => ellipse.id),
		diamonds: get(selectedDiamonds).map((diamond) => diamond.id),
		lines: get(selectedLines).map((line) => line.id),
		arrows: get(selectedArrows).map((arrow) => arrow.id),
		paths: get(selectedPaths).map((path) => path.id),
		images: get(selectedImages).map((image) => image.id),
		historyIndex: currentHistoryIndex()
	};
}

function pushSnapshot(snapshot: SelectionSnapshot, force = false): void {
	if (!force && selectionIndex >= 0 && snapshotEquals(selectionStack[selectionIndex], snapshot)) {
		return;
	}

	if (selectionIndex < selectionStack.length - 1) {
		selectionStack = selectionStack.slice(0, selectionIndex + 1);
	}

	selectionStack.push(snapshot);
	if (selectionStack.length > MAX_HISTORY) {
		selectionStack.shift();
	}
	selectionIndex = selectionStack.length - 1;
}

export function initSelectionHistory(): void {
	if (initialized) return;
	initialized = true;

	const selectionSnapshotStore = derived(
		[selectedRectangles, selectedEllipses, selectedDiamonds, selectedLines, selectedArrows, selectedPaths, selectedImages],
		([$rects, $ells, $dias, $lines, $arrows, $paths, $images]) => ({
			rectangles: $rects.map((rect) => rect.id),
			ellipses: $ells.map((ellipse) => ellipse.id),
			diamonds: $dias.map((diamond) => diamond.id),
			lines: $lines.map((line) => line.id),
			arrows: $arrows.map((arrow) => arrow.id),
			paths: $paths.map((path) => path.id),
			images: $images.map((image) => image.id),
			historyIndex: currentHistoryIndex()
		})
	);

	unsubscribe = selectionSnapshotStore.subscribe((snapshot) => {
		if (isSuppressed) return;
		queueSnapshot(snapshot);
	});

	resetSelectionHistory();
}

export function resetSelectionHistory(): void {
	selectionStack = [];
	selectionIndex = -1;
	pendingSnapshot = null;
	flushScheduled = false;
	const snapshot = getCurrentSelectionSnapshot();
	pushSnapshot(snapshot, true);
}

export function restoreSelectionForHistoryIndex(historyIndex: number): void {
	if (selectionStack.length === 0) {
		resetSelectionHistory();
	}
	if (selectionStack.length === 0) return;

	let targetIndex = selectionIndex;
	for (let i = selectionStack.length - 1; i >= 0; i -= 1) {
		if (selectionStack[i].historyIndex <= historyIndex) {
			targetIndex = i;
			break;
		}
	}

	if (targetIndex < 0) {
		targetIndex = 0;
	}

	selectionIndex = targetIndex;
	const snapshot = selectionStack[targetIndex] ?? selectionStack[selectionStack.length - 1];
	if (!snapshot) return;

	isSuppressed = true;
	const availableRects = get(rectangles);
	const availableEllipses = get(ellipses);
	const availableDiamonds = get(diamonds);
	const availableLines = get(lines);
	const availableArrows = get(arrows);
	const availablePaths = get(paths);
	const availableImages = get(images);

	selectedRectangles.set(filterByIds(availableRects, snapshot.rectangles));
	selectedEllipses.set(filterByIds(availableEllipses, snapshot.ellipses));
	selectedDiamonds.set(filterByIds(availableDiamonds, snapshot.diamonds));
	selectedLines.set(filterByIds(availableLines, snapshot.lines));
	selectedArrows.set(filterByIds(availableArrows, snapshot.arrows));
	selectedPaths.set(filterByIds(availablePaths, snapshot.paths));
	selectedImages.set(filterByIds(availableImages, snapshot.images));
	isSuppressed = false;
}

function filterByIds<T extends { id: number }>(items: T[], ids: number[]): T[] {
	return ids
		.map((id) => items.find((item) => item.id === id))
		.filter((item): item is T => Boolean(item));
}

export function disposeSelectionHistory(): void {
	if (unsubscribe) {
		unsubscribe();
		unsubscribe = null;
	}
	initialized = false;
	selectionStack = [];
	selectionIndex = -1;
	pendingSnapshot = null;
	flushScheduled = false;
	isSuppressed = false;
}

function queueSnapshot(snapshot: SelectionSnapshot): void {
	pendingSnapshot = snapshot;
	if (flushScheduled) return;
	flushScheduled = true;
	queueMicrotask(() => {
		flushScheduled = false;
		if (!pendingSnapshot) return;
		const snap = pendingSnapshot;
		pendingSnapshot = null;
		if (!isSuppressed) {
			pushSnapshot(snap);
		}
	});
}

