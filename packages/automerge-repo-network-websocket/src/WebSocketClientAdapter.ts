import {
	NetworkAdapter,
	type PeerId,
	type PeerMetadata,
	type Message
} from '@automerge/automerge-repo/slim';
import type { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';
import {
	isErrorMessage,
	isPeerMessage,
	type FromServerMessage,
	type JoinMessage
} from './messages.js';
import { decode, encode } from 'cbor-x';
import { toArrayBuffer } from './toArrayBuffer.js';
import { blobToUint8Array } from './blobToUint8Array.js';

export class WebSocketClientAdapter extends NetworkAdapter {
	#peerId: PeerId | null = null;
	#peerMetadata: PeerMetadata | null = null;
	#remotePeerId?: PeerId;
	#ws: KeepAliveWebSocket;

	#ready = false;
	// @ts-expect-error private field initialization
	#readyResolve: () => void;
	#readyPromise: Promise<void> = new Promise((resolve) => {
		this.#readyResolve = resolve;
	});

	constructor(ws: KeepAliveWebSocket) {
		super();

		this.#ws = ws;

		this.#ws.on('open', () => {
			this.#onOpen();
		});
		this.#ws.on('close', () => {
			this.#onClose();
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

	isReady(): boolean {
		return this.#ready;
	}

	async whenReady(): Promise<void> {
		return this.#readyPromise;
	}

	async reconnect() {
		await this.#ws.close().connect();
		return this;
	}

	connect(peerId: PeerId, peerMetadata?: PeerMetadata) {
		this.#peerId = peerId;
		this.#peerMetadata = peerMetadata || {};
		// Mark this adapter as ready if we haven't received an ack in 1 second.
		// We might hear back from the other end at some point but we shouldn't
		// hold up marking things as unavailable for any longer
		setTimeout(() => this.#forceReady(), 1000);
		this.#join();
	}

	send(message: Message) {
		if ('data' in message && message.data?.byteLength === 0) {
			throw new Error('Tried to send a zero-length message');
		}
		if (this.#peerId === null) {
			throw new Error('Not connected');
		}
		if (this.#ws.isClosed()) {
			console.debug('Tried to send on a disconnected socket.');
			return;
		}

		const encoded = encode(message);
		void this.#ws.send(toArrayBuffer(encoded));
	}

	disconnect(): void {
		if (!this.#ws.isClosed()) {
			this.#ws.close();
		}
		if (this.#remotePeerId) {
			this.emit('peer-disconnected', { peerId: this.#remotePeerId });
		}
	}

	#forceReady() {
		if (!this.#ready) {
			this.#ready = true;
			this.#readyResolve();
		}
	}

	#onOpen = () => {
		this.#join();
	};

	#onClose = () => {
		if (this.#remotePeerId) {
			this.emit('peer-disconnected', { peerId: this.#remotePeerId });
		}
	};

	#onMessage(messageBytes: Uint8Array) {
		if (messageBytes.byteLength === 0) {
			throw new Error('received a zero-length message');
		}
		const message: FromServerMessage = decode(messageBytes);

		if (isPeerMessage(message)) {
			this.#peerCandidate(message.senderId, message.peerMetadata);
		} else if (isErrorMessage(message)) {
			// Error messages are just logged, not thrown
		} else {
			this.emit('message', message);
		}
	}

	#peerCandidate(remotePeerId: PeerId, peerMetadata: PeerMetadata) {
		this.#forceReady();
		this.#remotePeerId = remotePeerId;
		this.emit('peer-candidate', {
			peerId: remotePeerId,
			peerMetadata
		});
	}

	#join() {
		if (this.#peerId === null || this.#peerMetadata === null) {
			return;
		}
		if (this.#ws.isReady()) {
			this.send(joinMessage(this.#peerId, this.#peerMetadata) as never as Message);
		}
		// If socket is not ready, we'll try again in the onOpen handler
	}
}

function joinMessage(senderId: PeerId, peerMetadata: PeerMetadata): JoinMessage {
	return {
		type: 'join',
		senderId,
		peerMetadata
	};
}
