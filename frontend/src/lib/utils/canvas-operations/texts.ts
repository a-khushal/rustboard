import { get } from 'svelte/store';
import { editorApi, texts, selectedTexts, type Text } from '$lib/stores/editor';

export function addText(x: number, y: number, text: string, saveHistory: boolean = true): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = saveHistory
        ? api.add_text(x, y, text)
        : api.add_text_without_snapshot(x, y, text);
    const updatedTexts = Array.from(api.get_texts() as Text[]);
    texts.set(updatedTexts);

    const newText = updatedTexts.find((t: Text) => t.id === newId);
    if (newText) {
        selectedTexts.set([newText]);
    }
    return Number(newId);
}

export function moveText(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_text(BigInt(id), x, y, saveHistory);
    updateTexts();
}

export function setTextFontSize(id: number, fontSize: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_text(BigInt(id), fontSize, saveHistory);
    updateTexts();
}

export function updateTextContent(id: number, value: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.update_text(BigInt(id), value, saveHistory);
    updateTexts();
}

export function deleteTextById(id: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    if (saveHistory) {
        api.delete_text(BigInt(id));
    } else {
        api.delete_text_without_snapshot(BigInt(id));
    }

    updateTexts();
}

export function updateTexts(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedTexts = Array.from(api.get_texts() as Text[]);
    texts.set(updatedTexts);

    const currentSelection = get(selectedTexts);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((t: Text) => t.id));
        const updatedSelection = updatedTexts.filter((t: Text) => selectedIds.has(t.id));
        selectedTexts.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}
