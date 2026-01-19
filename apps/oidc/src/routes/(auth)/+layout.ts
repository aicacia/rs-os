import { getCurrentUserInfo, requiresPasswordReset } from '$lib/common/state/auth.svelte';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { resolve } from '$app/paths';
import { setAfterSigninRedirectPathFromURL } from '$lib/common/state/afterSignInRedirectPath';

export const load: LayoutLoad = async (event) => {
	await event.parent();

	const currentUserInfo = await getCurrentUserInfo();

	if (currentUserInfo) {
		if (requiresPasswordReset()) {
			redirect(302, resolve('/password-reset'));
		}
		return {
			userInfo: currentUserInfo
		};
	} else {
		setAfterSigninRedirectPathFromURL(event.url);
		redirect(302, resolve('/signin'));
	}
};
