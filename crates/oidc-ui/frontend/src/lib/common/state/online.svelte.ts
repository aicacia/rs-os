import { browser } from '$app/environment';

let online = $state(typeof navigator === 'object' ? navigator.onLine : false);

export function isOnline() {
	return online;
}

function onOnline() {
	online = true;
}

function onOffline() {
	online = false;
}

if (browser) {
	window.addEventListener('online', onOnline);
	window.addEventListener('offline', onOffline);
}
