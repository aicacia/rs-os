import { Repo } from "@automerge/automerge-repo"
import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb"
import { webSocketClientAdapter } from "./state/webSocketClientAdapter.svelte"

export const repo = new Repo({
  network: [
    webSocketClientAdapter,
  ],
  storage: new IndexedDBStorageAdapter(),
})
