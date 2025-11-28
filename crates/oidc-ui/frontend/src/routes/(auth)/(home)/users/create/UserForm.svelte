<script lang="ts">
	import { Loader2 } from '@lucide/svelte';
	import UserFormFields from './UserFormFields.svelte';
	import { m } from '$lib/paraglide/messages';

	interface Props {
		initial?: { username?: string };
		mode?: 'create' | 'edit';
		onSubmit: (values: { username: string }) => Promise<void>;
		onCancel?: () => void;
	}
	let { initial = {}, mode = 'create', onSubmit, onCancel }: Props = $props();

	let canSubmit = $state(false);
	let submitting = $state(false);
	let fieldsRef: any;

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!canSubmit || submitting) return;
		submitting = true;
		try {
			const username = fieldsRef?.getValue();
			await onSubmit({ username });
		} finally {
			submitting = false;
		}
	}
</script>

<form class="space-y-4" onsubmit={handleSubmit}>
	<h2 class="text-2xl font-semibold">
		{mode === 'create' ? m.users_create_title() : m.users_edit_title()}
	</h2>

	<UserFormFields
		bind:this={fieldsRef}
		value={initial.username ?? ''}
		onValidChange={(v) => (canSubmit = v)}
		autofocus
	/>

	<div class="flex gap-2">
		<button
			type="submit"
			class="btn primary disabled:opacity-50"
			disabled={!canSubmit || submitting}
		>
			{#if submitting}
				<Loader2 class="mr-2 inline h-4 w-4 animate-spin" />
			{/if}
			{mode === 'create' ? m.actions_create() : m.actions_save()}
		</button>
		{#if onCancel}
			<button type="button" class="btn secondary" onclick={onCancel}>
				{m.actions_cancel()}
			</button>
		{/if}
	</div>
</form>
