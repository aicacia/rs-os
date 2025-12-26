import { WebRTCClientAdapter } from '../automerge/WebRTCClientAdapter';
import { websocket } from './websocket.svelte';

export const webRTCClientAdapter = new WebRTCClientAdapter(websocket);
