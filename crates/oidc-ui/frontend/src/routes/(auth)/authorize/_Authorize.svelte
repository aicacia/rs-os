<script lang="ts" module>
	import type { AuthorizeRequest, User } from '$lib/common/openapi/oidc';

	export interface AuthorizeProps {
		user: User;
		clientIdInfo: ClientInfo | null;
		authorizeRequest: AuthorizeRequest;
	}
</script>

<script lang="ts">
	import { oidcApi, clientApi } from '$lib/common/openapi';
	import type { Client } from '$lib/common/openapi/oidc/models/Client';
	import {
		getClientDiff,
		rejectAuthorizeRequest,
		resolveAuthorizeRequest,
		type ClientInfo
	} from './_utils';
	import { LoaderCircle } from '@lucide/svelte';
	import AddClient from './_ClientUpdates.svelte';
	import AuthorizeClient from './_AuthorizeClient.svelte';
	import { handleError } from '$lib/common/errors';

	let { user, clientIdInfo, authorizeRequest }: AuthorizeProps = $props();

	let clientDiff = $state<Partial<ClientInfo> | false>(false);
	let client = $state<Client | null>(null);

	let loadingClient = $state(true);
	$effect(() => {
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
			});
	});

	$effect(() => {
		if (clientIdInfo) {
			if (client) {
				clientDiff = getClientDiff(client, clientIdInfo);
			}
		} else {
			clientDiff = false;
		}
	});

	let loadingUserAllowed = $state(true);
	$effect(() => {
		if (clientDiff) {
			return;
		}
		loadingUserAllowed = true;
		clientApi
			.clientUserAllowed({ clientId: authorizeRequest.clientId })
			.then(onAuthorize)
			.catch((_e) => {
				return null;
			})
			.finally(() => {
				loadingUserAllowed = false;
			});
	});

	async function onAuthorize() {
		try {
			await resolveAuthorizeRequest(authorizeRequest);
		} catch (e) {
			handleError(e);
		}
	}
	async function onAllow() {
		try {
			await clientApi.clientUserApprove({ clientId: authorizeRequest.clientId });
			await onAuthorize();
		} catch (e) {
			handleError(e);
		}
	}
	async function onDeny() {
		rejectAuthorizeRequest(
			authorizeRequest,
			'access_denied',
			'Access to the requested resource was denied.'
		);
	}
	async function onAcceptClientUpdates(clientRegisterRequest: ClientInfo) {
		try {
			loadingClient = true;
			client = await oidcApi.registerClient({ clientRegisterRequest });
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

	let disabled = $derived(loadingClient || loadingUserAllowed);
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
				logoUri: client.logoUri,
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
