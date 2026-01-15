import { get } from 'svelte/store';
import { editorApi, ellipses, selectedEllipses, type Ellipse } from '$lib/stores/editor';
import { get as getStore } from 'svelte/store';
import { defaultStrokeWidth } from '$lib/stores/stroke-width';
import { defaultStrokeColor } from '$lib/stores/stroke-color';
import { dashPattern } from '$lib/stores/dash-pattern';

export function addEllipse(x: number, y: number, radius_x: number = 50, radius_y: number = 50): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const strokeWidth = getStore(defaultStrokeWidth);
    const strokeColor = getStore(defaultStrokeColor);
    const dashPatternValue = getStore(dashPattern);

    const newId = api.add_ellipse(x, y, radius_x, radius_y);
    api.set_ellipse_line_width(BigInt(newId), strokeWidth, false);
    api.set_ellipse_stroke_color(BigInt(newId), strokeColor, false);
    if (dashPatternValue !== 'solid') {
        api.set_ellipse_dash_pattern(BigInt(newId), dashPatternValue, false);
    }
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    ellipses.set(updatedEllipses);

    const newEllipse = updatedEllipses.find((e: Ellipse) => e.id === newId);
    if (newEllipse) {
        selectedEllipses.set([newEllipse]);
    }
    return Number(newId);
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

export function setEllipseRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_ellipse_rotation(BigInt(id), angle, saveHistory);
    updateEllipses();
}

export function updateEllipses(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedEllipses = Array.from(api.get_ellipses() as Ellipse[]);
    ellipses.set(updatedEllipses);

    const currentSelection = get(selectedEllipses);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((e: Ellipse) => e.id));
        const updatedSelection = updatedEllipses.filter((e: Ellipse) => selectedIds.has(e.id));
        selectedEllipses.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}
