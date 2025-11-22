<script lang="ts" module>
	import { Permission, type User } from '$lib/common/openapi/oidc';
	import type { ClientInfo } from './_utils';

	export interface ClientInfoProps {
		user: User;
		client: Partial<ClientInfo> & Pick<ClientInfo, 'logoUri' | 'name'>;
		disabled?: boolean;
		isNew: boolean;
		onAccept: (updates: ClientInfo) => Promise<void>;
		onReject: () => Promise<void>;
	}
</script>

<script lang="ts">
	import { hasPermission } from '$lib/common/state/currentUser.svelte';
	import { m } from '$lib/paraglide/messages';
	import ClientHeader from './_ClientHeader.svelte';
	import ClientFields from './_ClientFields.svelte';

	let { user, client, disabled, isNew, onAccept, onReject }: ClientInfoProps = $props();

	let loading = $state(false);

	async function onAcceptInternal() {
		try {
			loading = true;
			await onAccept(client as ClientInfo);
		} finally {
			loading = false;
		}
	}
	async function onRejectInternal() {
		try {
			loading = true;
			await onReject();
		} finally {
			loading = false;
		}
	}
</script>

<ClientHeader {client} />

<hr />

{#if isNew}
	<p>
		This client is requesting to be added to your OIDC provider. Review the details before
		approving.
	</p>
{:else}
	<p>
		This client has updated its configuration and is requesting changes to your OIDC provider.
		Review the details before approving.
	</p>
{/if}

<ClientFields {client} />

<hr />

<section>
	<div class="mt-4 flex flex-row justify-center gap-4">
		<button class="btn secondary" disabled={disabled || loading} onclick={onRejectInternal}
			>{m.client_reject()}</button
		>
		{#if hasPermission(user, Permission.ClientCreate)}
			<button class="btn danger" disabled={disabled || loading} onclick={onAcceptInternal}
				>{m.client_accept()}</button
			>
		{/if}
	</div>
</section>
