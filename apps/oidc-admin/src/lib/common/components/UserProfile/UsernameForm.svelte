<script lang="ts" module>
	import * as v from 'valibot';
	import { m } from '$lib/paraglide/messages';

	const UsernameFormSchema = () =>
		v.object({
			username: v.pipe(v.string(), v.minLength(1))
		});

	export interface UsernameFormProps {
		username: string;
		onUpdate: (username: string) => Promise<void>;
		readonly?: boolean;
	}
</script>

<script lang="ts">
	import { createForm } from '@aicacia/svelte-forms';
	import Issues from '$lib/common/components/Issues.svelte';
	import { handleError } from '$lib/common/errors';

	let { username = $bindable(), onUpdate, readonly = false }: UsernameFormProps = $props();

	const form = createForm(UsernameFormSchema(), {
		username: username ?? ''
	});

	$effect(() => {
		form.fields.username.value = username ?? '';
	});

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (readonly) return;

		const [value, err] = await form.validate();

		if (err) {
			return;
		}

		try {
			await onUpdate(value.username);
			username = value.username;
		} catch (e) {
			handleError(e);
		}
	}
</script>

<form onsubmit={submit} class="card">
	<h3>{m.profile_username_title()}</h3>
	<label class="block">
		<span>{m.profile_username_label()}</span>
		<input
			id="username"
			class="block w-full"
			bind:value={form.fields.username.value}
			placeholder={m.profile_username_placeholder()}
			aria-label={m.profile_username_label()}
			disabled={readonly}
		/>
		<Issues issues={form.fields.username.issues} />
	</label>
	{#if !readonly}
		<button type="submit" class="btn primary mt-4">{m.profile_save_username()}</button>
	{/if}
</form>
