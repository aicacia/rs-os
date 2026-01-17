import { getUserManager } from '$lib/common/state/user.svelte';
import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { notifications } from '$lib/common/state/notifications.svelte';
import { resolve } from '$app/paths';
import { setAuthToken } from '$lib/common/openapi';
import { getAndClearAfterSigninRedirectPath } from '$lib/common/state/afterSignInRedirectPath';

export const load: PageLoad = async (event) => {
	await event.parent();

	try {
		const userManager = await getUserManager();
		const user = await userManager.signinCallback(event.url.toString());
		if (user) {
			setAuthToken(user.access_token);
		}
	} catch (e) {
		if (e instanceof Error) {
			notifications.add(e.message);
		}
		redirect(302, resolve('/signin'));
	}
	redirect(302, getAndClearAfterSigninRedirectPath());
};
