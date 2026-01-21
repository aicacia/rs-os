import { browser } from '$app/environment';
import { getUserManager } from './user.svelte';
import { env } from '$env/dynamic/public';
import { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';
import { WebRTCClientAdapter } from '@aicacia/automerge-repo-network-webrtc';
import { WebSocketClientAdapter } from '@aicacia/automerge-repo-network-websocket';
import { SignalingRoom } from '@aicacia/signaling-room';

export const signalingRoomWebsocket = new KeepAliveWebSocket({
	autoconnect: false,
	url: ''
});

export const signalingRoom = new SignalingRoom(signalingRoomWebsocket);

export const webRTCClientAdapter = new WebRTCClientAdapter(signalingRoom);

export const documentStoreWebsocket = new KeepAliveWebSocket({
	autoconnect: false,
	url: ''
});

export const webSocketClientAdapter = new WebSocketClientAdapter(documentStoreWebsocket);

if (browser) {
	$effect.root(() => {
		$effect(() => {
			getUserManager().then(async (userManager) => {
				const user = await userManager.getUser();

				if (!user) {
					console.warn('No user found, cannot connect to document store WebSocket');
					return;
				}
				await Promise.all([
					documentStoreWebsocket.setUrl(
						`${env.PUBLIC_OS_DOCUMENT_STORE_WS_URL}/private?token=${user.access_token}`
					)
					// signalingRoomWebsocket.setUrl(
					// 	`${env.PUBLIC_OS_SIGNALING_WS_URL}/private?token=${user.access_token}&room=signaling`
					// )
				]);
			});
		});
	});
}
