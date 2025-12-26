import { browser } from '$app/environment';
import { getUserManager } from './user.svelte';
import { PUBLIC_OS_SIGNALING_WS_URL } from '$env/static/public';
import { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';

let websocketURL = $state(`${PUBLIC_OS_SIGNALING_WS_URL}/user`);

export const websocket = new KeepAliveWebSocket({
	autoconnect: false,
	url: () => websocketURL as string
});

if (browser) {
	$effect.root(() => {
		$effect(() => {
			websocket.close();

			getUserManager()
				.getUser()
				.then(async (user) => {
					if (!user) {
						console.warn('No user found, cannot connect to document store WebSocket');
						return;
					}
					websocketURL = `${PUBLIC_OS_SIGNALING_WS_URL}/user?token=${user.access_token}`;
					await websocket.connect();
				});
		});
	});
}
