<script lang="ts" module>
	export interface ClientProps {
		user: User;
		client: Client;
	}
</script>

<script lang="ts">
	import type { Client, User } from '$lib/common/openapi/oidc';

	let { user, client }: ClientProps = $props();

	function onDeny() {}
	function onApprove() {}
</script>

<div class="flex flex-col justify-center">
	{#if client.logoUri}
		<img
			src={client.logoUri}
			alt={`${client.name} logo`}
			class="mx-auto mb-3 h-16 w-16 rounded-full"
		/>
	{/if}

	<h1 class="text-xl font-semibold">
		{client.name}
	</h1>

	<p class="mt-1">
		wants to access your account as <strong>{user.username}</strong>
	</p>
</div>

<div class="my-4 flex flex-col justify-center">
	<h2 class="text-sm font-medium">Requested permissions</h2>

	{#if client.scopes && client.scopes.length}
		<ul class="list-inside list-disc text-sm">
			{#each client.scopes as scope}
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
	<button class="btn secondary" onclick={onDeny}>Deny</button>
	<button class="btn primary" onclick={onApprove}>Allow</button>
</div>
