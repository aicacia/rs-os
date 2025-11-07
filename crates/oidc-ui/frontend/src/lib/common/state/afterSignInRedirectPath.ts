import { goto } from '$app/navigation';
import { localStorageState } from '../util/localStorageState.svelte';

const afterSigninRedirectPath = localStorageState<string | null>(
	'after-signin-redirect-path',
	null
);

export function setAfterSigninRedirectPathFromURL(url: URL) {
	afterSigninRedirectPath.value = url.toString().substring(url.origin.length);
}

export async function afterSigninRedirect() {
	if (afterSigninRedirectPath.value) {
		// eslint-disable-next-line svelte/no-navigation-without-resolve
		await goto(afterSigninRedirectPath.value);
		afterSigninRedirectPath.value = null;
	}
}
