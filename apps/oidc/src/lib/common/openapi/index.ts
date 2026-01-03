import { Configuration, type ConfigurationParameters, OidcApi } from './oidc';
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
