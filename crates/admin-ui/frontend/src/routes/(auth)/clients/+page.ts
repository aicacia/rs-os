import type { PageLoad } from './$types';
import { clientApi } from '$lib/common/openapi';

export const load: PageLoad = async (event) => {
	await event.parent();
	const clients = await clientApi.clientList();
	return { clients };
};
