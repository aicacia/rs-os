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

export interface WebRTCClientAdapterOptions {
	pendingArriveTtlMs?: number;
	pendingArriveCleanupIntervalMs?: number;
}

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
	#peerDataHandlers: Map<string, { peer: Peer; handler: (data: unknown) => void }> = new Map();
	#pendingPeerConnects: Set<string> = new Set();
	#pendingArrives: Array<{
		fromSignalingId: string;
		message: ArriveSyncMessage;
		addedAt: number;
	}> = [];
	#readyWaiters: Array<{ resolve: () => void; reject: (reason?: unknown) => void }> = [];
	#connectWaiters: Array<{ resolve: () => void; reject: (reason?: unknown) => void }> = [];
	#pendingArriveTtlMs: number;
	#pendingArriveCleanupInterval?: ReturnType<typeof setInterval>;

	#signalingRoom: SignalingRoom;
	#signalingIdToPeerId: Map<string, string> = new Map();

	constructor(signalingRoom: SignalingRoom, options: WebRTCClientAdapterOptions = {}) {
		super();

		signalingRoom.on('join', this.#onPeerJoin);
		signalingRoom.on('connect', this.#onPeerConnect);
		signalingRoom.on('leave', this.#onPeerLeave);
		signalingRoom.getPeers().forEach(this.#onPeerJoin);
		this.#signalingRoom = signalingRoom;

		this.#pendingArriveTtlMs = options.pendingArriveTtlMs ?? 60_000;
		const cleanupEvery = options.pendingArriveCleanupIntervalMs ?? this.#pendingArriveTtlMs;
		if (cleanupEvery > 0) {
			this.#pendingArriveCleanupInterval = setInterval(() => {
				this.#prunePendingArrives();
			}, cleanupEvery);
		}
	}

	isReady() {
		return this.#ready;
	}

	whenReady() {
		if (this.#ready) {
			return Promise.resolve();
		}
		return new Promise<void>((resolve, reject) => {
			this.#readyWaiters.push({ resolve, reject });
		});
	}

	isConnected() {
		return this.#connected;
	}

	whenConnected() {
		if (this.#connected) {
			return Promise.resolve();
		}
		return new Promise<void>((resolve, reject) => {
			this.#connectWaiters.push({ resolve, reject });
		});
	}

	async connect(peerId: PeerId, peerMetadata?: PeerMetadata) {
		this.peerId = peerId;
		this.peerMetadata = peerMetadata;

		this.#forceConnected();
		this.#prunePendingArrives();
		if (this.#pendingArrives.length > 0) {
			const pending = this.#pendingArrives.slice();
			this.#pendingArrives = [];
			pending.forEach(({ fromSignalingId, message }) => {
				this.#handleArrive(fromSignalingId, message);
			});
		}
		if (this.#pendingPeerConnects.size > 0) {
			this.#pendingPeerConnects.forEach((signalingId) => {
				const peer = this.#signalingRoom.getPeer(signalingId);
				if (peer) {
					this.#sendArrive(peer);
				}
			});
			this.#pendingPeerConnects.clear();
		}
	}

	disconnect() {
		this.#signalingRoom.off('join', this.#onPeerJoin);
		this.#signalingRoom.off('connect', this.#onPeerConnect);
		this.#signalingRoom.off('leave', this.#onPeerLeave);
		this.#peerDataHandlers.forEach(({ handler, peer }, signalingId) => {
			const currentPeer = this.#signalingRoom.getPeer(signalingId) ?? peer;
			currentPeer.off('data', handler);
		});
		this.#peerDataHandlers.clear();
		this.#signalingRoom.close();
		this.#signalingIdToPeerId.clear();
		this.#ready = false;
		this.#connected = false;
		if (this.#pendingArriveCleanupInterval) {
			clearInterval(this.#pendingArriveCleanupInterval);
			this.#pendingArriveCleanupInterval = undefined;
		}
		const disconnectError = new Error('WebRTCClientAdapter disconnected');
		this.#readyWaiters.forEach(({ reject }) => reject(disconnectError));
		this.#connectWaiters.forEach(({ reject }) => reject(disconnectError));
		this.#readyWaiters = [];
		this.#connectWaiters = [];
		this.#pendingArrives = [];
		this.emit('close');
	}

	async send(message: Message) {
		if (!this.peerId) {
			throw new Error('send called before peerId is set');
		}
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
				if (!this.peerId || !this.peerMetadata) {
					this.#prunePendingArrives();
					this.#pendingArrives.push({ fromSignalingId, message, addedAt: Date.now() });
					break;
				}
				this.#handleArrive(fromSignalingId, message);
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
			this.#readyWaiters.forEach(({ resolve }) => resolve());
			this.#readyWaiters = [];
			this.#emitter.emit('ready');
		}
	};

	#forceConnected = () => {
		if (!this.#connected) {
			this.#connected = true;
			this.#connectWaiters.forEach(({ resolve }) => resolve());
			this.#connectWaiters = [];
			this.#emitter.emit('connect');
		}
	};

	#onPeerJoin = async (peer: Peer) => {
		const signalingId = peer.getId();
		const existing = this.#peerDataHandlers.get(signalingId);
		if (existing) {
			existing.peer.off('data', existing.handler);
		}
		const handler = (data: unknown) => {
			this.#receive(signalingId, new Uint8Array(data as ArrayBufferLike));
		};
		this.#peerDataHandlers.set(signalingId, { peer, handler });
		peer.on('data', handler);
	};

	#onPeerConnect = async (peer: Peer) => {
		if (!this.peerId) {
			this.#pendingPeerConnects.add(peer.getId());
			return;
		}
		if (!this.#connected) {
			this.#forceConnected();
		}
		await this.whenConnected();
		this.#sendArrive(peer);
	};

	#onPeerLeave = (signalingId: string) => {
		const entry = this.#peerDataHandlers.get(signalingId);
		if (entry) {
			const peer = this.#signalingRoom.getPeer(signalingId) ?? entry.peer;
			peer.off('data', entry.handler);
			this.#peerDataHandlers.delete(signalingId);
		}
		this.#pendingPeerConnects.delete(signalingId);
		this.#signalingIdToPeerId.delete(signalingId);
		this.#pendingArrives = this.#pendingArrives.filter(
			(entry) => entry.fromSignalingId !== signalingId
		);
	};

	#prunePendingArrives(now = Date.now()) {
		if (this.#pendingArriveTtlMs <= 0) {
			return;
		}
		this.#pendingArrives = this.#pendingArrives.filter(
			(entry) => now - entry.addedAt < this.#pendingArriveTtlMs
		);
	}

	#handleArrive(fromSignalingId: string, message: ArriveSyncMessage) {
		const peer = this.#signalingRoom.getPeer(fromSignalingId);
		if (!peer || !this.peerId || !this.peerMetadata) {
			return;
		}
		peer.send(
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
	}

	#sendArrive(peer: Peer) {
		if (!this.peerId) {
			return;
		}
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
	}
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
