import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { localStorageState } from '../util/localStorageState.svelte';

const afterSigninRedirectPath = localStorageState<string | null>(
	'after-signin-redirect-path',
	null
);

export function setAfterSigninRedirectPathFromURL(url: URL) {
	afterSigninRedirectPath.value = url.toString().substring(url.origin.length);
}

export async function afterSigninRedirect() {
	const path = afterSigninRedirectPath.value;
	if (path) {
		afterSigninRedirectPath.value = null;
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		await goto(path);
	} else {
		await goto(resolve('/'));
	}
}
