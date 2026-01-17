import { Configuration, type ConfigurationParameters, OidcApi } from './oidc';
import { env } from '$env/dynamic/public';
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';

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
	basePath: new URL(env.PUBLIC_OS_OIDC_API_URL).origin,
	credentials: 'same-origin'
};

export const oidcConfiguration = new Configuration(defaultConfigurationParameters);

export const oidcApi = new OidcApi(oidcConfiguration);

export function setAuthToken(newAuthToken?: string) {
	authToken = newAuthToken;
}
export function getAuthToken() {
	return authToken;
}
