import { Repo } from '@automerge/automerge-repo';
import { IndexedDBStorageAdapter } from '@automerge/automerge-repo-storage-indexeddb';
import { webRTCClientAdapter } from './state/webRTCClientAdapter.svelte';

export const repo = new Repo({
	network: [webRTCClientAdapter],
	storage: new IndexedDBStorageAdapter()
});
