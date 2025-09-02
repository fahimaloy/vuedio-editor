
<!-- src/components/Inspector.vue -->
<template>
  <aside class="rounded-xl border border-white/10 bg-white/[0.03] p-4">
    <div class="mb-3 flex items-center justify-between">
      <h3 class="text-sm font-semibold text-neutral-200">Inspector</h3>
      <button
        v-if="current"
        @click="resetToDefaults"
        class="rounded-md border border-white/10 bg-white/5 px-2 py-1 text-xs text-neutral-200 hover:bg-white/10"
      >
        Reset
      </button>
    </div>

    <div v-if="!current" class="text-xs text-neutral-500">
      Select an item to edit its properties.
    </div>

    <div v-else class="space-y-3">
      <div class="grid grid-cols-2 gap-2">
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">Label</label>
          <input v-model="draft.label" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">Z Index</label>
          <input type="number" v-model.number="draft.z" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
      </div>

      <div class="grid grid-cols-2 gap-2">
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">Start (s)</label>
          <input type="number" step="0.01" v-model.number="draft.start" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">End (s)</label>
          <input type="number" step="0.01" v-model.number="draft.end" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
      </div>

      <div class="grid grid-cols-4 gap-2">
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">X</label>
          <input type="number" step="0.01" v-model.number="draft.x" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">Y</label>
          <input type="number" step="0.01" v-model.number="draft.y" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">W</label>
          <input type="number" step="0.01" v-model.number="draft.w" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">H</label>
          <input type="number" step="0.01" v-model.number="draft.h" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
        </div>
      </div>

      <!-- Text-only fields -->
      <div v-if="current.kind==='text'" class="space-y-2 pt-2 border-t border-white/10">
        <div>
          <label class="block text-[11px] text-neutral-400 mb-1">Text</label>
          <textarea v-model="draft.text" rows="3" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none"></textarea>
        </div>
        <div class="grid grid-cols-3 gap-2">
          <div>
            <label class="block text-[11px] text-neutral-400 mb-1">Color</label>
            <input type="color" v-model="draft.color" class="h-8 w-full rounded-md border border-white/10 bg-neutral-900" />
          </div>
          <div>
            <label class="block text-[11px] text-neutral-400 mb-1">Font Size</label>
            <input type="number" v-model.number="draft.fontSize" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
          </div>
          <div>
            <label class="block text-[11px] text-neutral-400 mb-1">Font Family</label>
            <input v-model="draft.fontFamily" class="w-full rounded-md border border-white/10 bg-neutral-900 px-2 py-1 text-xs text-neutral-100 outline-none" />
          </div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-2">
        <button @click="emitRemove" class="rounded-md border border-white/10 bg-white/5 px-3 py-1.5 text-xs text-rose-300 hover:bg-white/10">Remove</button>
      </div>
    </div>
  </aside>
</template>

<script setup>
import { computed, reactive, watch } from 'vue'

const props = defineProps({
  selected: { type: Array, default: () => [] }, // [{trackId,itemId}]
  tracks:   { type: Array, default: () => [] },
  duration: { type: Number, default: 0 }
})
const emit = defineEmits(['update-item','remove','reset-item'])

const current = computed(() => {
  if (!props.selected?.length) return null
  const { trackId, itemId } = props.selected[0]
  const t = props.tracks.find(t=>t.id===trackId)
  return t?.items.find(i=>i.id===itemId) || null
})

const draft = reactive({})
const fillDraft = (c) => {
  for (const k of Object.keys(draft)) delete draft[k]
  if (!c) return
  Object.assign(draft, JSON.parse(JSON.stringify(c)))
}
watch(current, (c) => fillDraft(c), { immediate: true })

// Realtime update: whenever draft changes, emit patch
watch(draft, (val) => {
  if (!current.value) return
  const { trackId, itemId } = props.selected[0]
  // Clamp geometry here (0..1 and min size) to prevent hiding
  const patch = { ...val }
  if ('x' in patch || 'y' in patch || 'w' in patch || 'h' in patch) {
    const x = patch.x ?? current.value.x ?? 0
    const y = patch.y ?? current.value.y ?? 0
    const w = Math.min(Math.max(patch.w ?? current.value.w ?? 0.1, 0.02), 1)
    const h = Math.min(Math.max(patch.h ?? current.value.h ?? 0.1, 0.02), 1)
    patch.x = Math.min(Math.max(x, 0), 1 - w)
    patch.y = Math.min(Math.max(y, 0), 1 - h)
    patch.w = w; patch.h = h
  }
  emit('update-item', { trackId, itemId, patch })
}, { deep: true })

const emitRemove = () => {
  if (!current.value) return
  emit('remove', { trackId: props.selected[0].trackId, itemId: props.selected[0].itemId })
}

const resetToDefaults = () => {
  if (!current.value) return
  emit('reset-item', { trackId: props.selected[0].trackId, itemId: props.selected[0].itemId })
}
</script>
