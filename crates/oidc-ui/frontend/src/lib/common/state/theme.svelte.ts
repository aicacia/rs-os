import { browser } from '$app/environment';

export type ThemeType = 'dark' | 'light';

let theme = $state<ThemeType>(
	browser && window.matchMedia('(prefers-color-scheme: dark)')?.matches ? 'dark' : 'light'
);

export function setTheme(newTheme: ThemeType) {
	theme = newTheme;
}

export function getTheme() {
	return theme;
}
