import { get } from 'svelte/store';
import { editorApi, rectangles, selectedRectangles, type Rectangle } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { edgeStyle } from '$lib/stores/edge-style';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { defaultStrokeColor } from '$lib/stores/stroke-color';
import { dashPattern } from '$lib/stores/dash-pattern';
import { sendOperation } from '$lib/utils/collaboration';

export function addRectangle(x: number, y: number, width: number = 100, height: number = 50): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const currentEdgeStyle = getStore(edgeStyle);
    const radius = currentEdgeStyle === 'rounded' ? 4.0 : 0.0;
    const strokeWidth = getStore(defaultStrokeWidth);
    const strokeColor = getStore(defaultStrokeColor);
    const dashPatternValue = getStore(dashPattern);

    const newId = api.add_rectangle(x, y, width, height);
    api.set_rectangle_border_radius(BigInt(newId), radius, false);
    api.set_rectangle_line_width(BigInt(newId), strokeWidth, false);
    api.set_rectangle_stroke_color(BigInt(newId), strokeColor, false);
    if (dashPatternValue !== 'solid') {
        api.set_rectangle_dash_pattern(BigInt(newId), dashPatternValue, false);
    }
    
    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    rectangles.set(updatedRectangles);

    const newRect = updatedRectangles.find((r: Rectangle) => r.id === Number(newId));
    if (newRect) {
        selectedRectangles.set([newRect]);
    }

    sendOperation({
        op: 'AddRectangle',
        id: Number(newId),
        position: { x, y },
        width,
        height
    });
    
    return Number(newId);
}

export function moveRectangle(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_rectangle(BigInt(id), x, y, saveHistory);
    updateRectangles();

    sendOperation({
        op: 'MoveRectangle',
        id,
        position: { x, y }
    });
}

export function resizeRectangle(id: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_rectangle(BigInt(id), width, height, saveHistory);
    updateRectangles();

    sendOperation({
        op: 'ResizeRectangle',
        id,
        width,
        height
    });
}

export function setRectangleRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_rectangle_rotation(BigInt(id), angle, saveHistory);
    updateRectangles();
}

export function updateRectangles(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    rectangles.set(updatedRectangles);

    const currentSelection = get(selectedRectangles);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((r: Rectangle) => r.id));
        const updatedSelection = updatedRectangles.filter((r: Rectangle) => selectedIds.has(r.id));
        selectedRectangles.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}
