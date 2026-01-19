import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { oidcApi, setAuthToken, getAuthToken } from '../openapi';
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
	return hasPermissionInternal(user, permission);
}

export function hasPermissions(user: UserInfo, permissions: Permission[]): boolean {
	return permissions.every((p) => hasPermissionInternal(user, p));
}

function hasPermissionInternal(user: UserInfo, permission: Permission): boolean {
	const userPermissions = user.permissions[env.PUBLIC_OS_OIDC_APPLICATION_URN] ?? [];

	// Check for exact match
	if (userPermissions.includes(permission)) {
		return true;
	}

	// Check for wildcard matches
	return userPermissions.some((userPerm) => {
		if (userPerm === '*') {
			return true;
		}
		if (userPerm.endsWith('*')) {
			const prefix = userPerm.slice(0, -1);
			return permission.startsWith(prefix);
		}
		return false;
	});
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

	if (requiresPasswordReset()) {
		// Force the password reset flow before allowing the user to continue
		await goto(resolve('/password-reset'));
		return user;
	}

	await afterSigninRedirect();
	return user;
}

export function requiresPasswordReset(): boolean {
	return token.item?.passwordResetRequired === true;
}

export async function resetPassword(newPassword: string, confirmPassword: string) {
	const authToken = getAuthToken();
	if (!authToken) {
		throw new Error('Not authenticated');
	}

	const url = new URL('/oidc/api/reset-password', env.PUBLIC_OS_OIDC_API_URL);
	const response = await fetch(url, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			Authorization: `Bearer ${authToken}`
		},
		body: JSON.stringify({
			new_password: newPassword,
			confirm_password: confirmPassword
		})
	});

	if (!response.ok) {
		let message = 'Failed to reset password';
		try {
			const body = await response.json();
			if (body?.error?.message) {
				message = body.error.message;
			} else if (body?.message) {
				message = body.message;
			}
		} catch (_e) {
			// ignore JSON parse errors and use default message
		}
		throw new Error(message);
	}

	// Clear the local reset flag so downstream checks allow navigation
	if (token.item) {
		token.item = { ...token.item, passwordResetRequired: false };
	}

	// Refresh user info to keep local cache consistent
	try {
		userInfo.item = await oidcApi.userInfo();
	} catch (e) {
		handleError(e);
	}
}

function resetAuth() {
	userInfo.item = null;
	token.item = null;
	setAuthToken(undefined);
}

export async function logout() {
	resetAuth();
}
