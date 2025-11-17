import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, diamonds, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond } from '$lib/stores/editor';

export function updateAllStoresAfterUndoRedo(): void {
    const api = get(editorApi);
    if (!api) return;

    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    const updatedLines = Array.from(api.get_lines() as Line[]);
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);
    const updatedDiamonds = Array.from(api.get_diamonds() as Diamond[]);

    rectangles.set(updatedRectangles);
    ellipses.set(updatedEllipses);
    lines.set(updatedLines);
    arrows.set(updatedArrows);
    diamonds.set(updatedDiamonds);

    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
}

