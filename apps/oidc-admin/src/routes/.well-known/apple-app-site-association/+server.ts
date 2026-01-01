import { json } from '@sveltejs/kit';

export const prerender = true;

export async function GET() {
	return json({
		applinks: {
			details: [
				{
					appIDs: ['com.aicacia.os'],
					components: [
						{
							'/': '/*',
							comment: 'Matches any URL whose path starts with /'
						}
					]
				}
			]
		}
	});
}
