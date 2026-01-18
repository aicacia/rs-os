<script lang="ts">
  import Modal from '$lib/common/components/Modal.svelte';
  import { AlertCircle } from '@lucide/svelte';
  import { m } from '$lib/paraglide/messages';

  interface Props {
    username: string;
    message: string;
    onConfirm: () => Promise<void> | void;
    onCancel: () => void;
    open?: boolean;
  }
  let { username, message, onConfirm, onCancel, open = true }: Props = $props();
  let inputValue = $state('');
  let confirming = $state(false);

  const matches = $derived(() => inputValue === username);

  async function confirm() {
    if (confirming || !matches) return;
    confirming = true;
    try {
      await onConfirm();
    } finally {
      confirming = false;
    }
  }
</script>

<Modal title={m.actions_delete()} open={open} onCancel={onCancel}>
  {#snippet children()}
    <div class="flex items-start gap-3">
      <AlertCircle class="mt-1 h-6 w-6 text-red-600" />
      <div class="space-y-2 w-full">
        <p class="text-sm text-gray-700 dark:text-gray-300">{message}</p>

        <label class="block text-sm font-medium mt-2" for="confirm-username-input">
          {m.users_username()}
        </label>
        <input
          id="confirm-username-input"
          type="text"
          class="mt-1 w-full"
          bind:value={inputValue}
          placeholder={username}
        />
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <button class="btn secondary" onclick={onCancel}>{m.actions_cancel()}</button>
    <button class="btn danger" onclick={confirm} disabled={confirming || !matches}
      >{m.actions_delete()}</button
    >
  {/snippet}
</Modal>
