<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { UpdateUserInfoRequest, User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { m } from '$lib/paraglide/messages';

	let { user = $bindable() }: { user: User } = $props();

	let infoForm = $state<UpdateUserInfoRequest>({
		name: undefined,
		givenName: undefined,
		familyName: undefined,
		website: undefined,
		locale: undefined
	});

	$effect(() => {
		infoForm = {
			name: user.info?.name ?? user.username ?? '',
			givenName: user.info?.givenName ?? undefined,
			familyName: user.info?.familyName ?? undefined,
			website: user.info?.website ?? undefined,
			locale: user.info?.locale ?? undefined
		};
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();
		try {
			user = await currentUserApi.updateUserInfo({ updateUserInfoRequest: infoForm });
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3 class="text-lg font-medium">{m.profile_info_title()}</h3>
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<label class="block">
			<span class="text-sm font-medium">{m.profile_full_name_label()}</span>
			<input
				id="profile-full-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={infoForm.name}
				placeholder={m.profile_full_name_placeholder()}
				aria-label={m.profile_full_name_label()}
			/>
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_given_name_label()}</span>
			<input
				id="profile-given-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={infoForm.givenName}
				placeholder={m.profile_given_name_placeholder()}
				aria-label={m.profile_given_name_label()}
			/>
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_family_name_label()}</span>
			<input
				id="profile-family-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={infoForm.familyName}
				placeholder={m.profile_family_name_placeholder()}
				aria-label={m.profile_family_name_label()}
			/>
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_website_label()}</span>
			<input
				id="profile-website"
				class="mt-1 block w-full px-3 py-2"
				bind:value={infoForm.website}
				placeholder={m.profile_website_placeholder()}
				aria-label={m.profile_website_label()}
			/>
		</label>
		<label class="block md:col-span-2">
			<span class="text-sm font-medium">{m.profile_locale_label()}</span>
			<input
				id="profile-locale"
				class="mt-1 block w-full px-3 py-2"
				bind:value={infoForm.locale}
				placeholder={m.profile_locale_placeholder()}
				aria-label={m.profile_locale_label()}
			/>
		</label>
	</div>
	<button type="submit" class="btn success mt-4">{m.profile_save_profile()}</button>
</form>
