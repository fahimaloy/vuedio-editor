<!-- src/components/Timeline/TimelineItem.vue -->
<template>
    <div
        class="absolute top-1 h-14 overflow-hidden rounded-md border border-white/10 text-[10px] shadow backdrop-blur flex items-center"
        :class="
            selected
                ? 'bg-white/20 ring-2 ring-violet-400/60 text-neutral-50'
                : 'bg-white/10 hover:bg-white/15 text-neutral-100'
        "
        :style="{ left: left + '%', width: width + '%', zIndex: item?.z ?? 0 }"
        draggable="true"
        role="button"
        :aria-selected="selected"
        :aria-label="`${item?.label || 'Item'} from ${formatTime(start)} to ${formatTime(end)}`"
        @dragstart="$emit('dragstart', $event)"
        @click.stop="$emit('select', $event)"
    >
        <!-- Thumbnail or color indicator -->
        <div
            v-if="thumbnail"
            class="h-full aspect-video bg-black flex-shrink-0 rounded-l-md overflow-hidden"
        >
            <img
                :src="thumbnail"
                class="h-full w-full object-cover"
                alt="thumbnail"
            />
        </div>
        <span
            v-else
            class="inline-block h-2 w-2 rounded-full flex-shrink-0 ml-2"
            :class="colorClass"
        ></span>

        <div class="flex-1 min-w-0 px-2">
            <div class="truncate font-medium">{{ label }}</div>
            <div class="text-[9px] text-neutral-300/80">
                {{ formatTime(start) }}–{{ formatTime(end) }}
            </div>
        </div>

        <button
            class="rounded border border-white/10 bg-white/10 px-1 py-0.5 text-[9px] text-rose-200 hover:bg-white/15 mr-1"
            @click.stop="$emit('remove')"
        >
            ×
        </button>

        <!-- resize handles -->
        <div
            class="absolute inset-y-0 left-0 w-1.5 cursor-col-resize hover:bg-white/20"
            @mousedown.prevent="$emit('resize', 'left', $event)"
        ></div>
        <div
            class="absolute inset-y-0 right-0 w-1.5 cursor-col-resize hover:bg-white/20"
            @mousedown.prevent="$emit('resize', 'right', $event)"
        ></div>
    </div>
</template>

<script setup>
const props = defineProps({
    item: Object,
    selected: Boolean,
    left: { type: String, default: "0" },
    width: { type: String, default: "0" },
    start: Number,
    end: Number,
    label: String,
    colorClass: String,
    thumbnail: String,
});

const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const m = Math.floor(s / 60),
        sec = Math.floor(s % 60);
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
};
</script>
