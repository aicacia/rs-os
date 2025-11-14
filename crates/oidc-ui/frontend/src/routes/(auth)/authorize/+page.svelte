<script lang="ts">
	import { page } from '$app/state';
	import { clientApi, oidcApi } from '$lib/common/openapi';
	import { LoaderCircle } from '@lucide/svelte';
	import { ClientUpsertRequestFromJSON } from '$lib/common/openapi/oidc/models/ClientUpsertRequest';
	import Client from './_Client.svelte';
	import { ResponseError } from '$lib/common/openapi/oidc';
	import { hasPermission } from '$lib/common/state/currentUser.svelte';
	import { CLIENT_CREATE } from '$lib/common/permissions';
	import { goto } from '$app/navigation';

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

	const userClientAllowed = $derived.by(async () => {
		if (!clientId) {
			return null;
		}
		return await clientApi.clientUserAllowed({ clientId });
	});

	const clientOptionPromise = $derived.by(async () => {
		if (!clientId) {
			return null;
		}
		return await clientApi.clientByClientId({ clientId });
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
		if (!hasPermission(data.user, CLIENT_CREATE)) {
			return null;
		}
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
		try {
			const clientOption = await clientOptionPromise;

			if (clientOption && (await userClientAllowed)) {
				console.log('creating authorization code and redirecting to redirect uri');
				return null; // TODO: create auth
			}

			if (!clientUpsertRequestPromise) {
				return clientOption;
			}

			const clientUpsertRequest = await clientUpsertRequestPromise;
			if (clientUpsertRequest) {
				return await clientApi.clientUpsert({ clientUpsertRequest });
			}
		} catch (e) {
			if (e instanceof ResponseError) {
				console.log(await e.response.json());
			} else {
				console.error(e);
			}
		}
	});
</script>

<div class="flex grow flex-col items-center justify-center">
	<div class="card w-lg">
		{#await clientPromise}
			<div class="flex flex-col items-center justify-center">
				<LoaderCircle class="h-16 w-16 animate-spin" />
			</div>
		{:then client}
			{#if client}
				<Client user={data.user} {client} />
			{:else}
				<p>No client found.</p>
			{/if}
		{/await}
	</div>
</div>
