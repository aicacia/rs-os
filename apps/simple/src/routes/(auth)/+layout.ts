import { redirect } from '@sveltejs/kit';
import type { LayoutLoad } from './$types';
import { resolve } from '$app/paths';
import { getUserManager } from '$lib/common/state/user.svelte';

export const load: LayoutLoad = async (event) => {
	await event.parent();

	const user = await getUserManager().getUser();

	if (user) {
		return {
			user
		};
	} else {
		redirect(302, resolve('/signin'));
	}
};
