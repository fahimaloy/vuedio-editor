<!-- src/components/Timeline/TimelineRuler.vue -->
<template>
    <div class="relative h-8 cursor-pointer" @click="onClick">
        <div class="flex justify-between px-2 text-[11px] text-neutral-400">
            <div v-for="t in markers" :key="t" class="relative flex-1">
                <span class="absolute left-0 top-1/2 -translate-y-1/2">{{
                    formatTime(t)
                }}</span>
                <div
                    class="absolute bottom-0 left-0 h-2 w-px bg-white/10"
                ></div>
            </div>
        </div>
        <Playhead :left="playheadLeft" :duration="duration" @seek="onSeek" />
    </div>
</template>

<script setup>
import { computed } from "vue";
import Playhead from "./Playhead.vue";

const props = defineProps({
    duration: { type: Number, default: 0 },
    currentTime: { type: Number, default: 0 },
});
const emit = defineEmits(["seek"]);

const playheadLeft = computed(() => {
    const d = props.duration || 0;
    const t = props.currentTime || 0;
    return d ? (t / d) * 100 : 0;
});

const markers = computed(() => {
    const d = props.duration || 0;
    if (!d) return [];
    const marks = [],
        step = d > 120 ? 10 : d > 30 ? 5 : 1;
    for (let t = 0; t <= d; t += step) marks.push(t);
    return marks;
});

const formatTime = (s) => {
    if (!isFinite(s)) return "00:00";
    const m = Math.floor(s / 60),
        sec = Math.floor(s % 60);
    return `${String(m).padStart(2, "0")}:${String(sec).padStart(2, "0")}`;
};

const onClick = (e) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left + (e.currentTarget.scrollLeft || 0);
    const pct = Math.min(Math.max(x / rect.width, 0), 1);
    emit("seek", pct * (props.duration || 0));
};

const onSeek = (time) => emit("seek", time);
</script>
