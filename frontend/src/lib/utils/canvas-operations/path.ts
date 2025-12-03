import { get } from 'svelte/store';
import { editorApi, paths, selectedPaths, type Path } from '$lib/stores/editor';

export function addPath(points: Array<{ x: number; y: number }>): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_path(points);
    const updatedPaths = Array.from(api.get_paths() as Path[]);
    paths.set(updatedPaths);

    const newPath = updatedPaths.find((p: Path) => p.id === newId);
    if (newPath) {
        selectedPaths.set([newPath]);
    }
    return Number(newId);
}

export function movePath(id: number, deltaX: number, deltaY: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_path(BigInt(id), deltaX, deltaY, saveHistory);
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

