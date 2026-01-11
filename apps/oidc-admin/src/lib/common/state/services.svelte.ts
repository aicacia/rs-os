import { env } from '$env/dynamic/public';
import { createServices } from '@aicacia/service-discovery';

const services = createServices(env.PUBLIC_OS_SERVICE_DISCOVERY_BASE_URL);

export async function getOIDCAPIURL() {
	return (await services).oidc;
}
