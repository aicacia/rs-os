import { getUser } from '$lib/common/state/user.svelte';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { resolve } from '$app/paths';
import { setAfterSigninRedirectPathFromURL } from '$lib/common/state/afterSignInRedirectPath';

export const load: LayoutLoad = async (event) => {
	await event.parent();

	const user = await getUser();

	if (user) {
		return {
			user
		};
	} else {
		setAfterSigninRedirectPathFromURL(event.url);
		redirect(302, resolve('/signin'));
	}
};
