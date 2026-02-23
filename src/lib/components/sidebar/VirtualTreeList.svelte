<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';

  let {
    items,
    threshold = 100,
    rowHeight = 26,
    scrollContainer = null,
    children
  }: {
    items: T[];
    threshold?: number;
    rowHeight?: number;
    scrollContainer?: HTMLElement | null;
    children: Snippet<[T, number]>;
  } = $props();

  let containerEl: HTMLElement | undefined = $state();
  let scrollTop = $state(0);
  let containerHeight = $state(0);

  const BUFFER_ROWS = 10;

  let useVirtual = $derived(items.length > threshold);

  // Track scroll position from the scroll container
  $effect(() => {
    if (!useVirtual || !scrollContainer) return;
    const el = scrollContainer;

    function onScroll() {
      scrollTop = el.scrollTop;
      containerHeight = el.clientHeight;
    }

    // Initial measurement
    scrollTop = el.scrollTop;
    containerHeight = el.clientHeight;

    el.addEventListener('scroll', onScroll, { passive: true });
    return () => el.removeEventListener('scroll', onScroll);
  });

  let totalHeight = $derived(items.length * rowHeight);

  let visibleRange = $derived.by(() => {
    if (!useVirtual || !containerEl) return { start: 0, end: items.length };

    // Get the offset of our container relative to the scroll container
    const containerRect = containerEl.getBoundingClientRect();
    const scrollRect = scrollContainer?.getBoundingClientRect();
    if (!scrollRect) return { start: 0, end: items.length };

    const offsetTop = containerRect.top - scrollRect.top + scrollTop;
    const viewStart = scrollTop - offsetTop;
    const viewEnd = viewStart + containerHeight;

    const startIndex = Math.max(0, Math.floor(viewStart / rowHeight) - BUFFER_ROWS);
    const endIndex = Math.min(items.length, Math.ceil(viewEnd / rowHeight) + BUFFER_ROWS);

    return { start: startIndex, end: endIndex };
  });

  let visibleItems = $derived(
    useVirtual
      ? items.slice(visibleRange.start, visibleRange.end)
      : items
  );

  let offsetY = $derived(useVirtual ? visibleRange.start * rowHeight : 0);
</script>

{#if useVirtual}
  <div class="virtual-list" bind:this={containerEl} style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({offsetY}px);">
      {#each visibleItems as item, i (visibleRange.start + i)}
        {@render children(item, visibleRange.start + i)}
      {/each}
    </div>
  </div>
{:else}
  {#each items as item, i (i)}
    {@render children(item, i)}
  {/each}
{/if}
