import { base } from '$app/paths';
import { PUBLIC_URL } from '$env/static/public';
import { json } from '@sveltejs/kit';
import icon256x256Png from '$lib/assets/icon256x256.png';

export const prerender = true;

export async function GET() {
	return json({
		name: 'Simple',
		client_id: `${PUBLIC_URL}${base}/client.json`,
		redirect_uris: [
			`${PUBLIC_URL}${base}/callback`,
			`${PUBLIC_URL}${base}/popup-callback`,
			`${PUBLIC_URL}${base}/silent-callback`
		],
		post_logout_redirect_uris: [`${PUBLIC_URL}${base}/logout`],
		logo_uri: `${PUBLIC_URL}${icon256x256Png}`,
		client_uri: `${PUBLIC_URL}${base}`,
		policy_uri: `${PUBLIC_URL}${base}/policy`,
		terms_of_service_uri: `${PUBLIC_URL}${base}${'/terms'}`,
		application_type: 'web',
		auth_method: 'none',
		grant_types: ['authorization_code', 'refresh_token', 'password'],
		response_types: [
			'code',
			'token',
			'id_token',
			'code token',
			'code id_token',
			'token id_token',
			'code token id_token',
			'none'
		],
		scopes: ['openid', 'profile', 'address', 'offline', 'email', 'phone_number'],
		audience: [`${PUBLIC_URL}${base}`],
		access_token_expires_in_seconds: 3600,
		id_token_expires_in_seconds: 3600,
		refresh_expires_in_seconds: 604800
	});
}
