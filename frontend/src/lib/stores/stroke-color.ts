import { writable, get } from 'svelte/store';
import { theme } from './theme';

function getDefaultStrokeColor(): string {
	const currentTheme = get(theme);
	return currentTheme === 'dark' ? '#ffffff' : '#000000';
}

export const defaultStrokeColor = writable<string>(getDefaultStrokeColor());

theme.subscribe((currentTheme) => {
	const currentColor = get(defaultStrokeColor);
	const themeDefault = currentTheme === 'dark' ? '#ffffff' : '#000000';
	const oppositeThemeDefault = themeDefault === '#ffffff' ? '#000000' : '#ffffff';
	
	if (currentColor === oppositeThemeDefault) {
		defaultStrokeColor.set(themeDefault);
	}
});

