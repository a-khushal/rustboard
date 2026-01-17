import { get } from 'svelte/store';
import { editorApi, paths, selectedPaths, type Path } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { defaultStrokeColor } from '$lib/stores/stroke-color';
import { sendOperation } from '$lib/utils/collaboration';

export function addPath(points: Array<{ x: number; y: number }>): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const strokeWidth = getStore(defaultStrokeWidth);
    const strokeColor = getStore(defaultStrokeColor);

    const newId = api.add_path(points);
    api.set_path_line_width(BigInt(newId), strokeWidth, false);
    api.set_path_stroke_color(BigInt(newId), strokeColor, false);
    const updatedPaths = Array.from(api.get_paths() as Path[]);
    paths.set(updatedPaths);

    const newPath = updatedPaths.find((p: Path) => p.id === newId);
    if (newPath) {
        selectedPaths.set([newPath]);
    }
    
    sendOperation({
        op: 'AddPath',
        id: Number(newId),
        points
    });
    
    return Number(newId);
}

export function movePath(id: number, deltaX: number, deltaY: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_path(BigInt(id), deltaX, deltaY, saveHistory);
    updatePaths();
    
    sendOperation({
        op: 'MovePath',
        id,
        offset_x: deltaX,
        offset_y: deltaY
    });
}

export function resizePath(id: number, x: number, y: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_path(BigInt(id), x, y, width, height, saveHistory);
    updatePaths();
}

export function setPathRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_path_rotation(BigInt(id), angle, saveHistory);
    updatePaths();
}

export function setPathPoints(id: number, points: Array<{ x: number; y: number }>, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_path_points(BigInt(id), points, saveHistory);
    updatePaths();
}

export function updatePaths(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedPaths = Array.from(api.get_paths() as Path[]);
    paths.set(updatedPaths);

    const currentSelection = get(selectedPaths);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((p: Path) => p.id));
        const updatedSelection = updatedPaths.filter((p: Path) => selectedIds.has(p.id));
        selectedPaths.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}

