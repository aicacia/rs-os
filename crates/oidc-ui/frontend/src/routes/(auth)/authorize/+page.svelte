<script lang="ts" module>
	const RESPONSE_MODES = Object.values(ResponseMode);
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { handleError } from '$lib/common/errors';
	import { type AuthorizeRequest, ResponseMode } from '$lib/common/openapi/oidc';
	import Authorize from './_Authorize.svelte';
	import { ClientInfoFromJSON, type ClientInfo } from './_utils';

	let { data } = $props();

	let urlClientId = $derived(page.url.searchParams.get('client_id'));
	let urlResponseType = $derived(page.url.searchParams.get('response_type'));
	let urlResponseMode = $derived(page.url.searchParams.get('response_mode'));
	let urlScope = $derived(page.url.searchParams.get('scope'));
	let urlRedirectUri = $derived(page.url.searchParams.get('redirect_uri'));
	let urlState = $derived(page.url.searchParams.get('state'));
	let urlNonce = $derived(page.url.searchParams.get('nonce'));

	let clientUrl = $derived.by(() => {
		if (!urlClientId) {
			return null;
		}
		try {
			return new URL(urlClientId);
		} catch (e) {
			return null;
		}
	});

	let clientIdInfo = $state<ClientInfo | null>(null);
	let clientId = $state<string | null>(null);

	$effect(() => {
		if (clientUrl) {
			fetch(clientUrl)
				.then(async (response) => {
					if (!response.ok) {
						console.error(`failed to load client url ${clientUrl}: ${await response.text()}`);
						return;
					}

					clientIdInfo = ClientInfoFromJSON(await response.json());
					clientId = clientIdInfo.clientId;
				})
				.catch(handleError);
		} else if (urlClientId) {
			try {
				clientIdInfo = ClientInfoFromJSON(JSON.parse(urlClientId));
			} catch (_e) {}
			if (clientIdInfo) {
				clientId = clientIdInfo.clientId;
				return;
			}
			clientId = urlClientId;
		}
	});

	let clientIdError = $state<string>();
	let responseTypeError = $state<string>();
	let responseModeError = $state<string>();
	let scopeError = $state<string>();
	let redirectUriError = $state<string>();

	let authorizeRequest = $state<AuthorizeRequest>();

	$effect(() => {
		if (!clientId) {
			clientIdError = 'Client ID is required';
		} else {
			clientIdError = undefined;
		}
		if (!urlResponseType) {
			responseTypeError = 'Response Type is required';
		} else {
			responseTypeError = undefined;
		}
		if (!urlResponseMode) {
			responseModeError = 'Response Mode is required';
		} else if (!RESPONSE_MODES.includes(urlResponseMode as never)) {
			responseModeError = `Invalid Response Mode: ${urlResponseMode} is not one of ${RESPONSE_MODES.join(',')}`;
		} else {
			responseModeError = undefined;
		}
		if (!urlRedirectUri) {
			redirectUriError = 'Redirect URI is required';
		} else {
			redirectUriError = undefined;
		}
		if (!urlScope) {
			scopeError = 'Scope is required';
		} else {
			scopeError = undefined;
		}
		if (clientIdError || responseTypeError || responseModeError || redirectUriError || scopeError) {
			authorizeRequest = undefined;
			return;
		}
		authorizeRequest = {
			clientId: clientId!,
			responseType: urlResponseType!,
			responseMode: urlResponseMode as ResponseMode,
			redirectUri: urlRedirectUri!,
			scope: urlScope!,
			state: urlState,
			nonce: urlNonce
		};
	});
</script>

<div class="overflow-auto">
	<div class="m-8 flex grow flex-col items-center justify-center">
		<div class="card w-md">
			{#if authorizeRequest}
				<Authorize user={data.user} {clientIdInfo} {authorizeRequest} />
			{:else}
				<section>
					<h5>Invalid Request</h5>
					<ul class="list-inside list-disc space-y-1 text-sm">
						{#if clientIdError}
							<li><strong>Invalid Client ID:</strong> {clientIdError}</li>
						{/if}
						{#if responseTypeError}
							<li><strong>Invalid Response Type:</strong> {responseTypeError}</li>
						{/if}
						{#if responseModeError}
							<li><strong>Invalid Response Mode:</strong> {responseModeError}</li>
						{/if}
						{#if redirectUriError}
							<li><strong>Invalid Redirect URI:</strong> {redirectUriError}</li>
						{/if}
						{#if scopeError}
							<li><strong>Invalid Scope:</strong> {scopeError}</li>
						{/if}
					</ul>
				</section>
			{/if}
		</div>
	</div>
</div>
