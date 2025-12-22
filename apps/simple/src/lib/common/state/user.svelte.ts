import { UserManager, type UserManagerSettings } from 'oidc-client-ts';
import { localStorageState } from '../util/localStorageState.svelte';
import { browser } from '$app/environment';
import icon256x256Png from '$lib/assets/icon256x256.png';
import { PUBLIC_URL } from '$env/static/public';

const userSettings = browser
	? ({
			client_id: `${PUBLIC_URL}`,
			redirect_uri: `${PUBLIC_URL}/callback`,
			post_logout_redirect_uri: `${PUBLIC_URL}/logout`,
			response_type: 'code',
			scope: 'openid profile offline',
			response_mode: 'query',
			loadUserInfo: true,
			popup_redirect_uri: `${PUBLIC_URL}/popup-callback`,
			popup_post_logout_redirect_uri: `${PUBLIC_URL}/popup-callback`,
			silent_redirect_uri: `${PUBLIC_URL}/silent-callback`,
			automaticSilentRenew: true,
			filterProtocolClaims: true,
			extraQueryParams: {
				registration: JSON.stringify({
					name: 'Simple',
					client_id: `${PUBLIC_URL}`,
					redirect_uris: [
						`${PUBLIC_URL}/callback`,
						`${PUBLIC_URL}/popup-callback`,
						`${PUBLIC_URL}/silent-callback`
					],
					post_logout_redirect_uris: [`${PUBLIC_URL}/logout`],
					logo_uri: `${PUBLIC_URL}${icon256x256Png}`,
					client_uri: `${PUBLIC_URL}`,
					policy_uri: `${PUBLIC_URL}/policy`,
					terms_of_service_uri: `${PUBLIC_URL}${'/terms'}`,
					application_type: 'web',
					auth_method: 'none',
					grant_types: ['authorization_code', 'refresh_token'],
					response_types: ['code'],
					scopes: ['openid', 'profile', 'address', 'offline', 'email', 'phone'],
					audience: [`${PUBLIC_URL}`],
					access_token_expires_in_seconds: 3600,
					id_token_expires_in_seconds: 3600,
					refresh_expires_in_seconds: 604800
				})
			}
		} satisfies Omit<UserManagerSettings, 'authority'>)
	: ({} as never);

const authority = localStorageState('authority', 'http://localhost:3000/oidc/api');

const userManager = $derived.by(
	() => new UserManager({ ...userSettings, authority: authority.value })
);

export function getAuthority() {
	return authority.value;
}

export function setAuthority(value: string) {
	authority.value = value;
}

export function getUserManager() {
	return userManager;
}
