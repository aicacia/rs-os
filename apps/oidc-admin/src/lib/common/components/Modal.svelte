<script lang="ts">
  interface Props {
    title?: string;
    open?: boolean;
    onCancel: () => void;
    closeOnBackdrop?: boolean;
    size?: 'sm' | 'md' | 'lg';
    children?: any;
    footer?: any;
  }

  let {
    title,
    open = true,
    onCancel,
    closeOnBackdrop = true,
    size = 'md',
    children,
    footer
  }: Props = $props();

  const sizeClass = $derived(() => {
    switch (size) {
      case 'sm':
        return 'max-w-sm';
      case 'lg':
        return 'max-w-2xl';
      case 'md':
      default:
        return 'max-w-md';
    }
  });

  const titleId = 'modal-title';
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? titleId : undefined}
  >
    <div
      class="absolute inset-0 bg-black/40"
      role="button"
      tabindex="0"
      onclick={() => closeOnBackdrop && onCancel()}
      onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && closeOnBackdrop && onCancel()}
    ></div>
    <div class={`relative w-full ${sizeClass} rounded-lg bg-white p-6 shadow-lg dark:bg-gray-900`}>
      {#if title}
        <h3 id={titleId} class="text-lg font-semibold mb-2">{title}</h3>
      {/if}
      {#if children}
        {@render children()}
      {/if}
      <div class="mt-4 flex justify-end gap-2">
        {#if footer}
          {@render footer()}
        {/if}
      </div>
    </div>
  </div>
{/if}
