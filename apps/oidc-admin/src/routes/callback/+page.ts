import { getUserManager } from '$lib/common/state/user.svelte';
import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';
import { createNotification } from '$lib/common/state/notifications.svelte';
import { resolve } from '$app/paths';

export const load: PageLoad = async (event) => {
	await event.parent();

	try {
		await getUserManager().signinCallback(event.url.toString());
	} catch (e) {
		if (e instanceof Error) {
			createNotification(e.message);
		}
		redirect(302, resolve('/signin'));
	}
	redirect(302, resolve('/'));
};
