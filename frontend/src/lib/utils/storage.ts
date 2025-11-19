import { get } from 'svelte/store';
import { editorApi, rectangles, ellipses, lines, arrows, diamonds, texts, zoom, viewportOffset, type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Text } from '$lib/stores/editor';

const STORAGE_KEY = 'rustboard-state';
const ZOOM_STORAGE_KEY = 'rustboard-zoom';
const VIEWPORT_OFFSET_STORAGE_KEY = 'rustboard-viewport-offset';

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
            const updatedArrows = api.get_arrows() as Arrow[];
            const updatedDiamonds = api.get_diamonds() as Diamond[];
            const updatedTexts = api.get_texts() as Text[];
            rectangles.set(updatedRectangles);
            ellipses.set(updatedEllipses);
            lines.set(updatedLines);
            arrows.set(updatedArrows);
            diamonds.set(updatedDiamonds);
            texts.set(updatedTexts);
            return true;
        }
        return false;
    } catch (error) {
        console.error('Failed to load state from localStorage:', error);
        return false;
    }
}

export function saveZoomToLocalStorage(): void {
    if (typeof window === 'undefined') return;

    try {
        const currentZoom = get(zoom);
        localStorage.setItem(ZOOM_STORAGE_KEY, currentZoom.toString());
    } catch (error) {
        console.error('Failed to save zoom to localStorage:', error);
    }
}

export function loadZoomFromLocalStorage(): boolean {
    if (typeof window === 'undefined') return false;

    try {
        const savedZoom = localStorage.getItem(ZOOM_STORAGE_KEY);
        if (!savedZoom) return false;

        const zoomValue = parseFloat(savedZoom);
        if (!isNaN(zoomValue) && zoomValue > 0) {
            zoom.set(zoomValue);
            return true;
        }
        return false;
    } catch (error) {
        console.error('Failed to load zoom from localStorage:', error);
        return false;
    }
}

export function saveViewportOffsetToLocalStorage(): void {
    if (typeof window === 'undefined') return;

    try {
        const currentOffset = get(viewportOffset);
        localStorage.setItem(VIEWPORT_OFFSET_STORAGE_KEY, JSON.stringify(currentOffset));
    } catch (error) {
        console.error('Failed to save viewport offset to localStorage:', error);
    }
}

export function loadViewportOffsetFromLocalStorage(): boolean {
    if (typeof window === 'undefined') return false;

    try {
        const savedOffset = localStorage.getItem(VIEWPORT_OFFSET_STORAGE_KEY);
        if (!savedOffset) return false;

        const offset = JSON.parse(savedOffset);
        if (offset && typeof offset.x === 'number' && typeof offset.y === 'number') {
            viewportOffset.set(offset);
            return true;
        }
        return false;
    } catch (error) {
        console.error('Failed to load viewport offset from localStorage:', error);
        return false;
    }
}

