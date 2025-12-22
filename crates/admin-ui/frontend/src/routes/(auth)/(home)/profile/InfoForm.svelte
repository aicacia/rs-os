<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const InfoFormSchema = () =>
		v.object({
			givenName: v.optional(v.string()),
			middleName: v.optional(v.string()),
			familyName: v.optional(v.string()),
			nickname: v.optional(v.string()),
			website: v.optional(v.string()),
			locale: v.optional(v.string()),
			address: v.optional(v.string()),
			birthdate: v.optional(v.number()),
			gender: v.optional(v.string()),
			zoneInfo: v.optional(v.string()),
			profilePicture: v.optional(v.string())
		});
</script>

<script lang="ts">
	import type { CurrentUser } from '$lib/common/openapi/admin/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';
	import { invalidateAll } from '$app/navigation';

	let { user = $bindable() }: { user: CurrentUser } = $props();

	const supportedLocales = $derived(
		typeof navigator !== 'undefined' ? Array.from(new Set(navigator.languages)) : []
	);

	const timeZones = $derived(
		typeof Intl !== 'undefined' && Intl.supportedValuesOf ? Intl.supportedValuesOf('timeZone') : []
	);

	const form = createForm(InfoFormSchema(), {
		givenName: user.info?.givenName ?? undefined,
		middleName: user.info?.middleName ?? undefined,
		familyName: user.info?.familyName ?? undefined,
		nickname: user.info?.nickname ?? undefined,
		website: user.info?.website ?? undefined,
		locale: user.info?.locale ?? undefined,
		address: user.info?.address ?? undefined,
		birthdate: user.info?.birthdate ?? undefined,
		gender: user.info?.gender ?? undefined,
		zoneInfo: user.info?.zoneInfo ?? undefined,
		profilePicture: user.info?.profilePicture ?? undefined
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
	$effect(() => {
		form.fields.middleName.value = user.info?.middleName ?? undefined;
	});
	$effect(() => {
		form.fields.nickname.value = user.info?.nickname ?? undefined;
	});
	$effect(() => {
		form.fields.address.value = user.info?.address ?? undefined;
	});
	$effect(() => {
		form.fields.birthdate.value = user.info?.birthdate ?? undefined;
	});
	$effect(() => {
		form.fields.gender.value = user.info?.gender ?? undefined;
	});
	$effect(() => {
		form.fields.zoneInfo.value = user.info?.zoneInfo ?? undefined;
	});
	$effect(() => {
		form.fields.profilePicture.value = user.info?.profilePicture ?? undefined;
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			user.info = await currentUserApi.updateUserInfo({ updateUserInfoRequest: value });
			await invalidateAll();
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3 class="text-lg font-medium">{m.profile_info_title()}</h3>
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
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
			<span class="text-sm font-medium">{m.profile_middle_name_label()}</span>
			<input
				id="profile-middle-name"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.middleName.value}
				placeholder={m.profile_middle_name_placeholder()}
				aria-label={m.profile_middle_name_label()}
			/>
			<Issues issues={form.fields.middleName.issues} />
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
			<span class="text-sm font-medium">{m.profile_nickname_label()}</span>
			<input
				id="profile-nickname"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.nickname.value}
				placeholder={m.profile_nickname_placeholder()}
				aria-label={m.profile_nickname_label()}
			/>
			<Issues issues={form.fields.nickname.issues} />
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
		<label class="block">
			<span class="text-sm font-medium">{m.profile_locale_label()}</span>
			<select
				id="profile-locale"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.locale.value}
				placeholder={m.profile_locale_placeholder()}
				aria-label={m.profile_locale_label()}
			>
				{#each supportedLocales as locale}
					<option value={locale}>{locale}</option>
				{/each}
			</select>
			<Issues issues={form.fields.locale.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_zone_info_label()}</span>
			<select
				id="profile-zone-info"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.zoneInfo.value}
				placeholder={m.profile_zone_info_placeholder()}
				aria-label={m.profile_zone_info_label()}
			>
				<option value="">{m.profile_zone_info_placeholder()}</option>
				{#each timeZones as timeZone}
					<option value={timeZone}>{timeZone}</option>
				{/each}
			</select>
			<Issues issues={form.fields.zoneInfo.issues} />
		</label>
		<label class="block md:col-span-2">
			<span class="text-sm font-medium">{m.profile_address_label()}</span>
			<input
				id="profile-address"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.address.value}
				placeholder={m.profile_address_placeholder()}
				aria-label={m.profile_address_label()}
			/>
			<Issues issues={form.fields.address.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_birthdate_label()}</span>
			<input
				id="profile-birthdate"
				type="date"
				class="mt-1 block w-full px-3 py-2"
				value={form.fields.birthdate.value
					? new Date(form.fields.birthdate.value * 1000).toISOString().split('T')[0]
					: ''}
				onchange={(e) => {
					const value = e.currentTarget.value;
					form.fields.birthdate.value = value
						? Math.floor(new Date(value).getTime() / 1000)
						: undefined;
				}}
				placeholder={m.profile_birthdate_placeholder()}
				aria-label={m.profile_birthdate_label()}
			/>
			<Issues issues={form.fields.birthdate.issues} />
		</label>
		<label class="block">
			<span class="text-sm font-medium">{m.profile_gender_label()}</span>
			<input
				id="profile-gender"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.gender.value}
				placeholder={m.profile_gender_placeholder()}
				aria-label={m.profile_gender_label()}
			/>
			<Issues issues={form.fields.gender.issues} />
		</label>
		<label class="block md:col-span-2">
			<span class="text-sm font-medium">{m.profile_profile_picture_label()}</span>
			<input
				id="profile-picture"
				type="url"
				class="mt-1 block w-full px-3 py-2"
				bind:value={form.fields.profilePicture.value}
				placeholder={m.profile_profile_picture_placeholder()}
				aria-label={m.profile_profile_picture_label()}
			/>
			<Issues issues={form.fields.profilePicture.issues} />
		</label>
	</div>
	<button type="submit" class="btn success mt-4">{m.profile_save_profile()}</button>
</form>
