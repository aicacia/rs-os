import { UserManager, type UserManagerSettings } from 'oidc-client-ts';
import { localStorageState } from '../util/localStorageState.svelte';
import { browser } from '$app/environment';

const userSettings = browser
	? ({
			client_id: `${window.location.origin}/client.json`,
			redirect_uri: `${window.location.origin}/callback`,
			post_logout_redirect_uri: `${window.location.origin}/logout`,
			response_type: 'code',
			scope: 'openid profile',
			response_mode: 'query',
			popup_redirect_uri: `${window.location.origin}/popup-callback`,
			popup_post_logout_redirect_uri: `${window.location.origin}/popup-callback`,
			silent_redirect_uri: `${window.location.origin}/silent-callback`,
			automaticSilentRenew: true,
			filterProtocolClaims: true
		} satisfies Omit<UserManagerSettings, 'authority'>)
	: ({} as never);

const authority = localStorageState('authority', 'http://localhost:3000/oidc/api');

export function getAuthority() {
	return authority.value;
}

export function setAuthority(value: string) {
	authority.value = value;
}

export function getUserManager() {
	return new UserManager({ ...userSettings, authority: authority.value });
}
