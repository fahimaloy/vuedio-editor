<!-- src/components/Timeline/TimelineItem.vue -->
<template>
    <div
        class="absolute top-2 h-16 overflow-hidden rounded-md border p-2 text-[11px] shadow backdrop-blur border-white/10"
        :class="
            selected
                ? 'bg-white/20 ring-2 ring-violet-400/60 text-neutral-50'
                : 'bg-white/10 hover:bg-white/15 text-neutral-100'
        "
        :style="{ left: left + '%', width: width + '%' }"
        draggable="true"
        @dragstart="$emit('dragstart', $event)"
        @click.stop="$emit('select', $event)"
    >
        <div class="flex items-center gap-2">
            <span
                class="inline-block h-2 w-2 rounded-full"
                :class="colorClass"
            ></span>
            <span class="truncate">{{ label }}</span>
            <button
                class="ml-auto rounded border border-white/10 bg-white/10 px-1.5 py-0.5 text-[10px] text-rose-200 hover:bg-white/15"
                @click.stop="$emit('remove')"
            >
                Remove
            </button>
        </div>
        <div class="mt-1 text-[10px] text-neutral-300/80">
            {{ formatTime(start) }}–{{ formatTime(end) }}
        </div>

        <!-- resize handles -->
        <div
            class="absolute inset-y-0 left-0 w-2 cursor-col-resize"
            @mousedown.prevent="$emit('resize', 'left', $event)"
        ></div>
        <div
            class="absolute inset-y-0 right-0 w-2 cursor-col-resize"
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
});

const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const m = Math.floor(s / 60),
        sec = Math.floor(s % 60);
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
};
</script>
