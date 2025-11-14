<script lang="ts" module>
	export interface ClientProps {
		user: User;
		client: Client;
	}
</script>

<script lang="ts">
	import { clientApi } from '$lib/common/openapi';

	import type { Client, User } from '$lib/common/openapi/oidc';
	import { getAverageLuminance } from '$lib/common/util/canvas';

	let { user, client }: ClientProps = $props();

	let clientLogoUriElement = $state<HTMLImageElement | null>();
	let isClientLogoDark = $state(true);

	$effect(() => {
		if (clientLogoUriElement) {
			getAverageLuminance(clientLogoUriElement).then((luminance) => {
				isClientLogoDark = luminance < 200;
			});
		}
	});

	let allowedScopes = $state<Record<string, boolean>>(
		Object.fromEntries(client.scopes.map((scope) => [scope, true]))
	);

	function onDeny() {}
	async function onApprove() {
		try {
			await clientApi.clientUserApprove({
				clientId: client.clientId
			});
		} catch (e) {
			console.error(e);
		}
	}
</script>

<div class="flex flex-col justify-center">
	<div class="flex flex-col items-center justify-center">
		{#if client.logoUri}
			<img
				bind:this={clientLogoUriElement}
				src={client.logoUri}
				alt={`${client.name} logo`}
				crossorigin="anonymous"
				class={{
					'h-24 w-24 rounded-full p-4': true,
					'bg-black': !isClientLogoDark,
					'bg-white': isClientLogoDark
				}}
			/>
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
	<button class="btn secondary" onclick={onDeny}>Deny</button>
	<button class="btn primary" onclick={onApprove}>Allow</button>
</div>
