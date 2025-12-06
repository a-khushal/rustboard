import { selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedTexts, selectedPaths, selectedImages, selectedGroups } from '$lib/stores/editor';

export function clearAllSelections(): void {
    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
    selectedTexts.set([]);
    selectedPaths.set([]);
    selectedImages.set([]);
    selectedGroups.set([]);
}
