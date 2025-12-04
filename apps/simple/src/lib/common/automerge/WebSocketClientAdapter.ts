import {
	NetworkAdapter,
	type PeerId,
	type PeerMetadata,
	type Message
} from '@automerge/automerge-repo/slim';
import { KeepAliveWebSocket, type KeepAliveWebSocketOptions } from '@aicacia/keepalivewebsocket';
import {
	isErrorMessage,
	isPeerMessage,
	type FromServerMessage,
	type JoinMessage
} from './messages';
import { decode, encode } from 'cbor-x';
import { toArrayBuffer } from './toArrayBuffer';
import { blobToUint8Array } from './blobToUint8Array';

export type WebSocketClientAdapterOptions = KeepAliveWebSocketOptions;

export class WebSocketClientAdapter extends NetworkAdapter {
	#peerId: PeerId | null = null;
	#peerMetadata: PeerMetadata | null = null;
	#remotePeerId?: PeerId;
	#ws: KeepAliveWebSocket;

	#ready = false;
	#joinSent = false;
	// @ts-expect-error private field initialization
	#readyResolve: () => void;
	#readyPromise: Promise<void> = new Promise((resolve) => {
		this.#readyResolve = resolve;
	});

	constructor(options: WebSocketClientAdapterOptions) {
		super();

		this.#ws = new KeepAliveWebSocket(options);

		this.#ws.on('open', () => {
			this.#join();
		});
		this.#ws.on('close', () => {
			this.#close();
		});
		this.#ws.on('message', async (data) => {
			let uint8Array: Uint8Array;
			if (data instanceof ArrayBuffer) {
				uint8Array = new Uint8Array(data);
			} else if (data instanceof Blob) {
				uint8Array = await blobToUint8Array(data);
			} else if (ArrayBuffer.isView(data)) {
				uint8Array = new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
			} else {
				throw new Error('Received message is not an ArrayBuffer, Blob, or ArrayBufferView');
			}
			this.#onMessage(uint8Array);
		});
	}

	async reconnect() {
		await this.#ws.close().connect();
		return this;
	}

	isReady(): boolean {
		return this.#ready;
	}

	async whenReady(): Promise<void> {
		return this.#readyPromise;
	}

	#setReady(ready: boolean) {
		this.#ready = ready;
		if (ready) {
			this.#readyResolve();
		} else {
			this.#readyPromise = new Promise((resolve) => {
				this.#readyResolve = resolve;
			});
		}
	}

	connect(peerId: PeerId, peerMetadata?: PeerMetadata) {
		this.#peerId = peerId;
		this.#peerMetadata = peerMetadata || {};
		void this.#join();
	}

	send(message: Message) {
		if ('data' in message && message.data?.byteLength === 0) {
			throw new Error('Tried to send a zero-length message');
		}
		if (this.#ws.isClosed()) {
			throw new Error('WebSocket is closed');
		}

		console.debug('Sending message', message);

		void this.#ws.send(toArrayBuffer(encode(message)));
	}

	disconnect(): void {
		this.#close();
	}

	#onMessage(messageBytes: Uint8Array) {
		if (messageBytes.byteLength === 0) {
			throw new Error('received a zero-length message');
		}
		const message: FromServerMessage = decode(messageBytes);

		console.debug('Received message', message);

		if (isPeerMessage(message)) {
			this.#peerCandidate(message.senderId, message.peerMetadata);
		} else if (isErrorMessage(message)) {
			console.error(`Received error message from server`, message);
		} else {
			this.emit('message', message);
		}
	}

	#peerCandidate(remotePeerId: PeerId, peerMetadata: PeerMetadata) {
		this.#remotePeerId = remotePeerId;
		this.#joinSent = false;
		this.#setReady(true);
		this.emit('peer-candidate', {
			peerId: remotePeerId,
			peerMetadata
		});
	}

	#close() {
		this.#setReady(false);
		this.#joinSent = false;
		if (this.#remotePeerId) {
			this.emit('peer-disconnected', { peerId: this.#remotePeerId });
			this.#remotePeerId = undefined;
		}
		if (!this.#ws.isClosed()) {
			this.#ws.close();
		}
	}

	#join() {
		if (this.isReady()) {
			return;
		}
		if (this.#joinSent) {
			return;
		}
		if (this.#peerId === null || this.#peerMetadata === null) {
			return;
		}
		this.#joinSent = true;
		this.send(joinMessage(this.#peerId, this.#peerMetadata) as never as Message);
	}
}

function joinMessage(senderId: PeerId, peerMetadata: PeerMetadata): JoinMessage {
	return {
		type: 'join',
		senderId,
		peerMetadata
	};
}
