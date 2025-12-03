import { selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedTexts, selectedPaths } from '$lib/stores/editor';

export function clearAllSelections(): void {
    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
    selectedTexts.set([]);
    selectedPaths.set([]);
}
