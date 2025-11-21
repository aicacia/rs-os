<script lang="ts">
	import { getUserManager } from '$lib/common/state/user.svelte';

	let { data } = $props();
	
	async function onSignOut() {
		getUserManager().signoutRedirect();
	}
</script>

<div class="flex flex-col grow items-center justify-center">
	<code class="flex flex-col w-lg h-96 overflow-auto">
		<pre>{JSON.stringify(data.user, null, 2)}</pre>
	</code>

	<div class="flex flex-row justify-center">
		<button class="btn danger" onclick={onSignOut}>Sign out</button>
	</div>
</div>