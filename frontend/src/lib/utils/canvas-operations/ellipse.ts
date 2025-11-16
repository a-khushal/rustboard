import { get } from 'svelte/store';
import { editorApi, ellipses, selectedEllipses, type Ellipse } from '$lib/stores/editor';

export function addEllipse(x: number, y: number, radius_x: number = 50, radius_y: number = 50): void {
    const api = get(editorApi);
    if (!api) return;

    const newId = api.add_ellipse(x, y, radius_x, radius_y);
    const updatedEllipses = api.get_ellipses() as Ellipse[];
    ellipses.set(updatedEllipses);

    const newEllipse = updatedEllipses.find((e: Ellipse) => e.id === newId);
    if (newEllipse) {
        selectedEllipses.set([newEllipse]);
    }
}

export function deleteEllipses(ids: number[]): void {
    const api = get(editorApi);
    if (!api) return;

    ids.forEach(id => {
        api.delete_ellipse(BigInt(id));
    });
    updateEllipses();
}

export function moveEllipse(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_ellipse(BigInt(id), x, y, saveHistory);
    updateEllipses();
}

export function resizeEllipse(id: number, radius_x: number, radius_y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_ellipse(BigInt(id), radius_x, radius_y, saveHistory);
    updateEllipses();
}

export function updateEllipses(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedEllipses = api.get_ellipses() as Ellipse[];
    ellipses.set(updatedEllipses);

    const currentSelection = get(selectedEllipses);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((e: Ellipse) => e.id));
        const updatedSelection = updatedEllipses.filter((e: Ellipse) => selectedIds.has(e.id));
        selectedEllipses.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}
