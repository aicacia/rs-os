import { UserManager, type UserManagerSettings } from 'oidc-client-ts';
import { browser } from '$app/environment';
import icon256x256Png from '$lib/assets/icon256x256.png';
import { env } from '$env/dynamic/public';
import { Permission, type UserInfo } from '../openapi/oidc-admin';
import { setAuthToken } from '../openapi';
import { getOIDCAPIURL } from './services.svelte';

const userSettings = async () =>
	browser
		? ({
				authority: await getOIDCAPIURL(),
				client_id: `${env.PUBLIC_URL}`,
				redirect_uri: `${env.PUBLIC_URL}/callback`,
				post_logout_redirect_uri: `${env.PUBLIC_URL}/logout`,
				response_type: 'code',
				scope: 'openid profile offline',
				response_mode: 'query',
				loadUserInfo: true,
				popup_redirect_uri: `${env.PUBLIC_URL}/popup-callback`,
				popup_post_logout_redirect_uri: `${env.PUBLIC_URL}/popup-callback`,
				silent_redirect_uri: `${env.PUBLIC_URL}/silent-callback`,
				automaticSilentRenew: true,
				filterProtocolClaims: true,
				extraQueryParams: {
					registration: JSON.stringify({
						name: 'OIDC Admin UI',
						client_id: `${env.PUBLIC_URL}`,
						redirect_uris: [
							`${env.PUBLIC_URL}/callback`,
							`${env.PUBLIC_URL}/popup-callback`,
							`${env.PUBLIC_URL}/silent-callback`
						],
						post_logout_redirect_uris: [`${env.PUBLIC_URL}/logout`],
						logo_uri: `${window.location.origin}${icon256x256Png}`,
						client_uri: `${env.PUBLIC_URL}`,
						policy_uri: `${env.PUBLIC_URL}/policy`,
						terms_of_service_uri: `${env.PUBLIC_URL}${'/terms'}`,
						application_type: 'web',
						auth_method: 'none',
						grant_types: ['authorization_code', 'refresh_token'],
						response_types: ['code'],
						scopes: ['openid', 'profile', 'address', 'offline', 'email', 'phone'],
						audience: [`${env.PUBLIC_URL}`],
						access_token_expires_in_seconds: 3600,
						id_token_expires_in_seconds: 3600,
						refresh_expires_in_seconds: 604800
					})
				}
			} satisfies UserManagerSettings)
		: ({} as never);

const userManager = $derived.by(async () => new UserManager({ ...(await userSettings()) }));

export async function getUserManager() {
	return await userManager;
}

const user = $derived.by<Promise<UserInfo | null>>(async () => {
	try {
		const userManager = await getUserManager();
		const user = await userManager.getUser();
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
			getUserManager().then(async (userManager) => {
				const user = await userManager.getUser();
				if (!user) {
					setAuthToken(undefined);
					return;
				}
				setAuthToken(user.access_token);
			});
		});
	});
}
