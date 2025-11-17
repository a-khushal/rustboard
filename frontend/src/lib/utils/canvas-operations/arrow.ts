import { get } from 'svelte/store';
import { editorApi, arrows, selectedArrows, type Arrow } from '$lib/stores/editor';

export function addArrow(startX: number, startY: number, endX: number, endY: number): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_arrow(startX, startY, endX, endY);
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);
    arrows.set(updatedArrows);

    const newArrow = updatedArrows.find((a: Arrow) => a.id === newId);
    if (newArrow) {
        selectedArrows.set([newArrow]);
    }
    return Number(newId);
}

export function moveArrow(id: number, startX: number, startY: number, endX: number, endY: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_arrow(BigInt(id), startX, startY, endX, endY, saveHistory);
    updateArrows();
}

export function updateArrows(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedArrows = Array.from(api.get_arrows() as Arrow[]);
    arrows.set(updatedArrows);

    const currentSelection = get(selectedArrows);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((a: Arrow) => a.id));
        const updatedSelection = updatedArrows.filter((a: Arrow) => selectedIds.has(a.id));
        selectedArrows.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}

