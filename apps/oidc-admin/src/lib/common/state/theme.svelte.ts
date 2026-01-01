import { browser } from '$app/environment';
import { localStorageState } from '../util/localStorageState.svelte';

export type ThemeType = 'dark' | 'light';

const theme = localStorageState<ThemeType>(
	'theme',
	browser && window.matchMedia('(prefers-color-scheme: dark)')?.matches ? 'dark' : 'light'
);

export function setTheme(newTheme: ThemeType) {
	theme.value = newTheme;
}

export function getTheme() {
	return theme.value;
}
