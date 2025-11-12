import { resolve } from '$app/paths';
import { PUBLIC_URL } from '$env/static/public';
import { json } from '@sveltejs/kit';

export const prerender = true;

export async function GET() {
	return json({
		name: 'Example',
		client_id: `${PUBLIC_URL}${resolve('/client.json')}`,
		redirect_uris: [`${PUBLIC_URL}${resolve('/callback')}`],
		post_logout_redirect_uris: [`${PUBLIC_URL}${resolve('/')}`],
		logo_uri: `${PUBLIC_URL}${resolve('/')}`,
		client_uri: `${PUBLIC_URL}${resolve('/')}`,
		policy_uri: `${PUBLIC_URL}${resolve('/policy')}`,
		terms_of_service_uri: `${PUBLIC_URL}${resolve('/terms')}`,
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
		audience: [PUBLIC_URL],
		access_token_expires_in_seconds: 3600,
		id_token_expires_in_seconds: 3600,
		refresh_expires_in_seconds: 604800
	});
}
