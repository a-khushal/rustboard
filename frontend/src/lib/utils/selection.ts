import { selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedTexts } from '$lib/stores/editor';

export function clearAllSelections(): void {
    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
    selectedTexts.set([]);
}
