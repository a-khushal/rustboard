import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, type Rectangle, type Ellipse, type Line, type Arrow } from '$lib/stores/editor';

const STORAGE_KEY = 'rustboard-state';

export function saveStateToLocalStorage(): void {
    if (typeof window === 'undefined') return;

    const api = get(editorApi);
    if (!api) return;

    try {
        const serialized = api.serialize();
        localStorage.setItem(STORAGE_KEY, serialized);
    } catch (error) {
        console.error('Failed to save state to localStorage:', error);
    }
}

export function loadStateFromLocalStorage(): boolean {
    if (typeof window === 'undefined') return false;

    const api = get(editorApi);
    if (!api) return false;

    try {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (!saved) return false;

        const success = api.deserialize(saved);
        if (success) {
            const updatedRectangles = api.get_rectangles() as Rectangle[];
            const updatedEllipses = api.get_ellipses() as Ellipse[];
            const updatedLines = api.get_lines() as Line[];
            const updatedArrows = (api as any).get_arrows() as Arrow[];
            rectangles.set(updatedRectangles);
            ellipses.set(updatedEllipses);
            lines.set(updatedLines);
            arrows.set(updatedArrows);
            return true;
        }
        return false;
    } catch (error) {
        console.error('Failed to load state from localStorage:', error);
        return false;
    }
}

