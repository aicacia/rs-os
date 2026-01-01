import { json } from '@sveltejs/kit';

export const prerender = true;

export async function GET() {
	return json([
		{
			relation: ['delegate_permission/common.handle_all_urls'],
			target: {
				namespace: 'com.aicacia.os',
				package_name: 'os',
				sha256_cert_fingerprints: []
			}
		}
	]);
}
