import { browser } from '$app/environment';
import { getUserManager } from './user.svelte';
import { PUBLIC_OS_DOCUMENT_STORE_WS_URL } from '$env/static/public';
import { WebSocketClientAdapter } from '../automerge/WebSocketClientAdapter';

let websocketURL: string | null = $state(null);

export const webSocketClientAdapter = new WebSocketClientAdapter({
	url: () => websocketURL as string
});

if (browser) {
	getUserManager()
		.getUser()
		.then((user) => {
			if (!user) {
				console.warn('No user found, cannot connect to document store WebSocket');
				return;
			}
			console.log('User found, connecting to document store WebSocket');
			websocketURL = `${PUBLIC_OS_DOCUMENT_STORE_WS_URL}/ws?token=${user.access_token}`;
			webSocketClientAdapter.reconnect();
		});
}
