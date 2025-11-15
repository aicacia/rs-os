<script lang="ts" module>
	import {
		ClientUpsertRequestFromJSON,
		type ClientUpsertRequest
	} from '$lib/common/openapi/oidc/models/ClientUpsertRequest';

	export type ClientInfo = ClientUpsertRequest;
	export const ClientInfoFromJSON = ClientUpsertRequestFromJSON;

	function getClientDiff(client: Client, clientInfo: ClientInfo): Partial<ClientInfo> | false {
		const diff: Partial<ClientInfo> = {};
		let changed = false;

		for (const key of Object.keys(clientInfo) as (keyof ClientInfo)[]) {
			const a = client[key];
			const b = clientInfo[key];

			const aIsArray = Array.isArray(a);
			const bIsArray = Array.isArray(b);

			if (aIsArray !== bIsArray) {
				changed = true;
				diff[key] = a as any;
				continue;
			}

			if (aIsArray && bIsArray) {
				const arrA = a as unknown as string[];
				const arrB = b as unknown as string[];

				const arrChanged = arrA.length !== arrB.length || arrA.some((v, i) => v !== arrB[i]);

				if (arrChanged) {
					changed = true;
					diff[key] = a as any;
				}

				continue;
			}

			if (a !== b) {
				changed = true;
				diff[key] = a as any;
			}
		}

		if (!changed) {
			return false;
		}

		return diff;
	}
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { clientApi } from '$lib/common/openapi';
	import { LoaderCircle } from '@lucide/svelte';
	import type { Client } from '$lib/common/openapi/oidc/models/Client';
	import ClientComponent from './_Client.svelte';
	import { handleError } from '$lib/common/errors';
	import ClientNew from './_ClientNew.svelte';
	import ClientUpdated from './_ClientUpdated.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data } = $props();

	let clientInfoOrUrlOrId = $derived(page.url.searchParams.get('client_id'));
	let responseType = $derived(page.url.searchParams.get('response_type'));
	let scope = $derived(page.url.searchParams.get('scope'));
	let redirectUri = $derived(page.url.searchParams.get('redirect_uri'));
	let urlState = $derived(page.url.searchParams.get('state'));
	let nonce = $derived(page.url.searchParams.get('nonce'));

	let clientUrl = $derived.by(() => {
		if (!clientInfoOrUrlOrId) {
			return null;
		}
		try {
			return new URL(clientInfoOrUrlOrId);
		} catch (e) {
			return null;
		}
	});

	let clientInfo = $state<ClientInfo | null>(null);
	let clientId = $state<string | null>(null);

	$effect(() => {
		if (clientUrl) {
			fetch(clientUrl)
				.then(async (response) => {
					if (!response.ok) {
						console.error(`failed to load client url ${clientUrl}: ${await response.text()}`);
						return;
					}

					clientInfo = ClientInfoFromJSON(await response.json());
					clientId = clientInfo.clientId;
				})
				.catch((e) => {
					console.error('failed to load client url', e);
				});
		} else if (clientInfoOrUrlOrId) {
			try {
				clientInfo = ClientInfoFromJSON(JSON.parse(clientInfoOrUrlOrId));
			} catch (e) {
				console.error('client_id is not valid json', e);
			}
			if (clientInfo) {
				clientId = clientInfo.clientId;
				return;
			}
			clientId = clientInfoOrUrlOrId;
		}
	});

	let clientDiff = $state<Partial<ClientUpsertRequest> | false>(false);
	let clientPromise = $state(new Promise<Client | null>(() => {}));

	$effect(() => {
		if (!clientId) {
			return;
		}
		clientPromise = clientApi
			.clientByClientId({ clientId })
			.catch((_e) => {
				return null;
			})
			.then(async (client) => {
				if (client && clientInfo) {
					clientDiff = getClientDiff(client, clientInfo);
				}
				return client;
			});
	});

	async function onAllowClient() {}
	async function onDisallowClient() {
		const client = await clientPromise;
		if (!client || !redirectUri) {
			return;
		}
		const url = new URL(redirectUri);
		url.search = page.url.search;
		url.searchParams.set('error', 'access_denied');
		url.searchParams.set(
			'error_description',
			'The resource owner or authorization server denied the request'
		);
		window.location.href = url.toString();
	}
	async function onAcceptClient() {
		if (!clientInfo) {
			return;
		}
		try {
			clientPromise = clientApi.clientUpsert({ clientUpsertRequest: clientInfo }).then((client) => {
				if (client && clientInfo) {
					clientDiff = getClientDiff(client, clientInfo);
				}
				return client;
			});
			await clientPromise;
		} catch (e) {
			handleError(e);
		}
	}
	async function onRejectClient() {
		await onDisallowClient();
	}
</script>

<div class="overflow-auto">
	<div class="m-8 flex grow flex-col items-center justify-center">
		<div class="card w-md">
			{#await clientPromise}
				<div class="flex flex-col items-center justify-center">
					<LoaderCircle class="h-16 w-16 animate-spin" />
				</div>
			{:then client}
				{#if client}
					{#if clientDiff}
						<ClientUpdated {client} {clientDiff} {onAcceptClient} {onRejectClient} />
					{:else}
						<ClientComponent user={data.user} {client} {onAcceptClient} {onRejectClient} />
					{/if}
				{:else if clientId}
					<ClientNew user={data.user} {clientId} {clientInfo} {onAcceptClient} {onRejectClient} />
				{:else}
					{m.errors_name_client()}: {m.errors_message_not_found()}
				{/if}
			{/await}
		</div>
	</div>
</div>
