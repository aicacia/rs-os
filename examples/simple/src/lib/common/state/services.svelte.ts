import { ServiceDiscovery } from '@aicacia/service-discovery';
import { createStorage } from '@aicacia/svelte-headless';

const osBaseUrl = createStorage('os-base-url', 'http://localhost:3000');

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

export async function getServices() {
	return await serviceDiscovery.waitForServices();
}

export async function getOIDCAPIURL() {
	return (await getServices()).oidc;
}
