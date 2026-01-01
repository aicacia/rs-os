import type { PageLoad } from './$types';
import { userApi } from '$lib/common/openapi';

export const load: PageLoad = async (event) => {
	await event.parent();
	const userId = Number.parseInt(event.params.id);
	const user = await userApi.getUser({ id: userId });
	return { user };
};
