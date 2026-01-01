import type { PageLoad } from './$types';
import { userApi } from '$lib/common/openapi';

export const load: PageLoad = async ({ params }) => {
	const user = await userApi.getUser({ id: Number.parseInt(params.id) });
	return { user };
};
