import { browser } from '$app/environment';
import { getUserManager } from './user.svelte';
import { PUBLIC_OS_SIGNALING_WS_URL } from '$env/static/public';
import { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';

const websocketType = $state<'user' | 'client' | 'anonymous'>('anonymous');
const websocketRoom = $state('test-room');

let websocketURL = $state(`${PUBLIC_OS_SIGNALING_WS_URL}/${websocketType}?room=${websocketRoom}`);

export const websocket = new KeepAliveWebSocket({
	autoconnect: false,
	url: () => websocketURL as string
});

if (browser) {
	$effect.root(() => {
		$effect(() => {
			getUserManager()
				.getUser()
				.then(async (user) => {
					if (!user) {
						console.warn('No user found, cannot connect to document store WebSocket');
						return;
					}
					console.log('User found, connecting to document store WebSocket');
					switch (websocketType) {
						case 'anonymous':
							websocketURL = `${PUBLIC_OS_SIGNALING_WS_URL}/${websocketType}?room=${websocketRoom}`;
							break;
						default:
							websocketURL = `${PUBLIC_OS_SIGNALING_WS_URL}/${websocketType}?token=${user.access_token}`;
							break;
					}
					await websocket.close().connect();
				});
		});
	});
}
