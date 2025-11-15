<script lang="ts" module>
	import type { User } from '$lib/common/openapi/oidc';

	export interface ClientNewProps {
		user: User;
		clientId: string;
		clientInfo: ClientInfo | null;
		onAcceptClient: () => Promise<void>;
		onRejectClient: () => Promise<void>;
	}
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	import type { ClientInfo } from './+page.svelte';
	import ClientInfoComponent from './_ClientInfo.svelte';

	let { user, clientId, clientInfo, onAcceptClient, onRejectClient }: ClientNewProps = $props();
</script>

{#if clientInfo}
	<ClientInfoComponent {user} client={clientInfo} {onAcceptClient} {onRejectClient} />
{:else}
	<h6><code>{clientId}</code></h6>
	<p>
		This client ID is requesting to be added to your OIDC provider. However it is not found in your
		database.
	</p>

	<section>
		<div class="mt-4 flex flex-row justify-center gap-4">
			<button class="btn secondary" onclick={onRejectClient}>{m.client_reject()}</button>
		</div>
	</section>
{/if}
