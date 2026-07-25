import { ref, computed } from "vue";
import { defineStore } from "pinia";

export const useEditorStore = defineStore("editor", () => {
  // Project state
  const videoUrl = ref(null);
  const videoDimensions = ref({ width: 0, height: 0 });
  const duration = ref(0);
  const currentTime = ref(0);
  const isPlaying = ref(false);
  const timelineScale = ref(1); // zoom level
  const snapToGrid = ref(true);
  const snapValue = ref(0.1); // seconds

  // Tracks with items
  const tracks = ref([
    { id: "t-video-1", type: "video", label: "Video Track", items: [] },
    { id: "t-audio-1", type: "audio", label: "Audio Track", items: [] },
    { id: "t-text-1", type: "text", label: "Text Track", items: [] },
  ]);

  // Selection state
  const selected = ref([]); // [{ trackId, itemId }]

  // Filters
  const filters = ref([]);

  // History for undo/redo
  const history = ref([]);
  const historyIndex = ref(-1);
  const maxHistorySize = 50;

  // Media library
  const mediaLibrary = ref([]);

  // Computed helpers
  const currentItem = computed(() => {
    if (!selected.value.length) return null;
    const { trackId, itemId } = selected.value[0];
    const track = tracks.value.find((t) => t.id === trackId);
    return track?.items.find((i) => i.id === itemId) || null;
  });

  const playheadPosition = computed(() => {
    const d = duration.value || 0;
    return d ? (currentTime.value / d) * 100 : 0;
  });

  // Generate unique ID
  const newId = () => {
    return crypto?.randomUUID
      ? crypto.randomUUID()
      : String(Date.now() + Math.random());
  };

  // History management
  const saveState = () => {
    const state = {
      tracks: JSON.parse(JSON.stringify(tracks.value)),
      filters: JSON.parse(JSON.stringify(filters.value)),
      mediaLibrary: JSON.parse(JSON.stringify(mediaLibrary.value)),
    };
    // Truncate history after current index
    history.value = history.value.slice(0, historyIndex.value + 1);
    history.value.push(state);
    if (history.value.length > maxHistorySize) {
      history.value.shift();
    }
    historyIndex.value = history.value.length - 1;
  };

  const undo = () => {
    if (historyIndex.value <= 0) return;
    historyIndex.value--;
    const state = history.value[historyIndex.value];
    restoreState(state);
  };

  const redo = () => {
    if (historyIndex.value >= history.value.length - 1) return;
    historyIndex.value++;
    const state = history.value[historyIndex.value];
    restoreState(state);
  };

  const restoreState = (state) => {
    tracks.value = state.tracks;
    filters.value = state.filters;
    mediaLibrary.value = state.mediaLibrary;
  };

  // Track actions
  const addTrack = (type) => {
    const count = tracks.value.filter((t) => t.type === type).length;
    const track = {
      id: `t-${type}-${Date.now()}`,
      type,
      label: `${type.charAt(0).toUpperCase() + type.slice(1)} ${count + 1}`,
      items: [],
    };
    tracks.value.push(track);
    saveState();
  };

  const removeTrack = (trackId) => {
    tracks.value = tracks.value.filter((t) => t.id !== trackId);
    saveState();
  };

  // Item actions
  const findTrack = (id) => tracks.value.find((t) => t.id === id);
  const findItem = (trackId, itemId) => {
    const track = findTrack(trackId);
    return track?.items.find((i) => i.id === itemId);
  };

  const addItem = (trackId, item) => {
    const track = findTrack(trackId);
    if (!track) return;
    track.items.push({ ...item, id: newId() });
    track.items.sort((a, b) => a.start - b.start);
    saveState();
  };

  const updateItem = (trackId, itemId, patch) => {
    const track = findTrack(trackId);
    if (!track) return;
    const idx = track.items.findIndex((i) => i.id === itemId);
    if (idx < 0) return;
    track.items[idx] = { ...track.items[idx], ...patch };
    track.items.sort((a, b) => a.start - b.start);
  };

  const removeItem = (trackId, itemId) => {
    const track = findTrack(trackId);
    if (!track) return;
    track.items = track.items.filter((i) => i.id !== itemId);
    selected.value = selected.value.filter(
      (s) => !(s.trackId === trackId && s.itemId === itemId),
    );
    saveState();
  };

  // Selection actions
  const select = (trackId, itemId, multi = false) => {
    const key = { trackId, itemId };
    if (multi) {
      if (isSelected(trackId, itemId)) {
        selected.value = selected.value.filter(
          (s) => !(s.trackId === trackId && s.itemId === itemId),
        );
      } else {
        selected.value = [...selected.value, key];
      }
    } else {
      if (isSelected(trackId, itemId) && selected.value.length === 1) {
        selected.value = [];
      } else {
        selected.value = [key];
      }
    }
  };

  const isSelected = (trackId, itemId) => {
    return selected.value.some(
      (s) => s.trackId === trackId && s.itemId === itemId,
    );
  };

  const clearSelection = () => {
    selected.value = [];
  };

  // Filter actions
  const addFilter = (filter) => {
    filters.value = [...filters.value, { ...filter, _id: newId() }];
    saveState();
  };

  const removeFilter = (index) => {
    filters.value = filters.value.filter((_, i) => i !== index);
  };

  const updateFilter = (index, key, value) => {
    const f = filters.value[index];
    if (!f) return;
    filters.value[index] = {
      ...f,
      params: { ...f.params, [key]: { ...f.params[key], value } },
    };
  };

  // Time/format helpers
  const formatTime = (seconds) => {
    if (!isFinite(seconds)) return "00:00";
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  };

  const timeToPixels = (time, containerWidth) => {
    return (time / (duration.value || 1)) * containerWidth;
  };

  const pixelsToTime = (pixels, containerWidth) => {
    return (pixels / containerWidth) * (duration.value || 1);
  };

  // Snap time to grid
  const snapTime = (time) => {
    if (!snapToGrid.value) return time;
    const grid = snapValue.value;
    return Math.round(time / grid) * grid;
  };

  // Media library
  const addMedia = (file, url, type) => {
    mediaLibrary.value.push({
      id: newId(),
      name: file.name,
      url,
      type,
      file,
    });
  };

  const removeMedia = (id) => {
    mediaLibrary.value = mediaLibrary.value.filter((m) => m.id !== id);
  };

  return {
    // State
    videoUrl,
    videoDimensions,
    duration,
    currentTime,
    isPlaying,
    timelineScale,
    snapToGrid,
    snapValue,
    tracks,
    selected,
    filters,
    mediaLibrary,

    // Computed
    currentItem,
    playheadPosition,

    // Actions
    newId,
    saveState,
    undo,
    redo,
    addTrack,
    removeTrack,
    findTrack,
    findItem,
    addItem,
    updateItem,
    removeItem,
    select,
    isSelected,
    clearSelection,
    addFilter,
    removeFilter,
    updateFilter,
    formatTime,
    timeToPixels,
    pixelsToTime,
    snapTime,
    addMedia,
    removeMedia,
  };
});
