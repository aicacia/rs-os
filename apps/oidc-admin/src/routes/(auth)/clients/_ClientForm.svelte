<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	export const ClientFormSchema = () =>
		v.object({
			clientId: v.pipe(v.string(), v.minLength(1, m.client_form_validation_client_id_required())),
			name: v.pipe(v.string(), v.minLength(1, m.client_form_validation_name_required())),
			clientSecret: v.optional(v.nullable(v.string())),
			applicationType: v.pipe(
				v.string(),
				v.minLength(1, m.client_form_validation_app_type_required())
			),
			authMethod: v.pipe(
				v.string(),
				v.minLength(1, m.client_form_validation_auth_method_required())
			),
			grantTypes: v.pipe(
				v.array(v.string()),
				v.minLength(1, m.client_form_validation_grant_types_required())
			),
			responseTypes: v.pipe(
				v.array(v.string()),
				v.minLength(1, m.client_form_validation_response_types_required())
			),
			scopes: v.pipe(
				v.array(v.string()),
				v.minLength(1, m.client_form_validation_scopes_required())
			),
			redirectUris: v.optional(v.nullable(v.array(v.string()))),
			postLogoutRedirectUris: v.optional(v.nullable(v.array(v.string()))),
			audience: v.optional(v.nullable(v.array(v.string()))),
			logoUri: v.optional(v.nullable(v.string())),
			clientUri: v.optional(v.nullable(v.string())),
			policyUri: v.optional(v.nullable(v.string())),
			termsOfServiceUri: v.optional(v.nullable(v.string())),
			accessTokenExpiresInSeconds: v.pipe(
				v.number(),
				v.minValue(1, m.client_form_validation_min_seconds())
			),
			idTokenExpiresInSeconds: v.pipe(
				v.number(),
				v.minValue(1, m.client_form_validation_min_seconds())
			),
			refreshExpiresInSeconds: v.pipe(
				v.number(),
				v.minValue(1, m.client_form_validation_min_seconds())
			)
		});

	export type ClientFormData = v.InferOutput<ReturnType<typeof ClientFormSchema>>;
</script>

<script lang="ts">
	import type { ClientUpsertRequest, Client } from '$lib/common/openapi/oidc-admin/models/index';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';
	import { X, Plus, Eye, EyeOff } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	let {
		initialValues,
		onsubmit,
		readonly = false,
		actions
	}: {
		initialValues: Partial<ClientUpsertRequest> | Partial<Client>;
		onsubmit: (data: ClientFormData) => void | Promise<void>;
		readonly?: boolean;
		actions?: Snippet;
	} = $props();

	const form = createForm(ClientFormSchema(), {
		clientId: initialValues.clientId ?? '',
		name: initialValues.name ?? '',
		clientSecret: ('clientSecret' in initialValues ? initialValues.clientSecret : null) ?? null,
		applicationType: initialValues.applicationType ?? 'web',
		authMethod: initialValues.authMethod ?? 'client_secret_basic',
		grantTypes: initialValues.grantTypes ?? ['authorization_code'],
		responseTypes: initialValues.responseTypes ?? ['code'],
		scopes: initialValues.scopes ?? ['openid'],
		redirectUris: initialValues.redirectUris ?? null,
		postLogoutRedirectUris: initialValues.postLogoutRedirectUris ?? null,
		audience: initialValues.audience ?? null,
		logoUri: initialValues.logoUri ?? null,
		clientUri: initialValues.clientUri ?? null,
		policyUri: initialValues.policyUri ?? null,
		termsOfServiceUri: initialValues.termsOfServiceUri ?? null,
		accessTokenExpiresInSeconds: initialValues.accessTokenExpiresInSeconds ?? 3600,
		idTokenExpiresInSeconds: initialValues.idTokenExpiresInSeconds ?? 3600,
		refreshExpiresInSeconds: initialValues.refreshExpiresInSeconds ?? 86400
	});

	// Client secret visibility - hidden by default if has value
	const clientSecretIsEmpty = !form.fields.clientSecret.value;
	let showClientSecret = $state(clientSecretIsEmpty);

	// State for managing arrays - we'll use simple state tracking instead of the complex form array fields
	let grantTypes = $state<string[]>(initialValues.grantTypes ?? ['authorization_code']);
	let responseTypes = $state<string[]>(initialValues.responseTypes ?? ['code']);
	let scopesState = $state<string[]>(initialValues.scopes ?? ['openid']);
	let redirectUris = $state<string[]>(
		(initialValues.redirectUris as string[] | null | undefined) ?? []
	);
	let postLogoutRedirectUris = $state<string[]>(
		(initialValues.postLogoutRedirectUris as string[] | null | undefined) ?? []
	);
	let audienceState = $state<string[]>(
		(initialValues.audience as string[] | null | undefined) ?? []
	);

	// String array management
	let redirectUriInput = $state('');
	let postLogoutRedirectUriInput = $state('');
	let audienceInput = $state('');

	function addRedirectUri() {
		if (redirectUriInput.trim()) {
			redirectUris = [...redirectUris, redirectUriInput.trim()];
			redirectUriInput = '';
		}
	}

	function removeRedirectUri(index: number) {
		redirectUris = redirectUris.filter((_, i) => i !== index);
	}

	function addPostLogoutRedirectUri() {
		if (postLogoutRedirectUriInput.trim()) {
			postLogoutRedirectUris = [...postLogoutRedirectUris, postLogoutRedirectUriInput.trim()];
			postLogoutRedirectUriInput = '';
		}
	}

	function removePostLogoutRedirectUri(index: number) {
		postLogoutRedirectUris = postLogoutRedirectUris.filter((_, i) => i !== index);
	}

	function addAudience() {
		if (audienceInput.trim()) {
			audienceState = [...audienceState, audienceInput.trim()];
			audienceInput = '';
		}
	}

	function removeAudience(index: number) {
		audienceState = audienceState.filter((_, i) => i !== index);
	}

	// Multi-select management for grant types, response types, and scopes
	const availableGrantTypes = [
		'authorization_code',
		'implicit',
		'password',
		'client_credentials',
		'refresh_token',
		'urn:ietf:params:oauth:grant-type:device_code'
	];

	const availableResponseTypes = ['code', 'token', 'id_token'];

	const availableScopes = ['openid', 'profile', 'email', 'address', 'phone', 'offline_access'];

	function toggleGrantType(grantType: string) {
		if (grantTypes.includes(grantType)) {
			grantTypes = grantTypes.filter((gt) => gt !== grantType);
		} else {
			grantTypes = [...grantTypes, grantType];
		}
	}

	function toggleResponseType(responseType: string) {
		if (responseTypes.includes(responseType)) {
			responseTypes = responseTypes.filter((rt) => rt !== responseType);
		} else {
			responseTypes = [...responseTypes, responseType];
		}
	}

	function toggleScope(scope: string) {
		if (scopesState.includes(scope)) {
			scopesState = scopesState.filter((s) => s !== scope);
		} else {
			scopesState = [...scopesState, scope];
		}
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		const formData: ClientFormData = {
			clientId: form.fields.clientId.value ?? '',
			name: form.fields.name.value ?? '',
			clientSecret: form.fields.clientSecret.value,
			applicationType: form.fields.applicationType.value ?? '',
			authMethod: form.fields.authMethod.value ?? '',
			grantTypes,
			responseTypes,
			scopes: scopesState,
			redirectUris: redirectUris.length > 0 ? redirectUris : null,
			postLogoutRedirectUris: postLogoutRedirectUris.length > 0 ? postLogoutRedirectUris : null,
			audience: audienceState.length > 0 ? audienceState : null,
			logoUri: form.fields.logoUri.value,
			clientUri: form.fields.clientUri.value,
			policyUri: form.fields.policyUri.value,
			termsOfServiceUri: form.fields.termsOfServiceUri.value,
			accessTokenExpiresInSeconds: form.fields.accessTokenExpiresInSeconds.value ?? 3600,
			idTokenExpiresInSeconds: form.fields.idTokenExpiresInSeconds.value ?? 3600,
			refreshExpiresInSeconds: form.fields.refreshExpiresInSeconds.value ?? 86400
		};

		const [, basicErr] = await form.validate();
		if (basicErr) {
			return;
		}

		await onsubmit(formData);
	}
</script>

<form onsubmit={handleSubmit} class="space-y-4">
	<section class="card">
		<h3>{m.client_form_basic_info()}</h3>
		<div class="flex flex-col gap-4">
			<label class="flex flex-col">
				<span>{m.client_form_client_id()} *</span>
				<input
					type="text"
					class="w-full"
					bind:value={form.fields.clientId.value}
					placeholder={m.client_form_client_id_placeholder()}
					{readonly}
					aria-label={m.client_form_client_id()}
				/>
				<Issues issues={form.fields.clientId.issues} />
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_name()} *</span>
				<input
					type="text"
					class="w-full"
					bind:value={form.fields.name.value}
					placeholder={m.client_form_name_placeholder()}
					{readonly}
					aria-label={m.client_form_name()}
				/>
				<Issues issues={form.fields.name.issues} />
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_client_secret()}</span>
				<div class="relative">
					<input
						type={showClientSecret ? 'text' : 'password'}
						class="w-full pr-10"
						bind:value={form.fields.clientSecret.value}
						placeholder={m.client_form_client_secret_placeholder()}
						{readonly}
						aria-label={m.client_form_client_secret()}
					/>
					{#if !readonly}
						<button
							type="button"
							onclick={() => (showClientSecret = !showClientSecret)}
							class="btn icon sm absolute top-1/2 right-2 -translate-y-1/2"
							aria-label={showClientSecret ? 'Hide client secret' : 'Show client secret'}
						>
							{#if showClientSecret}
								<EyeOff class="h-4 w-4" />
							{:else}
								<Eye class="h-4 w-4" />
							{/if}
						</button>
					{/if}
				</div>
				<Issues issues={form.fields.clientSecret.issues} />
			</label>
		</div>
	</section>

	<!-- Application Configuration -->
	<section class="card">
		<h3>{m.client_form_app_config()}</h3>
		<div class="flex flex-col gap-4">
			<label class="flex flex-col">
				<span>{m.client_form_app_type()} *</span>
				<select class="w-full" bind:value={form.fields.applicationType.value} disabled={readonly}>
					<option value="web">{m.client_form_app_type_web()}</option>
					<option value="native">{m.client_form_app_type_native()}</option>
					<option value="service">{m.client_form_app_type_service()}</option>
				</select>
				<Issues issues={form.fields.applicationType.issues} />
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_auth_method()} *</span>
				<select class="w-full" bind:value={form.fields.authMethod.value} disabled={readonly}>
					<option value="client_secret_basic">{m.client_form_auth_method_basic()}</option>
					<option value="client_secret_post">{m.client_form_auth_method_post()}</option>
					<option value="none">{m.client_form_auth_method_none()}</option>
				</select>
				<Issues issues={form.fields.authMethod.issues} />
			</label>
		</div>
	</section>

	<!-- OAuth2 Configuration -->
	<section class="card">
		<h3>{m.client_form_oauth2_config()}</h3>

		<div class="flex flex-col gap-2">
			<span>{m.client_form_grant_types()} *</span>
			<div class="flex flex-wrap gap-2">
				{#each availableGrantTypes as grantType}
					<label class="flex gap-2">
						<input
							type="checkbox"
							checked={grantTypes.includes(grantType)}
							onchange={() => toggleGrantType(grantType)}
							disabled={readonly}
						/>
						<span>{grantType}</span>
					</label>
				{/each}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span>{m.client_form_response_types()} *</span>
			<div class="flex flex-wrap gap-2">
				{#each availableResponseTypes as responseType}
					<label class="flex gap-2">
						<input
							type="checkbox"
							checked={responseTypes.includes(responseType)}
							onchange={() => toggleResponseType(responseType)}
							disabled={readonly}
						/>
						<span>{responseType}</span>
					</label>
				{/each}
			</div>
		</div>

		<div class="flex flex-col gap-2">
			<span>{m.client_form_scopes()} *</span>
			<div class="flex flex-wrap gap-2">
				{#each availableScopes as scope}
					<label class="flex gap-2">
						<input
							type="checkbox"
							checked={scopesState.includes(scope)}
							onchange={() => toggleScope(scope)}
							disabled={readonly}
						/>
						<span>{scope}</span>
					</label>
				{/each}
			</div>
		</div>
	</section>

	<!-- URIs -->
	<section class="card">
		<h3>{m.client_form_uris()}</h3>
		<!-- Redirect URIs -->
		<div class="flex flex-col gap-2">
			<span>{m.client_form_redirect_uris()}</span>
			{#if !readonly}
				<div class="relative">
					<input
						type="url"
						class="w-full pr-10"
						bind:value={redirectUriInput}
						placeholder={m.client_form_redirect_uri_placeholder()}
						onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addRedirectUri())}
					/>
					<button
						type="button"
						onclick={addRedirectUri}
						class="btn success icon sm absolute top-1/2 right-2 -translate-y-1/2"
						aria-label="Add redirect URI"
					>
						<Plus class="h-4 w-4" />
					</button>
				</div>
			{/if}
			<div class="flex flex-wrap gap-2">
				{#each redirectUris as uri, index}
					<span class="badge primary">
						{uri}
						{#if !readonly}
							<button
								type="button"
								onclick={() => removeRedirectUri(index)}
								class="btn icon danger ms-2"
							>
								<X />
							</button>
						{/if}
					</span>
				{/each}
			</div>
		</div>

		<!-- Post Logout Redirect URIs -->
		<div class="flex flex-col gap-2">
			<span>{m.client_form_post_logout_redirect_uris()}</span>
			{#if !readonly}
				<div class="relative">
					<input
						type="url"
						class="w-full pr-10"
						bind:value={postLogoutRedirectUriInput}
						placeholder={m.client_form_post_logout_redirect_uri_placeholder()}
						onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addPostLogoutRedirectUri())}
					/>
					<button
						type="button"
						onclick={addPostLogoutRedirectUri}
						class="btn success icon sm absolute top-1/2 right-2 -translate-y-1/2"
						aria-label="Add post logout redirect URI"
					>
						<Plus class="h-4 w-4" />
					</button>
				</div>
			{/if}
			<div class="flex flex-wrap gap-2">
				{#each postLogoutRedirectUris as uri, index}
					<span class="badge primary">
						{uri}
						{#if !readonly}
							<button
								type="button"
								onclick={() => removePostLogoutRedirectUri(index)}
								class="btn icon danger ms-2"
							>
								<X />
							</button>
						{/if}
					</span>
				{/each}
			</div>
		</div>

		<!-- Additional URIs -->
		<div class="flex flex-col gap-4">
			<label class="flex flex-col">
				<span>{m.client_form_logo_uri()}</span>
				<input
					type="url"
					class="w-full"
					bind:value={form.fields.logoUri.value}
					placeholder={m.client_form_logo_uri_placeholder()}
					{readonly}
				/>
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_client_uri()}</span>
				<input
					type="url"
					class="w-full"
					bind:value={form.fields.clientUri.value}
					placeholder={m.client_form_client_uri_placeholder()}
					{readonly}
				/>
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_policy_uri()}</span>
				<input
					type="url"
					class="w-full"
					bind:value={form.fields.policyUri.value}
					placeholder={m.client_form_policy_uri_placeholder()}
					{readonly}
				/>
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_terms_uri()}</span>
				<input
					type="url"
					class="w-full"
					bind:value={form.fields.termsOfServiceUri.value}
					placeholder={m.client_form_terms_uri_placeholder()}
					{readonly}
				/>
			</label>
		</div>
	</section>

	<!-- Audience -->
	<section class="card">
		<h3>{m.client_form_audience()}</h3>
		<div class="flex flex-col gap-2">
			{#if !readonly}
				<div class="relative">
					<input
						type="text"
						class="w-full pr-10"
						bind:value={audienceInput}
						placeholder={m.client_form_audience_placeholder()}
						onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addAudience())}
					/>
					<button
						type="button"
						onclick={addAudience}
						class="btn success icon sm absolute top-1/2 right-2 -translate-y-1/2"
						aria-label="Add audience"
					>
						<Plus class="h-4 w-4" />
					</button>
				</div>
			{/if}
			<div class="flex flex-wrap gap-2">
				{#each audienceState as aud, index}
					<span class="badge primary">
						{aud}
						{#if !readonly}
							<button
								type="button"
								onclick={() => removeAudience(index)}
								class="btn icon danger ms-2"
							>
								<X />
							</button>
						{/if}
					</span>
				{/each}
			</div>
		</div>
	</section>

	<!-- Token Expiration -->
	<section class="card">
		<h3>{m.client_form_token_expiration()}</h3>
		<div class="flex flex-col gap-4">
			<label class="flex flex-col">
				<span>{m.client_form_access_token()} *</span>
				<input
					type="number"
					class="w-full"
					bind:value={form.fields.accessTokenExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.accessTokenExpiresInSeconds.issues} />
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_id_token()} *</span>
				<input
					type="number"
					class="w-full"
					bind:value={form.fields.idTokenExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.idTokenExpiresInSeconds.issues} />
			</label>

			<label class="flex flex-col">
				<span>{m.client_form_refresh_token()} *</span>
				<input
					type="number"
					class="w-full"
					bind:value={form.fields.refreshExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.refreshExpiresInSeconds.issues} />
			</label>
		</div>
	</section>

	{#if !readonly}
		<div class="flex justify-end gap-2">
			{#if actions}
				{@render actions()}
			{/if}
			<button type="submit" class="btn primary">
				{#if initialValues.id}
					{m.clients_update_button()}
				{:else}
					{m.clients_create_button()}
				{/if}
			</button>
		</div>
	{/if}
</form>
