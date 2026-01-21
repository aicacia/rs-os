import { Configuration, type ConfigurationParameters, ClientApi, UserApi } from './oidc-admin';
import { env } from '$env/dynamic/public';
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { setAfterSigninRedirectPathFromURL } from '../state/afterSignInRedirectPath';
import { page } from '$app/state';

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
		},
		{
			post: async (context) => {
				if (context.response.status === 401) {
					setAfterSigninRedirectPathFromURL(page.url);
					setAuthToken(undefined);
					await goto(resolve('/signin'));
				}
				return context.response;
			}
		}
	],
	accessToken() {
		return authToken as string;
	},
	basePath: new URL(env.PUBLIC_OS_OIDC_ADMIN_API_URL).origin,
	credentials: 'same-origin'
};

export const adminConfiguration = new Configuration(defaultConfigurationParameters);

export const clientApi = new ClientApi(adminConfiguration);
export const userApi = new UserApi(adminConfiguration);

export function setAuthToken(newAuthToken?: string) {
	authToken = newAuthToken;
}
export function getAuthToken() {
	return authToken;
}
