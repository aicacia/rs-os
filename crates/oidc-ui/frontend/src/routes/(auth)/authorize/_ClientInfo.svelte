<script lang="ts" module>
	import type { User } from '$lib/common/openapi/oidc';
	import { CLIENT_CREATE } from '$lib/common/permissions';
	import { hasPermission } from '$lib/common/state/currentUser.svelte';

	export interface ClientInfoProps {
		user: User;
		client: ClientInfo;
		onAcceptClient: () => Promise<void>;
		onRejectClient: () => Promise<void>;
	}
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	import type { ClientInfo } from './+page.svelte';
	import ClientInfoLogo from './_ClientInfoLogo.svelte';

	let { user, client, onAcceptClient, onRejectClient }: ClientInfoProps = $props();

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

		{#if client.clientUri}
			<a href={client.clientUri} target="_blank">
				{client.clientUri}
			</a>
		{/if}

		<p class="text-sm text-gray-400">{client.applicationType}</p>
	</div>
</div>

<hr />

<p>
	This client is requesting to be added to your OIDC provider. Review the details before approving.
</p>

<section>
	<h5>Scopes</h5>
	{#if client.scopes?.length}
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.scopes as s}
				<li>{s}</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-gray-500">None</p>
	{/if}
</section>

<section>
	<h5>Redirect URIs</h5>
	{#if client.redirectUris?.length}
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.redirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-gray-500">None</p>
	{/if}
</section>

<section>
	<h5>Post-logout Redirect URIs</h5>
	{#if client.postLogoutRedirectUris?.length}
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.postLogoutRedirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm text-gray-500">None</p>
	{/if}
</section>

<section>
	<h5>Grant Types</h5>
	<ul class="list-inside list-disc space-y-1 text-sm">
		{#each client.grantTypes as t}
			<li>{t}</li>
		{/each}
	</ul>
</section>

<section>
	<h5>Response Types</h5>
	<ul class="list-inside list-disc space-y-1 text-sm">
		{#each client.responseTypes as r}
			<li>{r}</li>
		{/each}
	</ul>
</section>

{#if client.audience?.length}
	<section>
		<h5>Audience</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.audience as a}
				<li>{a}</li>
			{/each}
		</ul>
	</section>
{/if}

<section>
	<h5>Token Expiry</h5>
	<ul class="list-inside list-disc space-y-1 text-sm">
		<li><strong>Access Token Expires:</strong> {client.accessTokenExpiresInSeconds}s</li>
		<li><strong>ID Token Expires:</strong> {client.idTokenExpiresInSeconds}s</li>
		<li><strong>Refresh Token Expires:</strong> {client.refreshExpiresInSeconds}s</li>
	</ul>
</section>

{#if client.policyUri || client.termsOfServiceUri}
	<section>
		<h5>Legal</h5>
		<ul class="space-y-1 text-sm">
			{#if client.policyUri}
				<li>
					<a href={client.policyUri} target="_blank"> Privacy Policy </a>
				</li>
			{/if}
			{#if client.termsOfServiceUri}
				<li>
					<a href={client.termsOfServiceUri} target="_blank"> Terms of Service </a>
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
		{#if hasPermission(user, CLIENT_CREATE)}
			<button class="btn danger" disabled={loading} onclick={onAcceptClientInternal}
				>{m.client_accept()}</button
			>
		{/if}
	</div>
</section>
