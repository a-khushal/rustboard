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

export function setTextBoxWidth(id: number, boxWidth: number | null, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_box_width(BigInt(id), boxWidth, saveHistory);
    updateTexts();
}

export function setTextRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_rotation(BigInt(id), angle, saveHistory);
    updateTexts();
}

export function updateTextContent(id: number, value: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    const currentTexts = get(texts);
    const currentText = currentTexts.find((t: Text) => t.id === id);
    const originalColor = currentText?.text_color;

    api.update_text(BigInt(id), value, saveHistory);
    updateTexts();

    if (originalColor && originalColor !== '#000000') {
        const updatedTexts = get(texts);
        const updatedText = updatedTexts.find((t: Text) => t.id === id);
        if (updatedText && (!updatedText.text_color || updatedText.text_color === '#000000')) {
            api.set_text_color(BigInt(id), originalColor, false);
            updateTexts();
        }
    }
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
