import { writable } from "svelte/store";

export type Tool = 'select' | 'rectangle' | 'ellipse' | 'line' | 'arrow';
export const activeTool = writable<Tool>('select' as Tool);