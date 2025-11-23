<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const InfoFormSchema = () =>
		v.object({
			name: v.optional(v.string()),
			givenName: v.optional(v.string()),
			familyName: v.optional(v.string()),
			website: v.optional(v.string()),
			locale: v.optional(v.string())
		});
</script>

<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';

	let { user = $bindable() }: { user: User } = $props();

	const form = createForm(InfoFormSchema(), {
		name: user.info?.name ?? user.username ?? '',
		givenName: user.info?.givenName ?? undefined,
		familyName: user.info?.familyName ?? undefined,
		website: user.info?.website ?? undefined,
		locale: user.info?.locale ?? undefined
	});

	$effect(() => {
		form.fields.name.value = user.info?.name ?? user.username ?? '';
	});
	$effect(() => {
		form.fields.givenName.value = user.info?.givenName ?? undefined;
	});
	$effect(() => {
		form.fields.familyName.value = user.info?.familyName ?? undefined;
	});
	$effect(() => {
		form.fields.website.value = user.info?.website ?? undefined;
	});
	$effect(() => {
		form.fields.locale.value = user.info?.locale ?? undefined;
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			user = await currentUserApi.updateUserInfo({ updateUserInfoRequest: value });
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
				bind:value={form.fields.name.value}
				placeholder={m.profile_full_name_placeholder()}
				aria-label={m.profile_full_name_label()}
			/>
			<Issues issues={form.fields.name.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_given_name_label()}</span>
			<input
				id="profile-given-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.givenName.value}
				placeholder={m.profile_given_name_placeholder()}
				aria-label={m.profile_given_name_label()}
			/>
			<Issues issues={form.fields.givenName.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_family_name_label()}</span>
			<input
				id="profile-family-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.familyName.value}
				placeholder={m.profile_family_name_placeholder()}
				aria-label={m.profile_family_name_label()}
			/>
			<Issues issues={form.fields.familyName.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_website_label()}</span>
			<input
				id="profile-website"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.website.value}
				placeholder={m.profile_website_placeholder()}
				aria-label={m.profile_website_label()}
			/>
			<Issues issues={form.fields.website.issues} />
		</label>
		<label class="block md:col-span-2">
			<span class="text-sm font-medium">{m.profile_locale_label()}</span>
			<input
				id="profile-locale"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.locale.value}
				placeholder={m.profile_locale_placeholder()}
				aria-label={m.profile_locale_label()}
			/>
			<Issues issues={form.fields.locale.issues} />
		</label>
	</div>
	<button type="submit" class="btn success mt-4">{m.profile_save_profile()}</button>
</form>
