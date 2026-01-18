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

	export interface PasswordFormProps {
		onUpdate: (password: string) => Promise<void>;
		readonly?: boolean;
	}
</script>

<script lang="ts">
	import { createForm } from '@aicacia/svelte-forms';
	import Issues from '$lib/common/components/Issues.svelte';
	import { handleError } from '$lib/common/errors';

	let { onUpdate, readonly = false }: PasswordFormProps = $props();

	const form = createForm(PasswordFormSchema(), {
		password: '',
		passwordConfirmation: ''
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (readonly) return;

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			await onUpdate(value.password);
			form.reset();
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3>{m.profile_change_password_title()}</h3>
	<label class="block">
		<span>{m.profile_new_password_label()}</span>
		<input
			id="new-password"
			type="password"
			class="block w-full"
			bind:value={form.fields.password.value}
			placeholder={m.profile_new_password_placeholder()}
			aria-label={m.profile_new_password_label()}
			disabled={readonly}
		/>
		<Issues issues={form.fields.password.issues} />
	</label>
	<label class="block">
		<span>{m.profile_confirm_password_label()}</span>
		<input
			id="confirm-password"
			type="password"
			class="block w-full"
			bind:value={form.fields.passwordConfirmation.value}
			placeholder={m.profile_confirm_password_placeholder()}
			aria-label={m.profile_confirm_password_label()}
			disabled={readonly}
		/>
		<Issues issues={form.fields.passwordConfirmation.issues} />
	</label>
	{#if !readonly}
		<button type="submit" class="btn danger mt-4">{m.profile_change_password_button()}</button>
	{/if}
</form>
