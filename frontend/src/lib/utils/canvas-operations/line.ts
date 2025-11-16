import { get } from 'svelte/store';
import { editorApi, lines, selectedLines, type Line } from '$lib/stores/editor';

export function addLine(startX: number, startY: number, endX: number, endY: number): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_line(startX, startY, endX, endY);
    const updatedLines = Array.from(api.get_lines() as Line[]);
    lines.set(updatedLines);

    const newLine = updatedLines.find((l: Line) => l.id === newId);
    if (newLine) {
        selectedLines.set([newLine]);
    }
    return Number(newId);
}

export function deleteLines(ids: number[]): void {
    const api = get(editorApi);
    if (!api) return;

    ids.forEach(id => {
        api.delete_line(BigInt(id));
    });
    updateLines();
}

export function moveLine(id: number, startX: number, startY: number, endX: number, endY: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_line(BigInt(id), startX, startY, endX, endY, saveHistory);
    updateLines();
}

export function updateLines(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedLines = Array.from(api.get_lines() as Line[]);
    lines.set(updatedLines);

    const currentSelection = get(selectedLines);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((l: Line) => l.id));
        const updatedSelection = updatedLines.filter((l: Line) => selectedIds.has(l.id));
        selectedLines.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}

