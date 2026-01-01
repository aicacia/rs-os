<script lang="ts" module>
	import type { AuthorizeRequest, OpenIdClaims } from '$lib/common/openapi/oidc';

	export interface AuthorizeProps {
		userInfo: OpenIdClaims;
		clientIdInfo: ClientInfo | null;
		authorizeRequest: AuthorizeRequest;
	}
</script>

<script lang="ts">
	import { oidcApi } from '$lib/common/openapi';
	import type { Client } from '$lib/common/openapi/oidc/models/Client';
	import {
		getClientDiff,
		rejectAuthorizeRequest,
		resolveAuthorizeRequest,
		type ClientInfo
	} from './_utils';
	import { m } from '$lib/paraglide/messages';
	import { LoaderCircle } from '@lucide/svelte';
	import AddClient from './_ClientUpdates.svelte';
	import AuthorizeClient from './_AuthorizeClient.svelte';
	import { handleError } from '$lib/common/errors';

	let { userInfo, clientIdInfo, authorizeRequest }: AuthorizeProps = $props();

	let clientDiff = $state<Partial<ClientInfo> | false>(false);
	let client = $state<Client | null>(null);

	let loadingClient = $state(true);
	$effect(() => {
		loadingClient = true;
		oidcApi
			.client({ clientId: authorizeRequest.clientId })
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
		oidcApi
			.isClientAllowedForUser({ clientId: authorizeRequest.clientId })
			.catch((e) => {
				loadingUserAllowed = false;
				throw e;
			})
			.then(onAuthorize);
	});

	let loadingAuthorizeRequest = $state(false);
	async function onAuthorize() {
		loadingAuthorizeRequest = true;
		try {
			await resolveAuthorizeRequest(authorizeRequest);
		} catch (e) {
			handleError(e);
		} finally {
			loadingAuthorizeRequest = false;
		}
	}
	async function onAllow() {
		try {
			await oidcApi.approveClientForUser({ clientId: authorizeRequest.clientId });
			await onAuthorize();
		} catch (e) {
			handleError(e);
		}
	}
	async function onDeny() {
		rejectAuthorizeRequest(authorizeRequest, 'access_denied', m.authorize_access_denied_reason());
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
			m.authorize_unauthorized_client_reason()
		);
	}

	let loading = $derived(loadingClient || loadingUserAllowed || loadingAuthorizeRequest);
	let disabled = $derived(loading);
</script>

{#if loading}
	<div class="flex flex-row items-center justify-center">
		<LoaderCircle class="animate-spin" />
	</div>
{:else if client}
	{#if clientDiff}
		<AddClient
			{userInfo}
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
		<AuthorizeClient {userInfo} {client} {disabled} {onAllow} {onDeny} />
	{/if}
{:else if clientIdInfo}
	<AddClient
		{userInfo}
		client={clientIdInfo}
		{disabled}
		isNew={true}
		onAccept={onAcceptClientUpdates}
		onReject={onRejectClientUpdates}
	/>
{/if}
