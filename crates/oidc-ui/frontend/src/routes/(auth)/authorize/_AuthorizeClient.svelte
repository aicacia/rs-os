<script lang="ts" module>
	import type { ClientInfo } from './_utils';
	import type { User } from '$lib/common/openapi/oidc';

	export interface ClientProps {
		user: User;
		client: ClientInfo;
		disabled?: boolean;
		onAllow: () => Promise<void>;
		onDeny: () => Promise<void>;
	}
</script>

<script lang="ts">
	import Avatar from '../../../lib/common/components/Avatar.svelte';

	let { user, client, disabled, onAllow, onDeny }: ClientProps = $props();

	let loading = $state(false);

	async function onAllowInternal() {
		try {
			loading = true;
			await onAllow();
		} finally {
			loading = false;
		}
	}
	async function onDenyInternal() {
		try {
			loading = true;
			await onDeny();
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex flex-col justify-center">
	<div class="flex flex-col items-center justify-center">
		{#if client.logoUri}
			<Avatar src={client.logoUri} alt={`${client.name}`} />
		{/if}

		<h1 class="m-0 mt-2 text-xl font-semibold">
			{client.name}
		</h1>
	</div>

	<p class="text-center">
		wants to access your account as <strong>{user.username}</strong>
	</p>
</div>

<hr />

<div class="my-4 flex flex-col justify-center">
	<h2 class="text-sm font-medium">Requested permissions</h2>

	{#if client.scopes && client.scopes.length}
		<ul class="list-inside list-disc text-sm">
			{#each client.scopes as scope (scope)}
				<li>{scope}</li>
			{/each}
		</ul>
	{:else}
		<p class="text-sm italic">No scopes requested</p>
	{/if}
</div>

{#if client.policyUri || client.termsOfServiceUri}
	<hr />
	<div class="mt-4 flex flex-row justify-center gap-4 text-xs">
		{#if client.policyUri}
			<a href={client.policyUri} target="_blank">Privacy Policy</a>
		{/if}
		{#if client.termsOfServiceUri}
			<a href={client.termsOfServiceUri} target="_blank">Terms of Service</a>
		{/if}
	</div>
{/if}

<div class="mt-4 flex flex-row justify-center gap-4">
	<button class="btn secondary" disabled={disabled || loading} onclick={onDenyInternal}>Deny</button
	>
	<button class="btn primary" disabled={disabled || loading} onclick={onAllowInternal}>Allow</button
	>
</div>
