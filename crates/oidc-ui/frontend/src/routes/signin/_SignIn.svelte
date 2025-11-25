<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const SignInSchema = () =>
		v.object({
			email: v.pipe(v.string(), v.nonEmpty(m.validation_email_required())),
			password: v.pipe(
				v.string(),
				v.nonEmpty(m.validation_password_required()),
				v.minLength(1, m.validation_password_min_length({ characters: 1 }))
			)
		});
</script>

<script lang="ts">
	import { resolve } from '$app/paths';
	import { signInUsernamePassword } from '$lib/common/state/currentUser.svelte';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';
	import { goto } from '$app/navigation';

	const form = createForm(SignInSchema(), {
		email: '',
		password: ''
	});

	async function onSubmit(e: SubmitEvent) {
		e.preventDefault();

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		await signInUsernamePassword(value.email, value.password);
	}
</script>

<form onsubmit={onSubmit} class="flex flex-col">
	<label class="flex flex-col">
		{m.signin_username_label()}
		<input
			type="text"
			aria-label={m.signin_username_label()}
			autocomplete="username"
			placeholder={m.signin_username_placeholder()}
			bind:value={form.fields.email.value}
		/>
		<Issues issues={form.fields.email.issues} />
	</label>
	<label class="flex flex-col">
		{m.signin_password_label()}
		<input
			aria-label={m.signin_password_label()}
			type="password"
			autocomplete="current-password"
			placeholder={m.signin_password_placeholder()}
			bind:value={form.fields.password.value}
		/>
		<Issues issues={form.fields.password.issues} />
	</label>
	<input class="btn primary mt-4" type="submit" value={m.sign_in()} />
</form>
