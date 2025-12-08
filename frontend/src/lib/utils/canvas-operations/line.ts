import { get } from 'svelte/store';
import { editorApi, lines, selectedLines, type Line } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { dashPattern } from '$lib/stores/dash-pattern';

export function addLine(startX: number, startY: number, endX: number, endY: number): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const strokeWidth = getStore(defaultStrokeWidth);
    const dashPatternValue = getStore(dashPattern);

    const newId = api.add_line(startX, startY, endX, endY);
    api.set_line_line_width(BigInt(newId), strokeWidth, false);
    if (dashPatternValue !== 'solid') {
        api.set_line_dash_pattern(BigInt(newId), dashPatternValue, false);
    }
    const updatedLines = Array.from(api.get_lines() as Line[]);
    lines.set(updatedLines);

    const newLine = updatedLines.find((l: Line) => l.id === newId);
    if (newLine) {
        selectedLines.set([newLine]);
    }
    return Number(newId);
}

export function moveLine(id: number, startX: number, startY: number, endX: number, endY: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_line(BigInt(id), startX, startY, endX, endY, saveHistory);
    updateLines();
}

export function setLineRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_line_rotation(BigInt(id), angle, saveHistory);
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

