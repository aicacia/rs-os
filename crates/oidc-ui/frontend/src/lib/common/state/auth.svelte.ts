import {
	defaultConfigurationParameters,
	oidcApi,
	oidcConfiguration,
	setAuthToken
} from '../openapi';
import {
	Configuration,
	OidcApi,
	OpenIdClaimsFromJSON,
	OpenIdClaimsToJSON,
	Permission,
	TokenFromJSON,
	TokenToJSON,
	type OpenIdClaims,
	type Token
} from '../openapi/oidc';
import { localStorageState } from '../util/localStorageState.svelte';
import { afterSigninRedirect } from './afterSignInRedirectPath';
import { isOnline } from './online.svelte';
import { handleError } from '../errors';

const userInfo = localStorageState<OpenIdClaims | null>('user_info', null, {
	serializer: {
		parse: (text) => OpenIdClaimsFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(OpenIdClaimsToJSON(value))
	}
});
const token = localStorageState<Token | null>('token', null, {
	serializer: {
		parse: (text) => TokenFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(TokenToJSON(value))
	}
});

const currentUserInfo = $derived.by(async () => {
	try {
		if (isOnline()) {
			if (token.value) {
				if (token.value.issuedAt.getTime() + token.value.expiresIn * 1000 < Date.now()) {
					if (token.value.refreshToken && token.value.refreshTokenExpiresIn) {
						if (
							token.value.issuedAt.getTime() + token.value.refreshTokenExpiresIn * 1000 <
							Date.now()
						) {
							throw new Error('refresh token expired');
						}
						token.value = await oidcApi.token({
							grantType: 'refresh_token',
							refreshToken: token.value.refreshToken,
							scope: 'openid profile offline'
						});
					} else {
						throw new Error('no refresh token');
					}
				}
				setAuthToken(token.value.accessToken);
				userInfo.value = await new OidcApi(
					new Configuration({
						...defaultConfigurationParameters,
						accessToken: token.value.idToken ?? token.value.accessToken
					})
				).userInfo();
			} else {
				throw new Error('not authorized');
			}
		}
		return userInfo.value;
	} catch (e) {
		if (!(e instanceof Error && e.message === 'not authorized')) {
			handleError(e);
		}
		resetAuth();
		return null;
	}
});

export async function getCurrentUserInfo() {
	return await currentUserInfo;
}

export function hasPermission(user: OpenIdClaims, permission: string): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return hasPermissionInternal(user, permission);
}

export function hasPermissions(user: OpenIdClaims, permissions: string[]): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return permissions.every((p) => hasPermissionInternal(user, p));
}

function hasAdminAll(user: OpenIdClaims): boolean {
	return hasPermissionInternal(user, Permission.Admin);
}

function hasPermissionInternal(user: OpenIdClaims, permission: string): boolean {
	return new RegExp(`\\b${permission.replaceAll('.', '\\.')}\\b`).test(user.scope);
}

export async function signInUsernamePassword(username: string, password: string) {
	token.value = await oidcApi.token({
		grantType: 'password',
		username,
		password,
		scope: 'openid profile address email phone_number offline'
	});
	const user = await currentUserInfo;
	await afterSigninRedirect();
	return user;
}

function resetAuth() {
	userInfo.value = null;
	token.value = null;
	setAuthToken(undefined);
}

export async function logout() {
	resetAuth();
}
