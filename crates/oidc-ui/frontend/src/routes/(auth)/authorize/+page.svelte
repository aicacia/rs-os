<script lang="ts">
	import { page } from '$app/state';
	import { clientApi } from '$lib/common/openapi';
	import { ClientUpsertRequestFromJSON } from '$lib/common/openapi/oidc/models/ClientUpsertRequest';
	import Client from './_Client.svelte';

	let { data } = $props();

	let clientId = $state<string | null>(null);
	let responseType = $state<string | null>(null);
	let scope = $state<string | null>(null);
	let redirectUri = $state<string | null>(null);
	let urlState = $state<string | null>(null);
	let nonce = $state<string | null>(null);

	$effect(() => {
		clientId = page.url.searchParams.get('client_id');
		responseType = page.url.searchParams.get('response_type');
		scope = page.url.searchParams.get('scope');
		redirectUri = page.url.searchParams.get('redirect_uri');
		urlState = page.url.searchParams.get('state');
		nonce = page.url.searchParams.get('nonce');
	});

	const clientUrl = $derived.by(() => {
		try {
			if (clientId) {
				return new URL(clientId);
			}
		} catch (e) {
			console.error('invalid client url', e);
		}
		return null;
	});

	const clientUpsertRequestPromise = $derived.by(async () => {
		if (clientUrl) {
			const response = await fetch(clientUrl);

			if (response.ok) {
				return ClientUpsertRequestFromJSON(await response.json());
			} else {
				console.error(`failed to load client url ${clientUrl}: ${await response.text()}`);
			}
		}
		return null;
	});

	const clientPromise = $derived.by(async () => {
		const clientUpsertRequest = await clientUpsertRequestPromise;
		return clientApi.clientUpsert({ clientUpsertRequest });
	});
</script>

<div class="flex grow flex-col items-center justify-center">
	<div class="card w-xl">
		{#await clientPromise}
			Loading...
		{:then client}
			<Client user={data.user} {client} />
		{/await}
	</div>
</div>
