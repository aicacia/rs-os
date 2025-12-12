import type { PageLoad } from './$types';
import { userApi } from '$lib/common/openapi';
import { handleError } from '$lib/common/errors';

export const load: PageLoad = async () => {
	try {
		const users = await userApi.userList();
		return { users };
	} catch (e) {
		await handleError(e);
		return { users: [] };
	}
};
