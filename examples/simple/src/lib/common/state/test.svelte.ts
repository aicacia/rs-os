import type { DocHandle, DocumentId } from '@automerge/automerge-repo';
import { createStorage } from '@aicacia/svelte-headless';
import { repo } from '../repo';

export interface TestDocument {
	count: number;
}

const testDocumentId = createStorage<DocumentId | null>('test-document-id', null);
const testDocumentHandle = $derived.by(initTestDocumentHandle);

async function initTestDocumentHandle() {
	let testDocumentHandle: DocHandle<TestDocument>;

	if (testDocumentId.value == null) {
		testDocumentHandle = repo.create<TestDocument>({ count: 0 });
		await testDocumentHandle.whenReady();
		testDocumentId.value = testDocumentHandle.documentId;
	} else {
		testDocumentHandle = await repo.find(testDocumentId.value);
		await testDocumentHandle.whenReady();
	}

	return testDocumentHandle;
}

export async function getTestDocumentHandle(): Promise<DocHandle<TestDocument>> {
	return testDocumentHandle;
}
