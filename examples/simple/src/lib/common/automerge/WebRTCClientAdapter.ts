import {
	NetworkAdapter,
	type Message,
	type PeerId,
	type PeerMetadata
} from '@automerge/automerge-repo';
import type { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';
import EventEmitter from 'eventemitter3';
import { Peer, type SignalMessage } from '@aicacia/peer';
import { Encoder, decode as cborXdecode } from 'cbor-x';

type ClientMessage =
	| {
			type: 'send';
			to: string;
			payload: unknown;
	  }
	| {
			type: 'broadcast';
			payload: unknown;
	  };

type ServerMessage =
	| {
			type: 'welcome';
			id: string;
			peers: string[];
	  }
	| {
			type: 'join';
			from: string;
			payload: unknown;
	  }
	| {
			type: 'leave';
			from: string;
	  }
	| {
			type: 'message';
			from: string;
			payload: unknown;
	  };

type ArriveSyncMessage = Omit<Message, 'targetId'> & {
	type: 'arrive';
	peerMetadata: PeerMetadata;
};

type WelcomeSyncMessage = Message & {
	type: 'welcome';
	peerMetadata: PeerMetadata;
};

type SyncMessage = ArriveSyncMessage | WelcomeSyncMessage | Message;

interface WebRTCClientAdapterInternalEvents {
	connect(): void;
	ready(): void;
}

export class WebRTCClientAdapter extends NetworkAdapter {
	#ready = false;
	#connected = false;
	#emitter = new EventEmitter<WebRTCClientAdapterInternalEvents>();

	#remotePeers: Map<string, Peer> = new Map();
	#signalingIdToPeerId: Map<string, string> = new Map();

	#websocket: KeepAliveWebSocket;

	constructor(websocket: KeepAliveWebSocket) {
		super();

		websocket.off('message', this.#onWebSocketMessage);
		websocket.on('message', this.#onWebSocketMessage);
		this.#websocket = websocket;
	}

	isReady() {
		return this.#ready;
	}

	whenReady() {
		if (this.#ready) {
			return Promise.resolve();
		}
		return new Promise<void>((resolve) => this.#emitter.once('ready', resolve));
	}

	isConnected() {
		return this.#connected;
	}

	whenConnected() {
		if (this.#connected) {
			return Promise.resolve();
		}
		return new Promise<void>((resolve) => this.#emitter.once('connect', resolve));
	}

	async connect(peerId: PeerId, peerMetadata?: PeerMetadata) {
		this.peerId = peerId;
		this.peerMetadata = peerMetadata;

		this.#forceConnected();
	}

	disconnect() {
		for (const peer of this.#remotePeers.values()) {
			peer.close();
		}
		this.#remotePeers.clear();
		this.#signalingIdToPeerId.clear();
		this.#ready = false;
		this.#connected = false;
		this.emit('close');
	}

	async send(message: Message) {
		console.log('WebRTCClientAdapter sending message', message);

		await Promise.all(
			[...this.#remotePeers.values()].map(async (peer) => {
				try {
					if (!peer.isReady()) {
						return;
					}
					peer.send(
						toArrayBuffer(
							encode({
								...message,
								senderId: this.peerId!
							} as Message)
						)
					);
				} catch (error) {
					console.error(`send error ${peer.getId()}`, error);
				}
			})
		);
	}

	#receive(fromSignalingId: string, messageBytes: Uint8Array) {
		if (messageBytes.byteLength === 0) {
			throw new Error('received a zero-length message');
		}
		const syncMessage: SyncMessage = decode(messageBytes);

		console.log('WebRTCClientAdapter received message', syncMessage);

		switch (syncMessage.type) {
			case 'arrive': {
				const message = syncMessage as ArriveSyncMessage;
				const peer = this.#remotePeers.get(fromSignalingId);

				console.assert(peer != null, 'remote peer is not set');

				peer!.send(
					toArrayBuffer(
						encode({
							type: 'welcome',
							senderId: this.peerId!,
							targetId: message.senderId,
							peerMetadata: this.peerMetadata!
						} as WelcomeSyncMessage)
					)
				);
				this.#signalingIdToPeerId.set(fromSignalingId, message.senderId);
				this.emit('peer-candidate', {
					peerId: message.senderId,
					peerMetadata: message.peerMetadata
				});
				this.#forceReady();
				break;
			}
			case 'welcome': {
				const message = syncMessage as WelcomeSyncMessage;
				this.#signalingIdToPeerId.set(fromSignalingId, message.senderId);
				this.emit('peer-candidate', {
					peerId: message.senderId,
					peerMetadata: message.peerMetadata
				});
				this.#forceReady();
				break;
			}
			default: {
				if (!syncMessage.data) {
					break;
				}
				this.emit('message', syncMessage as Message);
				break;
			}
		}
	}

	#forceReady = () => {
		if (!this.#ready) {
			this.#ready = true;
			this.#emitter.emit('ready');
		}
	};

	#forceConnected = () => {
		if (!this.#connected) {
			this.#connected = true;
			this.#emitter.emit('connect');
		}
	};

	#createPeer = async (signalingId: string, isInitiator: boolean) => {
		let peer = this.#remotePeers.get(signalingId);

		if (peer) {
			return peer;
		}

		peer = new Peer({ id: signalingId });
		this.#remotePeers.set(signalingId, peer);

		peer.on('signal', async (signalPayload) => {
			this.#websocket.send(
				JSON.stringify({
					type: 'send',
					to: signalingId,
					payload: signalPayload
				} as ClientMessage)
			);
		});
		peer.on('data', (data) => {
			this.#receive(peer.getId(), new Uint8Array(data as ArrayBufferLike));
		});
		peer.on('close', () => {
			const peerId = this.#signalingIdToPeerId.get(signalingId) as PeerId;
			if (peerId) {
				this.emit('peer-disconnected', { peerId });
			}
			this.#remotePeers.delete(signalingId);
			this.#signalingIdToPeerId.delete(signalingId);
		});
		peer.once('connect', async () => {
			await this.whenConnected();
			console.assert(this.peerId != null, 'peerId is not set');
			peer.send(
				toArrayBuffer(
					encode({
						type: 'arrive',
						senderId: this.peerId!,
						peerId: this.peerId!,
						peerMetadata: this.peerMetadata!
					} as ArriveSyncMessage)
				)
			);
		});

		if (isInitiator) {
			await peer.init();
		}

		return peer;
	};

	#onWebSocketMessage = async (
		data: string | ArrayBufferLike | Blob | ArrayBufferView<ArrayBufferLike>
	) => {
		const signalingMessage = JSON.parse(data as string) as ServerMessage;

		switch (signalingMessage.type) {
			case 'welcome': {
				try {
					await Promise.all(
						signalingMessage.peers.map((signalingId) => this.#createPeer(signalingId, true))
					);
				} catch (e) {
					console.error('Error creating peers from peers message', e);
				}
				break;
			}
			case 'join': {
				break;
			}
			case 'leave': {
				const peer = this.#remotePeers.get(signalingMessage.from);
				if (peer) {
					peer.close();
				}
				break;
			}
			case 'message': {
				const peer =
					this.#remotePeers.get(signalingMessage.from) ??
					(await this.#createPeer(signalingMessage.from, false));

				if (peer) {
					peer.signal(signalingMessage.payload as SignalMessage);
				}
				break;
			}
		}
	};
}

function toArrayBuffer(bytes: Uint8Array) {
	return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer;
}

const ENCODER = new Encoder({ tagUint8Array: false, useRecords: false });

function encode(obj: unknown): Buffer {
	return ENCODER.encode(obj);
}

function decode<T = unknown>(buf: Buffer | Uint8Array): T {
	return cborXdecode(buf);
}
