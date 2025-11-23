<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const PasswordFormSchema = () =>
		v.pipe(
			v.object({
				password: v.pipe(v.string(), v.minLength(1)),
				passwordConfirmation: v.pipe(v.string(), v.minLength(1))
			}),
			v.forward(
				v.partialCheck(
					[['password'], ['passwordConfirmation']],
					(input) => input.password === input.passwordConfirmation,
					'Passwords do not match'
				),
				['passwordConfirmation']
			)
		);
</script>

<script lang="ts">
	import type { User } from '$lib/common/openapi/oidc/models/index';
	import { currentUserApi } from '$lib/common/openapi';
	import { handleError } from '$lib/common/errors';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';

	let { user = $bindable() }: { user: User } = $props();

	const form = createForm(PasswordFormSchema(), {
		password: '',
		passwordConfirmation: ''
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			user = await currentUserApi.updatePassword({ updateUserPassword: value });
			form.fields.password.value = '';
			form.fields.passwordConfirmation.value = '';
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3 class="text-lg font-medium">{m.profile_change_password_title()}</h3>
	<label class="block">
		<span class="text-sm font-medium">{m.profile_new_password_label()}</span>
		<input
			id="profile-new-password"
			type="password"
			class="mt-1 block w-full px-3 py-2"
			bind:value={form.fields.password.value}
			placeholder={m.profile_new_password_placeholder()}
			aria-label={m.profile_new_password_label()}
		/>
		<Issues issues={form.fields.password.issues} />
	</label>
	<label class="block">
		<span class="text-sm font-medium">{m.profile_confirm_password_label()}</span>
		<input
			id="profile-confirm-password"
			type="password"
			class="mt-1 block w-full px-3 py-2"
			bind:value={form.fields.passwordConfirmation.value}
			placeholder={m.profile_confirm_password_placeholder()}
			aria-label={m.profile_confirm_password_label()}
		/>
		<Issues issues={form.fields.passwordConfirmation.issues} />
	</label>
	<button type="submit" class="btn danger mt-4">{m.profile_change_password_button()}</button>
</form>
