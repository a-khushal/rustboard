import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, type Rectangle, type Ellipse, type Line, type Arrow } from '$lib/stores/editor';

export function deleteShapes(
    rectangleIds: number[],
    ellipseIds: number[],
    lineIds: number[],
    arrowIds: number[]
): void {
    const api = get(editorApi);
    if (!api) return;

    const hasAnySelection = rectangleIds.length > 0 || ellipseIds.length > 0 || lineIds.length > 0 || arrowIds.length > 0;
    if (!hasAnySelection) return;

    api.save_snapshot();

    rectangleIds.forEach(id => {
        (api as any).delete_rectangle_without_snapshot(BigInt(id));
    });

    ellipseIds.forEach(id => {
        (api as any).delete_ellipse_without_snapshot(BigInt(id));
    });

    lineIds.forEach(id => {
        (api as any).delete_line_without_snapshot(BigInt(id));
    });

    arrowIds.forEach(id => {
        (api as any).delete_arrow_without_snapshot(BigInt(id));
    });

    api.save_snapshot();

    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    const updatedLines = Array.from(api.get_lines() as Line[]);
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);

    rectangles.set(updatedRectangles);
    ellipses.set(updatedEllipses);
    lines.set(updatedLines);
    arrows.set(updatedArrows);

    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
}

