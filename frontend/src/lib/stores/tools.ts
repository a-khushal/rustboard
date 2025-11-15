import { writable } from "svelte/store";

export type Tool = 'select' | 'rectangle' | 'elipse';
export const activeTool = writable<Tool>('select' as Tool);