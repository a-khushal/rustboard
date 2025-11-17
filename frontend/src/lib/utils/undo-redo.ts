import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, diamonds, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond } from '$lib/stores/editor';
import { restoreSelectionForHistoryIndex } from './selection-history';

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

    const historyIndex = typeof api.history_index === 'function' ? Number(api.history_index()) : 0;
    restoreSelectionForHistoryIndex(historyIndex);
}

