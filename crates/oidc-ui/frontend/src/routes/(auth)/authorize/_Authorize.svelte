<script lang="ts" module>
	import type { AuthorizeRequest, User } from '$lib/common/openapi/oidc';

	export interface AuthorizeProps {
		user: User;
		clientIdInfo: ClientInfo | null;
		authorizeRequest: AuthorizeRequest;
	}
</script>

<script lang="ts">
	import { clientApi } from '$lib/common/openapi';
	import type { Client } from '$lib/common/openapi/oidc/models/Client';
	import { getClientDiff, rejectAuthorizeRequest, type ClientInfo } from './_utils';
	import { LoaderCircle } from '@lucide/svelte';
	import AddClient from './_ClientUpdates.svelte';
	import AuthorizeClient from './_AuthorizeClient.svelte';
	import { handleError } from '$lib/common/errors';

	let { user, clientIdInfo, authorizeRequest }: AuthorizeProps = $props();

	let clientDiff = $state<Partial<ClientInfo> | false>(false);
	let client = $state<Client | null>(null);

	let loadingClient = $state(true);
	let disabled = $state(true);

	$effect(() => {
		disabled = true;
		loadingClient = true;
		clientApi
			.clientByClientId({ clientId: authorizeRequest.clientId })
			.catch((_e) => {
				return null;
			})
			.then((c) => {
				client = c;
			})
			.finally(() => {
				loadingClient = false;
				disabled = false;
			});
	});

	$effect(() => {
		if (client && clientIdInfo) {
			clientDiff = getClientDiff(client, clientIdInfo);
		} else {
			clientDiff = false;
		}
	});

	async function onAllow() {}
	async function onDeny() {
		rejectAuthorizeRequest(
			authorizeRequest,
			'access_denied',
			'Access to the requested resource was denied.'
		);
	}
	async function onAcceptClientUpdates(clientUpsertRequest: ClientInfo) {
		try {
			loadingClient = true;
			client = await clientApi.clientUpsert({ clientUpsertRequest });
		} catch (e) {
			handleError(e);
		} finally {
			loadingClient = false;
		}
	}
	async function onRejectClientUpdates() {
		rejectAuthorizeRequest(
			authorizeRequest,
			'unauthorized_client',
			'The client is not authorized to request an authorization code using this method.'
		);
	}
</script>

{#if loadingClient}
	<div class="flex flex-row items-center justify-center">
		<LoaderCircle class="animate-spin" />
	</div>
{:else if client}
	{#if clientDiff}
		<AddClient
			{user}
			client={{
				name: client.name,
				...clientDiff
			}}
			isNew={false}
			{disabled}
			onAccept={onAcceptClientUpdates}
			onReject={onRejectClientUpdates}
		/>
	{:else}
		<AuthorizeClient {user} {client} {disabled} {onAllow} {onDeny} />
	{/if}
{:else if clientIdInfo}
	<AddClient
		{user}
		client={clientIdInfo}
		{disabled}
		isNew={true}
		onAccept={onAcceptClientUpdates}
		onReject={onRejectClientUpdates}
	/>
{/if}
