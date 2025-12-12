<script lang="ts">
	import { AlertCircle, CheckCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		value?: string;
		disabled?: boolean;
		autofocus?: boolean;
		onValidChange?: (valid: boolean) => void;
	}
	let { value = '', disabled = false, autofocus = false, onValidChange }: Props = $props();

	let username = $state(value);
	const usernamePattern = /^[A-Za-z0-9_]+$/;

	const usernameErrors = $derived.by(() => {
		const errors: string[] = [];
		if (!username) errors.push(m.validation_username_required());
		if (username && !usernamePattern.test(username)) errors.push(m.validation_username_pattern());
		if (username && (username.length < 3 || username.length > 50))
			errors.push(m.validation_username_length());
		return errors;
	});
	const valid = $derived(usernameErrors.length === 0);

	$effect(() => {
		onValidChange?.(valid);
	});

	// expose getter for parent
	export function getValue() {
		return username;
	}
</script>

<div class="space-y-2">
	<label for="username-input" class="block text-sm font-medium text-gray-700 dark:text-gray-200"
		>{m.users_username()}</label
	>
	<input
		id="username-input"
		class="w-full rounded-md border border-gray-300 px-3 py-2 focus:ring-2 focus:ring-blue-500 dark:border-gray-600 {valid
			? 'valid'
			: username
				? 'invalid'
				: 'untested'}"
		placeholder="username"
		bind:value={username}
		{disabled}
		aria-invalid={!valid}
		aria-describedby="username-help"
	/>
	{#if usernameErrors.length > 0}
		<div id="username-help" class="flex items-center gap-2 text-sm text-red-700 dark:text-red-300">
			<AlertCircle class="h-4 w-4" />
			<span>{usernameErrors[0]}</span>
		</div>
	{:else if username}
		<div class="flex items-center gap-2 text-sm text-green-700 dark:text-green-300">
			<CheckCircle class="h-4 w-4" />
			<span></span>
		</div>
	{/if}
</div>
