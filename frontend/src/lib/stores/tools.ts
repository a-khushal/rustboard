import { writable } from "svelte/store";

export type Tool = 'select' | 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond';
export const activeTool = writable<Tool>('select' as Tool);