<script lang="ts">
	import { getUserManager } from '$lib/common/state/user.svelte';

	let { data } = $props();
	
	async function onSignOut() {
		getUserManager().signoutRedirect();
	}
</script>

<pre>
	{JSON.stringify(data.user, null, 2)}
</pre>

<button class="btn danger" onclick={onSignOut}>Sign out</button>
