import {
	Configuration,
	type ConfigurationParameters,
	CurrentUserApi,
	ClientApi,
	OidcApi,
	PasswordApi
} from './oidc';
import { PUBLIC_OS_OIDC_API_URL } from '$env/static/public';

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
	basePath: PUBLIC_OS_OIDC_API_URL,
	credentials: 'same-origin'
};

export const oidcConfiguration = new Configuration(defaultConfigurationParameters);

export const currentUserApi = new CurrentUserApi(oidcConfiguration);
export const clientApi = new ClientApi(oidcConfiguration);
export const oidcApi = new OidcApi(oidcConfiguration);
export const passwordApi = new PasswordApi(oidcConfiguration);

export function setAuthToken(newAuthToken?: string) {
	authToken = newAuthToken;
}
export function getAuthToken() {
	return authToken;
}
