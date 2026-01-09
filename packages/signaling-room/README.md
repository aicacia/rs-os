# Aicacia OS Signaling Room

```ts
const websocket = new WebSocket('http://localhost:3000/signaling/user');

const signalingRoom = new SignalingRoom(websocket);

signalingRoom.on('join', (peer) => {
	console.log('peer joined', peer);
});
signalingRoom.on('leave', (signalingId) => {
	console.log('peer left', peer);
});

const peers: Peer[] = signalingRoom.peers();
```
