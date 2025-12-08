import { get } from 'svelte/store';
import { editorApi, diamonds, selectedDiamonds, type Diamond } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { edgeStyle } from '$lib/stores/edge-style';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { dashPattern } from '$lib/stores/dash-pattern';

export function addDiamond(x: number, y: number, width: number = 100, height: number = 50): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const currentEdgeStyle = getStore(edgeStyle);
    const radius = currentEdgeStyle === 'rounded' ? 4.0 : 0.0;
    const strokeWidth = getStore(defaultStrokeWidth);
    const dashPatternValue = getStore(dashPattern);

    const newId = api.add_diamond(x, y, width, height);
    api.set_diamond_border_radius(BigInt(newId), radius, false);
    api.set_diamond_line_width(BigInt(newId), strokeWidth, false);
    if (dashPatternValue !== 'solid') {
        api.set_diamond_dash_pattern(BigInt(newId), dashPatternValue, false);
    }
    
    const updatedDiamonds = Array.from(api.get_diamonds() as Diamond[]);
    diamonds.set(updatedDiamonds);

    const newDiamond = updatedDiamonds.find((d: Diamond) => d.id === newId);
    if (newDiamond) {
        selectedDiamonds.set([newDiamond]);
    }
    return Number(newId);
}

export function moveDiamond(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_diamond(BigInt(id), x, y, saveHistory);
    updateDiamonds();
}

export function resizeDiamond(id: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_diamond(BigInt(id), width, height, saveHistory);
    updateDiamonds();
}

export function setDiamondRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_diamond_rotation(BigInt(id), angle, saveHistory);
    updateDiamonds();
}

export function updateDiamonds(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedDiamonds = Array.from(api.get_diamonds() as Diamond[]);
    diamonds.set(updatedDiamonds);

    const currentSelection = get(selectedDiamonds);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((d: Diamond) => d.id));
        const updatedSelection = updatedDiamonds.filter((d: Diamond) => selectedIds.has(d.id));
        selectedDiamonds.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}

