import type { PageLoad } from './$types';
import { clientApi } from '$lib/common/openapi';

export const load: PageLoad = async (event) => {
	await event.parent();
	const client = await clientApi.clientByClientId({ clientId: event.params.clientId });
	return { client };
};
