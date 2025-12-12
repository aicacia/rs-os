import { getCurrentUser } from '$lib/common/state/currentUser.svelte';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { resolve } from '$app/paths';
import { setAfterSigninRedirectPathFromURL } from '$lib/common/state/afterSignInRedirectPath';

export const load: LayoutLoad = async (event) => {
	await event.parent();

	const currentUser = await getCurrentUser();

	if (currentUser) {
		return {
			user: currentUser
		};
	} else {
		setAfterSigninRedirectPathFromURL(event.url);
		redirect(302, resolve('/signin'));
	}
};
