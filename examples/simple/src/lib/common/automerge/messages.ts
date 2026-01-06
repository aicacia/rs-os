import type { Message, PeerId, PeerMetadata } from '@automerge/automerge-repo/slim';

export type JoinMessage = {
	type: 'join';
	senderId: PeerId;
	peerMetadata: PeerMetadata;
};

export type PeerMessage = {
	type: 'peer';
	senderId: PeerId;
	peerMetadata: PeerMetadata;
	targetId: PeerId;
};

export type ErrorMessage = {
	type: 'error';
	senderId: PeerId;
	message: string;
	targetId: PeerId;
};

export type FromClientMessage = JoinMessage | Message;

export type FromServerMessage = PeerMessage | ErrorMessage | Message;

export const isJoinMessage = (message: FromClientMessage): message is JoinMessage =>
	message.type === 'join';

export const isPeerMessage = (message: FromServerMessage): message is PeerMessage =>
	message.type === 'peer';

export const isErrorMessage = (message: FromServerMessage): message is ErrorMessage =>
	message.type === 'error';
