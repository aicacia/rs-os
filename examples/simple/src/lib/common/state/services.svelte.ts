import { createServices } from '@aicacia/service-discovery';
import { createStorage } from '@aicacia/svelte-headless';

const osBaseUrl = createStorage('os-base-url', 'http://localhost:3000');

export function getOSBaseUrl() {
	return osBaseUrl.item;
}

export function setOSBaseUrl(value: string) {
	osBaseUrl.item = value;
}

const services = $derived(createServices(osBaseUrl.item));

export async function getOIDCAPIURL() {
	return (await services).oidc;
}
