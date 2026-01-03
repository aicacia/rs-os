import { oidcApi, setAuthToken } from '../openapi';
import {
	Permission,
	TokenFromJSON,
	TokenToJSON,
	type UserInfo,
	type Token,
	UserInfoFromJSON,
	UserInfoToJSON
} from '../openapi/oidc';
import { localStorageState } from '../util/localStorageState.svelte';
import { afterSigninRedirect } from './afterSignInRedirectPath';
import { isOnline } from './online.svelte';
import { handleError } from '../errors';
import { env } from '$env/dynamic/public';

const userInfo = localStorageState<UserInfo | null>('user_info', null, {
	serializer: {
		parse: (text) => UserInfoFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(UserInfoToJSON(value))
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
							clientId: env.PUBLIC_URL,
							refreshToken: token.value.refreshToken,
							scope: 'openid profile offline'
						});
					} else {
						throw new Error('no refresh token');
					}
				}
				setAuthToken(token.value.accessToken);
				userInfo.value = await oidcApi.userInfo();
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

export function hasPermission(user: UserInfo, permission: Permission): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return hasPermissionInternal(user, permission);
}

export function hasPermissions(user: UserInfo, permissions: Permission[]): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return permissions.every((p) => hasPermissionInternal(user, p));
}

function hasAdminAll(user: UserInfo): boolean {
	return hasPermissionInternal(user, Permission.Admin);
}

function hasPermissionInternal(user: UserInfo, permission: Permission): boolean {
	return user.permissions.includes(permission) ?? false;
}

export async function signInUsernamePassword(username: string, password: string) {
	token.value = await oidcApi.token({
		grantType: 'password',
		clientId: env.PUBLIC_URL,
		username,
		password,
		scope: 'openid profile address email phone offline'
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
