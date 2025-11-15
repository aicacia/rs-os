import { getUserManager } from '$lib/common/state/user.svelte';
import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { createNotification } from '$lib/common/state/notifications.svelte';

export const ssr = false;

export const load: PageLoad = async (event) => {
	await event.parent();

	try {
		await getUserManager().signinSilentCallback(event.url.toString());
		redirect(302, '/');
	} catch (e) {
		if (e instanceof Error) {
			createNotification(e.message);
		}
		redirect(302, '/signin');
	}
};
