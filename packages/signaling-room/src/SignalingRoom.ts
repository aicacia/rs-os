import { KeepAliveWebSocket } from '@aicacia/keepalivewebsocket';
import { Peer, type SignalMessage } from '@aicacia/peer';
import { EventEmitter } from 'eventemitter3';

function isSignalMessage(message: unknown): message is SignalMessage {
	return typeof message === 'object' && message !== null && 'type' in message;
}

type SignalingId = string;

type ClientMessage =
	| {
			type: 'send';
			to: SignalingId;
			payload: unknown;
	  }
	| {
			type: 'broadcast';
			payload: unknown;
	  };

type ServerMessage =
	| {
			type: 'welcome';
			id: SignalingId;
			peers: SignalingId[];
	  }
	| {
			type: 'join';
			from: SignalingId;
			payload: unknown;
	  }
	| {
			type: 'leave';
			from: SignalingId;
	  }
	| {
			type: 'message';
			from: SignalingId;
			payload: unknown;
	  };

export interface SignalingRoomEvents {
	join(this: SignalingRoom, peer: Peer): void;
	connect(this: SignalingRoom, peer: Peer): void;
	leave(this: SignalingRoom, signalingId: SignalingId): void;
	error(this: SignalingRoom, error: Error): void;
}

export class SignalingRoom extends EventEmitter<SignalingRoomEvents> {
	#keepAliveWebSocket: KeepAliveWebSocket;
	#remotePeers = new Map<SignalingId, Peer>();

	constructor(keepAliveWebSocket: KeepAliveWebSocket) {
		super();
		keepAliveWebSocket.on('message', this.#onWebSocketMessage);
		this.#keepAliveWebSocket = keepAliveWebSocket;
	}

	getPeer(signalingId: SignalingId) {
		return this.#remotePeers.get(signalingId);
	}

	getPeers() {
		return Array.from(this.#remotePeers.values());
	}

	close() {
		this.#keepAliveWebSocket.off('message', this.#onWebSocketMessage);
		const errors: unknown[] = [];
		for (const peer of this.#remotePeers.values()) {
			try {
				peer.close();
			} catch (e) {
				errors.push(e);
			}
		}
		if (errors.length > 0) {
			if (errors.length === 1) {
				throw errors[0];
			} else {
				throw new AggregateError(errors, 'Multiple errors occurred while closing peers');
			}
		}
		this.#remotePeers.clear();
		return this;
	}

	async #getOrCreatePeer(signalingId: SignalingId, isInitiator: boolean) {
		let peer = this.#remotePeers.get(signalingId);

		if (peer) {
			return peer;
		}

		peer = new Peer({ id: signalingId });
		this.#remotePeers.set(signalingId, peer);

		peer.on('signal', (signalPayload: unknown) => {
			this.#keepAliveWebSocket.send(
				JSON.stringify({
					type: 'send',
					to: signalingId,
					payload: signalPayload
				} as ClientMessage)
			);
		});
		peer.on('close', () => {
			this.emit('leave', signalingId);
			this.#remotePeers.delete(signalingId);
		});
		peer.on('connect', () => {
			this.emit('connect', peer);
		});

		this.emit('join', peer);

		if (isInitiator) {
			await peer.init();
		}

		return peer;
	}

	#onWebSocketMessage = async (
		data: string | ArrayBufferLike | Blob | ArrayBufferView<ArrayBufferLike>
	) => {
		try {
			const signalingMessage = JSON.parse(data as string) as ServerMessage;

			switch (signalingMessage.type) {
				case 'welcome': {
					await Promise.all(
						signalingMessage.peers.map((signalingId) => this.#getOrCreatePeer(signalingId, true))
					);
					break;
				}
				case 'join': {
					// ignore as the new peer will be the initiator when it receives the 'welcome' message
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
					if (!isSignalMessage(signalingMessage.payload)) {
						break;
					}
					const peer = await this.#getOrCreatePeer(signalingMessage.from, false);
					await peer.signal(signalingMessage.payload);
					break;
				}
			}
		} catch (e) {
			this.emit('error', e as Error);
		}
	};
}
