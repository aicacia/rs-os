<script lang="ts">
	import { getTestDocumentHandle, type TestDocument } from '$lib/common/state/test.svelte';
	import { getUserManager } from '$lib/common/state/user.svelte';
	import type { DocHandle } from '@automerge/automerge-repo';

	let { data } = $props();
	
	async function onSignOut() {
		getUserManager().signoutRedirect();
	}

	let handle: DocHandle<TestDocument> | null = $state(null);
	let count = $state(0);
	$effect(() => {
		getTestDocumentHandle().then((doc) => {
			handle = doc;
			count = doc.doc().count;
		});
	})

	function onIncrement() {
		if (handle) {
			handle.change((doc) => {
				doc.count += 1;
				count = doc.count;
			});
		}
	}
	function onDecrement() {
		if (handle) {
			handle.change((doc) => {
				doc.count -= 1;
				count = doc.count;
			});
		}
	}
</script>

<div class="flex flex-col grow items-center justify-center">
	<h1>Welcome, {data.user.profile.preferred_username}!</h1>

	{#if handle}
		<div class="flex flex-row items-center justify-center">
			<button class="btn danger" onclick={onDecrement}>-</button>
			<p>Your test document count is: {count}</p>
			<button class="btn success" onclick={onIncrement}>+</button>
		</div>
	{/if}

	<div class="flex flex-row justify-center">
		<button class="btn danger" onclick={onSignOut}>Sign out</button>
	</div>
</div>