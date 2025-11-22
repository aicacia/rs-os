import { currentUserApi, oidcApi, setAuthToken } from '../openapi';
import {
	Permission,
	TokenFromJSON,
	TokenToJSON,
	UserFromJSON,
	UserToJSON,
	type Token,
	type User
} from '../openapi/oidc';
import { localStorageState } from '../util/localStorageState.svelte';
import { afterSigninRedirect } from './afterSignInRedirectPath';
import { isOnline } from './online.svelte';
import { handleError } from '../errors';

const user = localStorageState<User | null>('user', null, {
	serializer: {
		parse: (text) => UserFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(UserToJSON(value))
	}
});
const token = localStorageState<Token | null>('token', null, {
	serializer: {
		parse: (text) => TokenFromJSON(JSON.parse(text)),
		stringify: (value) => JSON.stringify(TokenToJSON(value))
	}
});

const currentUser = $derived.by(async () => {
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
				user.value = await currentUserApi.currentUser();
			} else {
				throw new Error('not authorized');
			}
		}
		return user.value;
	} catch (e) {
		if (!(e instanceof Error && e.message === 'not authorized')) {
			handleError(e);
		}
		resetAuth();
		return null;
	}
});

export async function getCurrentUser() {
	return await currentUser;
}

export function hasPermission(user: User, permission: string): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return hasPermissionInternal(user, permission);
}

export function hasPermissions(user: User, permissions: string[]): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return permissions.every((p) => hasPermissionInternal(user, p));
}

function hasAdminAll(user: User): boolean {
	return hasPermissionInternal(user, Permission.Admin);
}

function hasPermissionInternal(user: User, permission: string): boolean {
	return user.roles.some((r) => r.permissions.includes(permission));
}

export async function signInUsernamePassword(username: string, password: string) {
	token.value = await oidcApi.token({
		grantType: 'password',
		username,
		password,
		scope: 'openid profile offline'
	});
	const user = await currentUser;
	afterSigninRedirect();
	return user;
}

function resetAuth() {
	user.value = null;
	token.value = null;
	setAuthToken(undefined);
}
