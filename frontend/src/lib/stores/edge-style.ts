import { writable } from 'svelte/store';

export type EdgeStyle = 'sharp' | 'rounded';

export const edgeStyle = writable<EdgeStyle>('sharp');

