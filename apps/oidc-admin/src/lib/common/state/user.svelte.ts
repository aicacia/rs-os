import { UserManager, type UserManagerSettings } from 'oidc-client-ts';
import { browser } from '$app/environment';
import icon256x256Png from '$lib/assets/icon256x256.png';
import { PUBLIC_OS_OIDC_API_URL, PUBLIC_URL } from '$env/static/public';
import { Permission, type UserInfo } from '../openapi/oidc';
import { setAuthToken } from '../openapi';

const userSettings = browser
	? ({
			authority: PUBLIC_OS_OIDC_API_URL,
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
					name: 'OIDC Admin UI',
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
		} satisfies UserManagerSettings)
	: ({} as never);

const userManager = $derived.by(() => new UserManager({ ...userSettings }));

export function getUserManager() {
	return userManager;
}

const user = $derived.by<Promise<UserInfo | null>>(async () => {
	try {
		const manager = getUserManager();
		const user = await manager.getUser();
		return user?.profile as unknown as UserInfo | null;
	} catch (e) {
		console.error('Error getting user from UserManager', e);
		return null;
	}
});

export function getUser() {
	return user;
}

export function hasPermission(user: UserInfo, permission: string): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return hasPermissionInternal(user, permission);
}

export function hasPermissions(user: UserInfo, permissions: string[]): boolean {
	if (hasAdminAll(user)) {
		return true;
	}
	return permissions.every((p) => hasPermissionInternal(user, p));
}

function hasAdminAll(user: UserInfo): boolean {
	return hasPermissionInternal(user, Permission.Admin);
}

function hasPermissionInternal(user: UserInfo, permission: string): boolean {
	return user.permissions.includes(permission as Permission);
}

if (browser) {
	$effect.root(() => {
		$effect(() => {
			getUserManager()
				.getUser()
				.then((user) => {
					if (!user) {
						setAuthToken(undefined);
						return;
					}
					setAuthToken(user.access_token);
				});
		});
	});
}
