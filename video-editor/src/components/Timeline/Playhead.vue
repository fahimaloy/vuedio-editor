<!-- src/components/Timeline/Playhead.vue -->
<template>
    <div
        class="absolute inset-y-0 w-0.5 bg-gradient-to-b from-rose-500 to-rose-300 shadow-[0_0_20px_rgba(244,63,94,0.6)] cursor-col-resize"
        :style="{ left: left + '%' }"
        @mousedown="onMouseDown"
    >
        <div
            class="absolute -top-2 -bottom-2 -left-1 -right-1 cursor-col-resize"
        ></div>
    </div>
</template>

<script setup>
import { ref } from "vue";

const props = defineProps({
    left: { type: Number, default: 0 },
    duration: { type: Number, default: 0 },
});
const emit = defineEmits(["seek"]);

let dragState = null;

const onMouseDown = (e) => {
    if (e.button !== 0) return;
    const rect = e.currentTarget.parentElement?.getBoundingClientRect();
    if (!rect) return;

    dragState = {
        startX: e.clientX,
        rect,
        scrollLeft: rect.scrollLeft || 0,
    };

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onEnd);
};

const onMove = (e) => {
    if (!dragState) return;
    const x = e.clientX - dragState.rect.left + dragState.scrollLeft;
    const pct = Math.min(Math.max(x / dragState.rect.width, 0), 1);
    emit("seek", pct * (props.duration || 0));
};

const onEnd = () => {
    dragState = null;
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onEnd);
};
</script>
