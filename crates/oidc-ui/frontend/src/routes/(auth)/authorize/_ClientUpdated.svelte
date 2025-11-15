<script lang="ts" module>
	import type { Client } from '$lib/common/openapi/oidc';
	import { m } from '$lib/paraglide/messages';

	export interface ClientUpdatedProps {
		client: Client;
		clientDiff: Partial<ClientInfo>;
		onAcceptClient: () => Promise<void>;
		onRejectClient: () => Promise<void>;
	}
</script>

<script lang="ts">
	import type { ClientInfo } from './+page.svelte';
	import ClientInfoLogo from './_ClientInfoLogo.svelte';

	let { client, clientDiff, onAcceptClient, onRejectClient }: ClientUpdatedProps = $props();

	let loading = $state(false);

	async function onAcceptClientInternal() {
		loading = true;
		try {
			await onAcceptClient();
		} finally {
			loading = false;
		}
	}
	async function onRejectClientInternal() {
		loading = true;
		try {
			await onRejectClient();
		} finally {
			loading = false;
		}
	}
</script>

<div class="mb-4 flex items-center gap-4">
	<ClientInfoLogo {client} />

	<div>
		<h1 class="text-xl font-semibold">{client.name}</h1>

		{#if clientDiff.clientUri}
			<a href={clientDiff.clientUri} target="_blank">
				{clientDiff.clientUri}
			</a>
		{/if}

		<p class="text-sm text-gray-400">{clientDiff.applicationType}</p>
	</div>
</div>

<hr />

<p>
	This client is requesting updates to be added to your OIDC provider. Review the details before
	approving.
</p>

{#if clientDiff.scopes?.length}
	<section>
		<h5>Scopes</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.scopes as s}
				<li>{s}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.redirectUris?.length}
	<section>
		<h5>Redirect URIs</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.redirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.postLogoutRedirectUris?.length}
	<section>
		<h5>Post-logout Redirect URIs</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.postLogoutRedirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.grantTypes?.length}
	<section>
		<h5>Grant Types</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.grantTypes as t}
				<li>{t}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.responseTypes?.length}
	<section>
		<h5>Response Types</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.responseTypes as r}
				<li>{r}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.audience?.length}
	<section>
		<h5>Audience</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each clientDiff.audience as a}
				<li>{a}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if clientDiff.accessTokenExpiresInSeconds || clientDiff.idTokenExpiresInSeconds || clientDiff.refreshExpiresInSeconds}
	<section>
		<h5>Token Expiry</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#if clientDiff.accessTokenExpiresInSeconds}
				<li><strong>Access Token Expires:</strong> {clientDiff.accessTokenExpiresInSeconds}s</li>
			{/if}
			{#if clientDiff.idTokenExpiresInSeconds}
				<li><strong>ID Token Expires:</strong> {clientDiff.idTokenExpiresInSeconds}s</li>
			{/if}
			{#if clientDiff.refreshExpiresInSeconds}
				<li><strong>Refresh Token Expires:</strong> {clientDiff.refreshExpiresInSeconds}s</li>
			{/if}
		</ul>
	</section>
{/if}

{#if clientDiff.policyUri || clientDiff.termsOfServiceUri}
	<section>
		<h5>Legal</h5>
		<ul class="space-y-1 text-sm">
			{#if clientDiff.policyUri}
				<li>
					<a href={clientDiff.policyUri} target="_blank"> Privacy Policy </a>
				</li>
			{/if}
			{#if clientDiff.termsOfServiceUri}
				<li>
					<a href={clientDiff.termsOfServiceUri} target="_blank"> Terms of Service </a>
				</li>
			{/if}
		</ul>
	</section>
{/if}

<section>
	<div class="mt-4 flex flex-row justify-center gap-4">
		<button class="btn secondary" disabled={loading} onclick={onRejectClientInternal}
			>{m.client_reject()}</button
		>
		<button class="btn danger" disabled={loading} onclick={onAcceptClientInternal}
			>{m.client_accept()}</button
		>
	</div>
</section>
