import {
	NetworkAdapter,
	type Message,
	type PeerId,
	type PeerMetadata
} from '@automerge/automerge-repo/slim';
import { EventEmitter } from 'eventemitter3';
import { SignalingRoom } from '@aicacia/signaling-room';
import { Encoder, decode as cborXdecode } from 'cbor-x';
import { Peer } from '@aicacia/peer';

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

	#signalingRoom: SignalingRoom;
	#signalingIdToPeerId: Map<string, string> = new Map();

	constructor(signalingRoom: SignalingRoom) {
		super();

		signalingRoom.on('join', this.#onPeerJoin);
		signalingRoom.on('connect', this.#onPeerConnect);
		signalingRoom.on('leave', this.#onPeerLeave);
		signalingRoom.getPeers().forEach(this.#onPeerJoin);
		this.#signalingRoom = signalingRoom;
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
		this.#signalingRoom.off('join', this.#onPeerJoin);
		this.#signalingRoom.off('connect', this.#onPeerConnect);
		this.#signalingRoom.off('leave', this.#onPeerLeave);
		this.#signalingRoom.close();
		this.#signalingIdToPeerId.clear();
		this.#ready = false;
		this.#connected = false;
		this.emit('close');
	}

	async send(message: Message) {
		await Promise.all(
			this.#signalingRoom.getPeers().map(async (peer) => {
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

		switch (syncMessage.type) {
			case 'arrive': {
				const message = syncMessage as ArriveSyncMessage;
				const peer = this.#signalingRoom.getPeer(fromSignalingId);

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

	#onPeerJoin = async (peer: Peer) => {
		peer.on('data', (data) => {
			this.#receive(peer.getId(), new Uint8Array(data as ArrayBufferLike));
		});
	};

	#onPeerConnect = async (peer: Peer) => {
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
	};

	#onPeerLeave = (signalingId: string) => {
		this.#signalingIdToPeerId.delete(signalingId);
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
