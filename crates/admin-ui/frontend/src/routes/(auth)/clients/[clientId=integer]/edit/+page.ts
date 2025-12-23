import type { PageLoad } from './$types';
import { clientApi } from '$lib/common/openapi';

export const load: PageLoad = async (event) => {
	await event.parent();
	const clientId = Number.parseInt(event.params.clientId, 10);
	const client = await clientApi.clientById({ clientId });
	return { client };
};
