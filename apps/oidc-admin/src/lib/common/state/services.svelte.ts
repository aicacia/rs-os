import { env } from '$env/dynamic/public';
import { ServiceDiscovery } from '@aicacia/service-discovery';
import { createStorage } from '@aicacia/svelte-headless';

const osBaseUrl = createStorage('os-base-url', env.PUBLIC_OS_SERVICE_DISCOVERY_BASE_URL);

export function getOSBaseUrl() {
	return osBaseUrl.item;
}

export function setOSBaseUrl(value: string) {
	osBaseUrl.item = value;
}

let isReady = $state(false);

const serviceDiscovery = new ServiceDiscovery(osBaseUrl.item);

$effect.root(() => {
	$effect(() => {
		isReady = false;
		serviceDiscovery.setBaseUrl(osBaseUrl.item);
	});
});

serviceDiscovery.on('discovered-services', () => {
	isReady = true;
});

export function getIsServicesReady() {
	return isReady;
}

export async function getOIDCAPIURL() {
	const services = await serviceDiscovery.waitForServices();
	return services.oidc;
}
