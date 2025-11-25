import { get } from 'svelte/store';
import { editorApi, rectangles, selectedRectangles, type Rectangle } from '$lib/stores/editor';

export function addRectangle(x: number, y: number, width: number = 100, height: number = 50): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_rectangle(x, y, width, height);
    const updatedRectangles = Array.from(api.get_rectangles() as Rectangle[]);
    rectangles.set(updatedRectangles);

    const newRect = updatedRectangles.find((r: Rectangle) => r.id === newId);
    if (newRect) {
        selectedRectangles.set([newRect]);
    }
    return Number(newId);
}

export function moveRectangle(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_rectangle(BigInt(id), x, y, saveHistory);
    updateRectangles();
}

export function resizeRectangle(id: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_rectangle(BigInt(id), width, height, saveHistory);
    updateRectangles();
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
