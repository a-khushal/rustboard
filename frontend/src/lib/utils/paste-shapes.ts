import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, type Rectangle, type Ellipse, type Line, type Arrow } from '$lib/stores/editor';
import type { ClipboardData } from './clipboard';

function calculateBoundingBox(clipboard: ClipboardData, fallbackX: number, fallbackY: number): { minX: number; minY: number } {
    const allShapes: Array<{ minX: number; minY: number }> = [];

    clipboard.rectangles.forEach(r => {
        allShapes.push({ minX: r.position.x, minY: r.position.y });
    });

    clipboard.ellipses.forEach(e => {
        allShapes.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y });
    });

    clipboard.lines.forEach(l => {
        allShapes.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) });
    });

    clipboard.arrows.forEach(a => {
        allShapes.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) });
    });

    if (allShapes.length === 0) return { minX: fallbackX, minY: fallbackY };
    return {
        minX: Math.min(...allShapes.map(s => s.minX)),
        minY: Math.min(...allShapes.map(s => s.minY))
    };
}

export function pasteShapes(clipboard: ClipboardData, offsetX: number, offsetY: number): {
    rectangles: number[];
    ellipses: number[];
    lines: number[];
    arrows: number[];
} {
    const api = get(editorApi);
    if (!api) return { rectangles: [], ellipses: [], lines: [], arrows: [] };

    const { minX, minY } = calculateBoundingBox(clipboard, offsetX, offsetY);

    api.save_snapshot();

    const pastedIds = {
        rectangles: [] as number[],
        ellipses: [] as number[],
        lines: [] as number[],
        arrows: [] as number[]
    };

    clipboard.rectangles.forEach(rect => {
        const newX = rect.position.x - minX + offsetX;
        const newY = rect.position.y - minY + offsetY;
        const newId = (api as any).add_rectangle_without_snapshot(newX, newY, rect.width, rect.height);
        pastedIds.rectangles.push(Number(newId));
    });

    clipboard.ellipses.forEach(ellipse => {
        const newX = ellipse.position.x - minX + offsetX;
        const newY = ellipse.position.y - minY + offsetY;
        const newId = (api as any).add_ellipse_without_snapshot(newX, newY, ellipse.radius_x, ellipse.radius_y);
        pastedIds.ellipses.push(Number(newId));
    });

    clipboard.lines.forEach(line => {
        const startX = line.start.x - minX + offsetX;
        const startY = line.start.y - minY + offsetY;
        const endX = line.end.x - minX + offsetX;
        const endY = line.end.y - minY + offsetY;
        const newId = (api as any).add_line_without_snapshot(startX, startY, endX, endY);
        pastedIds.lines.push(Number(newId));
    });

    clipboard.arrows.forEach(arrow => {
        const startX = arrow.start.x - minX + offsetX;
        const startY = arrow.start.y - minY + offsetY;
        const endX = arrow.end.x - minX + offsetX;
        const endY = arrow.end.y - minY + offsetY;
        const newId = (api as any).add_arrow_without_snapshot(startX, startY, endX, endY);
        pastedIds.arrows.push(Number(newId));
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

    selectedRectangles.set(updatedRectangles.filter(r => pastedIds.rectangles.includes(r.id)));
    selectedEllipses.set(updatedEllipses.filter(e => pastedIds.ellipses.includes(e.id)));
    selectedLines.set(updatedLines.filter(l => pastedIds.lines.includes(l.id)));
    selectedArrows.set(updatedArrows.filter(a => pastedIds.arrows.includes(a.id)));

    return pastedIds;
}

