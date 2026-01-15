import { get } from 'svelte/store';
import { editorApi, arrows, selectedArrows, type Arrow } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { defaultStrokeColor } from '$lib/stores/stroke-color';
import { dashPattern } from '$lib/stores/dash-pattern';

export function addArrow(startX: number, startY: number, endX: number, endY: number): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const strokeWidth = getStore(defaultStrokeWidth);
    const strokeColor = getStore(defaultStrokeColor);
    const dashPatternValue = getStore(dashPattern);

    const newId = api.add_arrow(startX, startY, endX, endY);
    api.set_arrow_line_width(BigInt(newId), strokeWidth, false);
    api.set_arrow_stroke_color(BigInt(newId), strokeColor, false);
    if (dashPatternValue !== 'solid') {
        api.set_arrow_dash_pattern(BigInt(newId), dashPatternValue, false);
    }
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

export function setArrowRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_arrow_rotation(BigInt(id), angle, saveHistory);
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

