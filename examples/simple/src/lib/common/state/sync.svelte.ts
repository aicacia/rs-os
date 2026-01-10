import { browser } from '$app/environment';
import { getUserManager } from './user.svelte';
import { env } from '$env/dynamic/public';
import { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';
import { WebRTCClientAdapter } from '@aicacia/automerge-repo-network-webrtc';
import { SignalingRoom } from '@aicacia/signaling-room';

export const signalingRoomWebsocket = new KeepAliveWebSocket({
	autoconnect: false,
	url: ''
});

export const signalingRoom = new SignalingRoom(signalingRoomWebsocket);

export const webRTCClientAdapter = new WebRTCClientAdapter(signalingRoom);

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
					await signalingRoomWebsocket.setUrl(
						`${env.PUBLIC_OS_SIGNALING_WS_URL}/private?token=${user.access_token}&room=signaling`
					);
				});
		});
	});
}
