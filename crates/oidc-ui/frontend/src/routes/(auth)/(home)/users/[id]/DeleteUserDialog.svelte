<script lang="ts">
	import { AlertCircle } from '@lucide/svelte';
	import * as m from '$lib/paraglide/messages/_index.js';

	interface Props {
		username: string;
		message: string;
		onConfirm: () => Promise<void> | void;
		onCancel: () => void;
		open?: boolean;
	}
	let { username, message, onConfirm, onCancel, open = true }: Props = $props();
	let confirming = $state(false);

	async function confirm() {
		if (confirming) return;
		confirming = true;
		try {
			await onConfirm();
		} finally {
			confirming = false;
		}
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center p-4"
		role="dialog"
		aria-modal="true"
		aria-labelledby="delete-title"
	>
		<div
			class="absolute inset-0 bg-black/40"
			role="button"
			tabindex="0"
			onclick={onCancel}
			onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onCancel()}
		></div>
		<div class="relative w-full max-w-md rounded-lg bg-white p-6 shadow-lg dark:bg-gray-900">
			<div class="flex items-start gap-3">
				<AlertCircle class="mt-1 h-6 w-6 text-red-600" />
				<div class="space-y-1">
					<h3 id="delete-title" class="text-lg font-semibold">
						{(m as any).actions_delete()}
						{username}
					</h3>
					<p class="text-sm text-gray-700 dark:text-gray-300">{message}</p>
				</div>
			</div>
			<div class="mt-4 flex justify-end gap-2">
				<button class="btn secondary" onclick={onCancel}>{(m as any).actions_cancel()}</button>
				<button class="btn danger" onclick={confirm} disabled={confirming}
					>{(m as any).actions_delete()}</button
				>
			</div>
		</div>
	</div>
{/if}
