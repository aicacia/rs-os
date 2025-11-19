<script lang="ts" module>
	const RESPONSE_TYPES = Object.values(ResponseType);
	const RESPONSE_MODES = Object.values(ResponseMode);
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { type AuthorizeRequest, ResponseMode, ResponseType } from '$lib/common/openapi/oidc';
	import Authorize from './_Authorize.svelte';
	import { ClientInfoFromJSON, rejectAuthorizeRequest, type ClientInfo } from './_utils';

	let { data } = $props();

	let urlClientId = $derived(page.url.searchParams.get('client_id'));
	let urlResponseType = $derived(page.url.searchParams.get('response_type'));
	let urlResponseMode = $derived(page.url.searchParams.get('response_mode'));
	let urlScope = $derived(page.url.searchParams.get('scope'));
	let urlRedirectUri = $derived(page.url.searchParams.get('redirect_uri'));
	let urlState = $derived(page.url.searchParams.get('state'));
	let urlNonce = $derived(page.url.searchParams.get('nonce'));
	let urlRegistration = $derived(page.url.searchParams.get('registration'));

	let clientIdInfo = $state<ClientInfo | null>(null);

	$effect(() => {
		if (urlRegistration) {
			try {
				clientIdInfo = ClientInfoFromJSON(JSON.parse(urlRegistration));
			} catch (_e) {}
		}
	});

	let clientIdError = $state<string>();
	let responseTypeError = $state<string>();
	let responseModeError = $state<string>();
	let scopeError = $state<string>();
	let redirectUriError = $state<string>();

	let authorizeRequest = $state<AuthorizeRequest>();

	$effect(() => {
		if (!urlClientId) {
			clientIdError = 'Client ID is required';
		} else {
			clientIdError = undefined;
		}
		if (!urlResponseType) {
			responseTypeError = 'Response Type is required';
		} else {
			responseTypeError = undefined;
		}
		if (!urlResponseType) {
			responseTypeError = 'Response Type is required';
		} else if (!RESPONSE_TYPES.includes(urlResponseType as never)) {
			responseTypeError = `${urlResponseType} is not one of ${RESPONSE_TYPES.join(',')}`;
		} else {
			responseTypeError = undefined;
		}
		if (!urlResponseMode) {
			responseModeError = 'Response Mode is required';
		} else if (!RESPONSE_MODES.includes(urlResponseMode as never)) {
			responseModeError = `${urlResponseMode} is not one of ${RESPONSE_MODES.join(',')}`;
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
			clientId: urlClientId!,
			responseType: urlResponseType as ResponseType,
			responseMode: urlResponseMode as ResponseMode,
			redirectUri: urlRedirectUri!,
			scope: urlScope!,
			state: urlState,
			nonce: urlNonce
		};
	});

	function onReject() {
		if (!urlRedirectUri) {
			window.close();
			return;
		}
		rejectAuthorizeRequest(
			{
				redirectUri: urlRedirectUri,
				state: urlState,
				nonce: urlNonce
			},
			'invalid_request',
			'The request is missing a required parameter, includes an invalid parameter value, includes a parameter more than once, or is otherwise malformed.'
		);
	}
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
					<div>
						<div class="mt-4 flex flex-row justify-center gap-4">
							<button class="btn secondary" onclick={onReject}>Deny</button>
						</div>
					</div>
				</section>
			{/if}
		</div>
	</div>
</div>
