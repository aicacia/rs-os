import type { DocHandle, DocumentId } from '@automerge/automerge-repo';
import { localStorageState } from '../util/localStorageState.svelte';
import { repo } from '../repo';

export interface TestDocument {
	count: number;
}

const testDocumentId = localStorageState<DocumentId | null>(
	'test-document-id',
	null
);
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