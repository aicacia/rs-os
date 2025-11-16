<script lang="ts" module>
	import type { ClientInfo } from './_utils';

	export interface ClientFieldsProps {
		client: Partial<ClientInfo>;
	}
</script>

<script lang="ts">
	let { client }: ClientFieldsProps = $props();
</script>

{#if client.scopes?.length}
	<section>
		<h5>Scopes</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.scopes as s}
				<li>{s}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if client.redirectUris?.length}
	<section>
		<h5>Redirect URIs</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.redirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if client.postLogoutRedirectUris?.length}
	<section>
		<h5>Post-logout Redirect URIs</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.postLogoutRedirectUris as uri}
				<li>{uri}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if client.grantTypes?.length}
	<section>
		<h5>Grant Types</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.grantTypes as t}
				<li>{t}</li>
			{/each}
		</ul>
	</section>
{/if}

{#if client.responseTypes?.length}
	<section>
		<h5>Response Types</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#each client.responseTypes as r}
				<li>{r}</li>
			{/each}
		</ul>
	</section>
{/if}

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

{#if client.accessTokenExpiresInSeconds || client.idTokenExpiresInSeconds || client.refreshExpiresInSeconds}
	<section>
		<h5>Token Expiry</h5>
		<ul class="list-inside list-disc space-y-1 text-sm">
			{#if client.accessTokenExpiresInSeconds}
				<li><strong>Access Token Expires:</strong> {client.accessTokenExpiresInSeconds}s</li>
			{/if}
			{#if client.idTokenExpiresInSeconds}
				<li><strong>ID Token Expires:</strong> {client.idTokenExpiresInSeconds}s</li>
			{/if}
			{#if client.refreshExpiresInSeconds}
				<li><strong>Refresh Token Expires:</strong> {client.refreshExpiresInSeconds}s</li>
			{/if}
		</ul>
	</section>
{/if}

{#if client.policyUri || client.termsOfServiceUri}
	<section>
		<h5>Legal</h5>
		<ul class="space-y-1 text-sm">
			{#if client.policyUri}
				<li>
					<a href={client.policyUri} target="_blank">Privacy Policy</a>
				</li>
			{/if}
			{#if client.termsOfServiceUri}
				<li>
					<a href={client.termsOfServiceUri} target="_blank">Terms of Service</a>
				</li>
			{/if}
		</ul>
	</section>
{/if}
