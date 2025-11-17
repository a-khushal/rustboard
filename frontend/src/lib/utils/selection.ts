import { selectedRectangles, selectedEllipses, selectedLines, selectedArrows } from '$lib/stores/editor';

export function clearAllSelections(): void {
    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
}
