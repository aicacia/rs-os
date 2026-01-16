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
import { createStorage } from '@aicacia/svelte-headless';
import { afterSigninRedirect } from './afterSignInRedirectPath';
import { isOnline } from '@aicacia/svelte-headless';
import { handleError } from '../errors';
import { env } from '$env/dynamic/public';

const userInfo = createStorage<UserInfo | null>('user_info', null, {
	serializer: {
		parse: (text) => UserInfoFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(UserInfoToJSON(value))
	}
});
const token = createStorage<Token | null>('token', null, {
	serializer: {
		parse: (text) => TokenFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(TokenToJSON(value))
	}
});

const currentUserInfo = $derived.by(async () => {
	try {
		if (isOnline()) {
			if (token.item) {
				if (token.item.issuedAt.getTime() + token.item.expiresIn * 1000 < Date.now()) {
					if (token.item.refreshToken && token.item.refreshTokenExpiresIn) {
						if (
							token.item.issuedAt.getTime() + token.item.refreshTokenExpiresIn * 1000 <
							Date.now()
						) {
							throw new Error('refresh token expired');
						}
						token.item = await oidcApi.token({
							grantType: 'refresh_token',
							clientId: env.PUBLIC_OS_OIDC_CLIENT_ID,
							refreshToken: token.item.refreshToken,
							scope: 'openid profile offline'
						});
					} else {
						throw new Error('no refresh token');
					}
				}
				setAuthToken(token.item.accessToken);
				userInfo.item = await oidcApi.userInfo();
			} else {
				throw new Error('not authorized');
			}
		}
		return userInfo.item;
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
	return user.permissions[env.PUBLIC_OS_OIDC_APPLICATION_URN].includes(permission) ?? false;
}

export async function signInUsernamePassword(username: string, password: string) {
	token.item = await oidcApi.token({
		grantType: 'password',
		clientId: env.PUBLIC_OS_OIDC_CLIENT_ID,
		username,
		password,
		scope: 'openid profile address email phone offline'
	});
	const user = await currentUserInfo;
	await afterSigninRedirect();
	return user;
}

function resetAuth() {
	userInfo.item = null;
	token.item = null;
	setAuthToken(undefined);
}

export async function logout() {
	resetAuth();
}
