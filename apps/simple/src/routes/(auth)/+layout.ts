import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { resolve } from '$app/paths';
import { getUserManager } from '$lib/common/state/user.svelte';


export const load: LayoutLoad = async (event) => {
	await event.parent();

	const currentUser = await getUserManager().getUser();

	if (currentUser) {
		return {
			user: currentUser
		};
	} else {
		redirect(302, resolve('/signin'));
	}
};
