import {
	Configuration,
	type ConfigurationParameters,
	CurrentUserApi,
	ClientApi,
	UserApi
} from './oidc-admin';
import { env } from '$env/dynamic/public';

let authToken: string | undefined;

export const defaultConfigurationParameters: ConfigurationParameters = {
	middleware: [
		{
			pre: async (context) => ({
				...context,
				init: {
					...context.init,
					mode: 'cors'
				}
			})
		}
	],
	accessToken() {
		return authToken as string;
	},
	basePath: new URL(env.PUBLIC_OS_ADMIN_API_URL).origin,
	credentials: 'same-origin'
};

export const adminConfiguration = new Configuration(defaultConfigurationParameters);

export const currentUserApi = new CurrentUserApi(adminConfiguration);
export const clientApi = new ClientApi(adminConfiguration);
export const userApi = new UserApi(adminConfiguration);

export function setAuthToken(newAuthToken?: string) {
	authToken = newAuthToken;
}
export function getAuthToken() {
	return authToken;
}
