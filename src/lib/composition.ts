import type { AppSettings, LayerType, OverlayComposition, OverlayLayer } from "./types";

export function newLayerId(): string {
  return crypto.randomUUID();
}

export function createLayer(
  type: LayerType,
  partial?: Partial<OverlayLayer>,
): OverlayLayer {
  const defaults: Record<LayerType, Partial<OverlayLayer>> = {
    image: {
      name: "Image",
      x: 20,
      y: 50,
      width: 48,
      icon_id: "bell",
      image_path: null,
    },
    title: { name: "Event title", x: 50, y: 35, width: 220, height: 52 },
    countdown: { name: "Countdown", x: 50, y: 65 },
    text: { name: "Text", x: 50, y: 50, text_content: "Custom text", font_size: null, width: 140, height: 40 },
  };

  return {
    id: newLayerId(),
    type,
    visible: true,
    z_index: 0,
    ...defaults[type],
    ...partial,
  } as OverlayLayer;
}

export function createDefaultComposition(settings?: Partial<AppSettings>): OverlayComposition {
  const iconId = settings?.icon_id ?? "bell";
  const iconSize = settings?.icon_size ?? 48;
  const showTitle = settings?.show_title ?? true;
  const showCountdown = settings?.show_countdown ?? true;

  const layers: OverlayLayer[] = [
    {
      ...createLayer("image", { name: "Main icon", x: 15, y: 50, z_index: 0 }),
      icon_id: iconId,
      image_path: settings?.custom_icon_path ?? null,
      width: iconSize,
    },
  ];

  if (showTitle) {
    layers.push({
      ...createLayer("title", { x: 48, y: 38, z_index: 1 }),
      visible: true,
    });
  }

  if (showCountdown) {
    layers.push({
      ...createLayer("countdown", { x: 48, y: 62, z_index: 2 }),
      visible: true,
    });
  }

  return {
    canvas_width: 320,
    canvas_height: 88,
    layers: normalizeLayerZIndex(layers),
  };
}

export function layoutToComposition(
  layout: AppSettings["layout"],
  settings: AppSettings,
): OverlayComposition {
  if (!layout) {
    return createDefaultComposition(settings);
  }

  const layers: OverlayLayer[] = [
    {
      id: newLayerId(),
      type: "image",
      name: "Main icon",
      visible: true,
      x: layout.icon.x,
      y: layout.icon.y,
      z_index: 0,
      width: settings.icon_size,
      icon_id: settings.icon_id,
      image_path: settings.custom_icon_path,
    },
  ];

  if (settings.show_title) {
    layers.push({
      id: newLayerId(),
      type: "title",
      name: "Event title",
      visible: true,
      x: layout.title.x,
      y: layout.title.y,
      z_index: 1,
    });
  }

  if (settings.show_countdown) {
    layers.push({
      id: newLayerId(),
      type: "countdown",
      name: "Countdown",
      visible: true,
      x: layout.countdown.x,
      y: layout.countdown.y,
      z_index: 2,
    });
  }

  return {
    canvas_width: layout.bubble_width,
    canvas_height: layout.bubble_height,
    layers,
  };
}

export function normalizeComposition(composition: OverlayComposition): OverlayComposition {
  return {
    ...composition,
    layers: normalizeLayerZIndex(
      composition.layers.map((layer, index) => ({
        ...layer,
        z_index: layer.z_index ?? index,
      })),
    ),
  };
}

export function normalizeSettings(settings: AppSettings): AppSettings {
  const composition =
    settings.composition?.layers?.length > 0
      ? normalizeComposition(settings.composition)
      : settings.layout
        ? layoutToComposition(settings.layout, settings)
        : createDefaultComposition(settings);

  return {
    ...settings,
    composition,
    dedupe_reminders: settings.dedupe_reminders ?? true,
    quiet_hours_enabled: settings.quiet_hours_enabled ?? false,
    quiet_hours_start: settings.quiet_hours_start ?? "22:00",
    quiet_hours_end: settings.quiet_hours_end ?? "07:00",
    monitor_target: settings.monitor_target ?? "primary",
    launch_at_login: settings.launch_at_login ?? false,
    sound_enabled: settings.sound_enabled ?? true,
    auto_contrast_text: settings.auto_contrast_text ?? false,
    push_sync_enabled: settings.push_sync_enabled ?? true,
  };
}

export function updateComposition(
  settings: AppSettings,
  composition: OverlayComposition,
): AppSettings {
  return normalizeSettings({ ...settings, composition: normalizeComposition(composition) });
}

export function updateLayerInComposition(
  composition: OverlayComposition,
  layerId: string,
  patch: Partial<OverlayLayer>,
): OverlayComposition {
  return normalizeComposition({
    ...composition,
    layers: composition.layers.map((layer) =>
      layer.id === layerId ? { ...layer, ...patch } : layer,
    ),
  });
}

export function addLayerToComposition(
  composition: OverlayComposition,
  type: LayerType,
): OverlayComposition {
  const maxZ = composition.layers.reduce((max, layer) => Math.max(max, layer.z_index), -1);
  const layer = createLayer(type, { z_index: maxZ + 1 });
  return normalizeComposition({
    ...composition,
    layers: [...composition.layers, layer],
  });
}

export function removeLayerFromComposition(
  composition: OverlayComposition,
  layerId: string,
): OverlayComposition {
  return normalizeComposition({
    ...composition,
    layers: composition.layers.filter((layer) => layer.id !== layerId),
  });
}

export function moveLayerInComposition(
  composition: OverlayComposition,
  layerId: string,
  direction: "up" | "down",
): OverlayComposition {
  const sorted = [...composition.layers].sort((a, b) => a.z_index - b.z_index);
  const index = sorted.findIndex((layer) => layer.id === layerId);
  if (index < 0) {
    return composition;
  }

  const swapIndex = direction === "up" ? index + 1 : index - 1;
  if (swapIndex < 0 || swapIndex >= sorted.length) {
    return composition;
  }

  const next = [...sorted];
  [next[index], next[swapIndex]] = [next[swapIndex], next[index]];
  return normalizeComposition({
    ...composition,
    layers: next.map((layer, z) => ({ ...layer, z_index: z })),
  });
}

export function syncPrimaryImageLayer(
  settings: AppSettings,
  patch: { icon_id?: string; image_path?: string | null; width?: number },
): AppSettings {
  const imageLayers = settings.composition.layers.filter((layer) => layer.type === "image");
  if (imageLayers.length === 0) {
    const composition = addLayerToComposition(settings.composition, "image");
    const newLayer = composition.layers.find((layer) => layer.type === "image");
    if (!newLayer) {
      return settings;
    }
    return updateComposition(settings, {
      ...composition,
      layers: composition.layers.map((layer) =>
        layer.id === newLayer.id ? { ...layer, ...patch } : layer,
      ),
    });
  }

  const primary = imageLayers.sort((a, b) => a.z_index - b.z_index)[0];
  return updateComposition(
    settings,
    updateLayerInComposition(settings.composition, primary.id, patch),
  );
}

export function setLayerVisibilityByType(
  settings: AppSettings,
  type: "title" | "countdown",
  visible: boolean,
): AppSettings {
  const flagKey = type === "title" ? "show_title" : "show_countdown";
  let next = { ...settings, [flagKey]: visible };

  const hasLayer = next.composition.layers.some((layer) => layer.type === type);
  if (!hasLayer && visible) {
    next = updateComposition(next, addLayerToComposition(next.composition, type));
  }

  return updateComposition(next, {
    ...next.composition,
    layers: next.composition.layers.map((layer) =>
      layer.type === type ? { ...layer, visible } : layer,
    ),
  });
}

function normalizeLayerZIndex(layers: OverlayLayer[]): OverlayLayer[] {
  return [...layers]
    .sort((a, b) => a.z_index - b.z_index)
    .map((layer, index) => ({ ...layer, z_index: index }));
}

export function sortedLayers(composition: OverlayComposition): OverlayLayer[] {
  return [...composition.layers].sort((a, b) => a.z_index - b.z_index);
}
