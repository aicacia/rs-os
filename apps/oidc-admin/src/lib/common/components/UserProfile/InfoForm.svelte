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

	export interface UserInfo {
		givenName?: string;
		middleName?: string;
		familyName?: string;
		nickname?: string;
		website?: string;
		locale?: string;
		address?: string;
		birthdate?: number;
		gender?: string;
		zoneInfo?: string;
		profilePicture?: string;
	}

	export interface InfoFormProps {
		userInfo: UserInfo;
		onUpdate: (info: UserInfo) => Promise<UserInfo>;
		readonly?: boolean;
	}
</script>

<script lang="ts">
	import { createForm } from '@aicacia/svelte-forms';
	import Issues from '$lib/common/components/Issues.svelte';
	import { handleError } from '$lib/common/errors';

	let { userInfo = $bindable(), onUpdate, readonly = false }: InfoFormProps = $props();

	const supportedLocales = $derived(
		typeof navigator !== 'undefined' ? Array.from(new Set(navigator.languages)) : []
	);

	const timeZones = $derived(
		typeof Intl !== 'undefined' && Intl.supportedValuesOf ? Intl.supportedValuesOf('timeZone') : []
	);

	const form = createForm(InfoFormSchema(), {
		givenName: userInfo?.givenName ?? undefined,
		middleName: userInfo?.middleName ?? undefined,
		familyName: userInfo?.familyName ?? undefined,
		nickname: userInfo?.nickname ?? undefined,
		website: userInfo?.website ?? undefined,
		locale: userInfo?.locale ?? undefined,
		address: userInfo?.address ?? undefined,
		birthdate: userInfo?.birthdate ?? undefined,
		gender: userInfo?.gender ?? undefined,
		zoneInfo: userInfo?.zoneInfo ?? undefined,
		profilePicture: userInfo?.profilePicture ?? undefined
	});

	$effect(() => {
		form.fields.givenName.value = userInfo?.givenName ?? undefined;
	});
	$effect(() => {
		form.fields.familyName.value = userInfo?.familyName ?? undefined;
	});
	$effect(() => {
		form.fields.website.value = userInfo?.website ?? undefined;
	});
	$effect(() => {
		form.fields.locale.value = userInfo?.locale ?? undefined;
	});
	$effect(() => {
		form.fields.middleName.value = userInfo?.middleName ?? undefined;
	});
	$effect(() => {
		form.fields.nickname.value = userInfo?.nickname ?? undefined;
	});
	$effect(() => {
		form.fields.address.value = userInfo?.address ?? undefined;
	});
	$effect(() => {
		form.fields.birthdate.value = userInfo?.birthdate ?? undefined;
	});
	$effect(() => {
		form.fields.gender.value = userInfo?.gender ?? undefined;
	});
	$effect(() => {
		form.fields.zoneInfo.value = userInfo?.zoneInfo ?? undefined;
	});
	$effect(() => {
		form.fields.profilePicture.value = userInfo?.profilePicture ?? undefined;
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (readonly) return;

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			userInfo = await onUpdate(value);
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3>{m.profile_info_title()}</h3>
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<label class="block">
			<span>{m.profile_given_name_label()}</span>
			<input
				id="given-name"
				class="block w-full"
				bind:value={form.fields.givenName.value}
				placeholder={m.profile_given_name_placeholder()}
				aria-label={m.profile_given_name_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.givenName.issues} />
		</label>
		<label class="block">
			<span>{m.profile_middle_name_label()}</span>
			<input
				id="middle-name"
				class="block w-full"
				bind:value={form.fields.middleName.value}
				placeholder={m.profile_middle_name_placeholder()}
				aria-label={m.profile_middle_name_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.middleName.issues} />
		</label>
		<label class="block">
			<span>{m.profile_family_name_label()}</span>
			<input
				id="family-name"
				class="block w-full"
				bind:value={form.fields.familyName.value}
				placeholder={m.profile_family_name_placeholder()}
				aria-label={m.profile_family_name_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.familyName.issues} />
		</label>
		<label class="block">
			<span>{m.profile_nickname_label()}</span>
			<input
				id="nickname"
				class="block w-full"
				bind:value={form.fields.nickname.value}
				placeholder={m.profile_nickname_placeholder()}
				aria-label={m.profile_nickname_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.nickname.issues} />
		</label>
		<label class="block">
			<span>{m.profile_website_label()}</span>
			<input
				id="website"
				class="block w-full"
				bind:value={form.fields.website.value}
				placeholder={m.profile_website_placeholder()}
				aria-label={m.profile_website_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.website.issues} />
		</label>
		<label class="block">
			<span>{m.profile_locale_label()}</span>
			<select
				id="locale"
				class="block w-full"
				bind:value={form.fields.locale.value}
				placeholder={m.profile_locale_placeholder()}
				aria-label={m.profile_locale_label()}
				disabled={readonly}
			>
				{#each supportedLocales as locale}
					<option value={locale}>{locale}</option>
				{/each}
			</select>
			<Issues issues={form.fields.locale.issues} />
		</label>
		<label class="block">
			<span>{m.profile_zone_info_label()}</span>
			<select
				id="zone-info"
				class="block w-full"
				bind:value={form.fields.zoneInfo.value}
				placeholder={m.profile_zone_info_placeholder()}
				aria-label={m.profile_zone_info_label()}
				disabled={readonly}
			>
				<option value="">{m.profile_zone_info_placeholder()}</option>
				{#each timeZones as timeZone}
					<option value={timeZone}>{timeZone}</option>
				{/each}
			</select>
			<Issues issues={form.fields.zoneInfo.issues} />
		</label>
		<label class="md:col-span-2">
			<span>{m.profile_address_label()}</span>
			<input
				id="address"
				class="block w-full"
				bind:value={form.fields.address.value}
				placeholder={m.profile_address_placeholder()}
				aria-label={m.profile_address_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.address.issues} />
		</label>
		<label class="block">
			<span>{m.profile_birthdate_label()}</span>
			<input
				id="birthdate"
				type="date"
				class="block w-full"
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
				disabled={readonly}
			/>
			<Issues issues={form.fields.birthdate.issues} />
		</label>
		<label class="block">
			<span>{m.profile_gender_label()}</span>
			<input
				id="gender"
				class="block w-full"
				bind:value={form.fields.gender.value}
				placeholder={m.profile_gender_placeholder()}
				aria-label={m.profile_gender_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.gender.issues} />
		</label>
		<label class="md:col-span-2">
			<span>{m.profile_profile_picture_label()}</span>
			<input
				id="profile-picture"
				type="url"
				class="block w-full"
				bind:value={form.fields.profilePicture.value}
				placeholder={m.profile_profile_picture_placeholder()}
				aria-label={m.profile_profile_picture_label()}
				disabled={readonly}
			/>
			<Issues issues={form.fields.profilePicture.issues} />
		</label>
	</div>
	{#if !readonly}
		<button type="submit" class="btn success mt-4">{m.profile_save_profile()}</button>
	{/if}
</form>
