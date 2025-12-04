import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, diamonds, texts, paths, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedTexts, selectedPaths, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Text, type Path } from '$lib/stores/editor';
import { updatePaths } from '$lib/utils/canvas-operations/path';

export function deleteShapes(
    rectangleIds: number[],
    ellipseIds: number[],
    lineIds: number[],
    arrowIds: number[],
    diamondIds: number[],
    textIds: number[],
    pathIds: number[] = []
): void {
    const api = get(editorApi);
    if (!api) return;

    const hasAnySelection = rectangleIds.length > 0 || ellipseIds.length > 0 || lineIds.length > 0 || arrowIds.length > 0 || diamondIds.length > 0 || textIds.length > 0 || pathIds.length > 0;
    if (!hasAnySelection) return;

    api.save_snapshot();

    rectangleIds.forEach(id => {
        api.delete_rectangle_without_snapshot(BigInt(id));
    });

    ellipseIds.forEach(id => {
        api.delete_ellipse_without_snapshot(BigInt(id));
    });

    lineIds.forEach(id => {
        api.delete_line_without_snapshot(BigInt(id));
    });

    arrowIds.forEach(id => {
        api.delete_arrow_without_snapshot(BigInt(id));
    });

    diamondIds.forEach(id => {
        api.delete_diamond_without_snapshot(BigInt(id));
    });

    textIds.forEach(id => {
        api.delete_text_without_snapshot(BigInt(id));
    });

    pathIds.forEach(id => {
        api.delete_path_without_snapshot(BigInt(id));
    });

    api.save_snapshot();

    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    const updatedLines = Array.from(api.get_lines() as Line[]);
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);
    const updatedDiamonds = Array.from(api.get_diamonds() as Diamond[]);
    const updatedTexts = Array.from(api.get_texts() as Text[]);
    rectangles.set(updatedRectangles);
    ellipses.set(updatedEllipses);
    lines.set(updatedLines);
    arrows.set(updatedArrows);
    diamonds.set(updatedDiamonds);
    texts.set(updatedTexts);

    updatePaths();

    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
    selectedTexts.set([]);
    selectedPaths.set([]);
}
