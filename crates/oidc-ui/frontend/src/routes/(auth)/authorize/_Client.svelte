<script lang="ts" module>
	export interface ClientProps {
		user: User;
		client: ClientInfo;
		onAcceptClient: () => Promise<void>;
		onRejectClient: () => Promise<void>;
	}
</script>

<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc';
	import { getAverageLuminance } from '$lib/common/util/canvas';
	import type { ClientInfo } from './+page.svelte';
	import ClientInfoLogo from './_ClientInfoLogo.svelte';

	let { user, client, onAcceptClient, onRejectClient }: ClientProps = $props();

	let clientLogoUriElement = $state<HTMLImageElement | null>();
	let isClientLogoDark = $state(true);

	$effect(() => {
		if (clientLogoUriElement) {
			getAverageLuminance(clientLogoUriElement).then((luminance) => {
				isClientLogoDark = luminance < 200;
			});
		}
	});

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

<div class="flex flex-col justify-center">
	<div class="flex flex-col items-center justify-center">
		<ClientInfoLogo {client} />

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
	<button class="btn secondary" disabled={loading} onclick={onRejectClientInternal}>Deny</button>
	<button class="btn primary" disabled={loading} onclick={onAcceptClientInternal}>Allow</button>
</div>
