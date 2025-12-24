<script lang="ts">
	import { onMount } from 'svelte';
	import { getUserManager } from '$lib/common/state/user.svelte';
	import { websocket } from '$lib/common/state/websocket.svelte';

	let { data } = $props();
	
	async function onSignOut() {
		getUserManager().signoutRedirect();
	}

	onMount(() => {
		websocket.on('open', () => {
			console.log('WebSocket connection opened');
		});
	});
</script>

<div class="flex flex-col grow items-center justify-center">
	<h1>Welcome, {data.user.profile.preferred_username}!</h1>

	<div class="flex flex-row justify-center">
		<button class="btn danger" onclick={onSignOut}>Sign out</button>
	</div>
</div>