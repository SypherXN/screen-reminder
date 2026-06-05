import type { CSSProperties, PointerEvent as ReactPointerEvent } from "react";
import { iconEmoji } from "../components/AppearancePanel";
import { sortedLayers } from "../lib/composition";
import { useTimeUntil } from "../lib/timeUntil";
import type { AppSettings, OverlayComposition, OverlayLayer } from "../lib/types";

interface ReminderBubbleProps {
  settings: AppSettings;
  title: string;
  location?: string | null;
  startTime: string;
  className?: string;
  style?: CSSProperties;
  onClick?: () => void;
  interactive?: boolean;
  editMode?: boolean;
  selectedLayerId?: string | null;
  onCompositionChange?: (composition: OverlayComposition) => void;
  onSelectLayer?: (layerId: string | null) => void;
}

export function ReminderBubble({
  settings,
  title,
  location,
  startTime,
  className = "",
  style,
  onClick,
  interactive = true,
  editMode = false,
  selectedLayerId = null,
  onCompositionChange,
  onSelectLayer,
}: ReminderBubbleProps) {
  const countdown = useTimeUntil(settings.show_countdown ? startTime : null);
  const composition = settings.composition;
  const layers = sortedLayers(composition).filter((layer) => layer.visible);

  const startDrag = (layer: OverlayLayer, event: ReactPointerEvent<HTMLSpanElement>) => {
    if (!editMode || !onCompositionChange) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    onSelectLayer?.(layer.id);

    const canvas = event.currentTarget.closest(".reminder-canvas") as HTMLElement | null;
    if (!canvas) {
      return;
    }

    const move = (moveEvent: PointerEvent) => {
      const rect = canvas.getBoundingClientRect();
      const x = clampPercent(((moveEvent.clientX - rect.left) / rect.width) * 100);
      const y = clampPercent(((moveEvent.clientY - rect.top) / rect.height) * 100);
      onCompositionChange({
        ...composition,
        layers: composition.layers.map((entry) =>
          entry.id === layer.id ? { ...entry, x, y } : entry,
        ),
      });
    };

    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  const content = layers.map((layer) => (
    <LayerElement
      key={layer.id}
      layer={layer}
      selected={editMode && selectedLayerId === layer.id}
      editMode={editMode}
      settings={settings}
      title={title}
      location={location}
      countdown={countdown}
      onPointerDown={(event) => startDrag(layer, event)}
      onSelect={() => onSelectLayer?.(layer.id)}
    />
  ));

  const canvasStyle: CSSProperties = {
    width: composition.canvas_width,
    height: composition.canvas_height,
    fontFamily: settings.font_family,
    fontSize: settings.font_size,
    color: settings.font_color,
    ...style,
  };

  if (interactive && !editMode) {
    return (
      <button
        type="button"
        className={`reminder-canvas reminder-canvas--interactive ${className}`}
        style={canvasStyle}
        onClick={onClick}
      >
        {content}
      </button>
    );
  }

  return (
    <div
      className={`reminder-canvas ${editMode ? "reminder-canvas--edit" : ""} ${className}`}
      style={canvasStyle}
      onClick={() => editMode && onSelectLayer?.(null)}
    >
      {content}
    </div>
  );
}

function LayerElement({
  layer,
  settings,
  title,
  location,
  countdown,
  selected,
  editMode,
  onPointerDown,
  onSelect,
}: {
  layer: OverlayLayer;
  settings: AppSettings;
  title: string;
  location?: string | null;
  countdown: string;
  selected: boolean;
  editMode: boolean;
  onPointerDown: (event: ReactPointerEvent<HTMLSpanElement>) => void;
  onSelect: () => void;
}) {
  const fontSize = layer.font_size ?? settings.font_size;

  let body: React.ReactNode = null;
  switch (layer.type) {
    case "image": {
      const size = layer.width ?? settings.icon_size;
      body = layer.image_path ? (
        <img src={layer.image_path} alt="" style={{ width: size, height: size }} />
      ) : (
        <span style={{ fontSize: size, lineHeight: 1 }}>{iconEmoji(layer.icon_id ?? "bell")}</span>
      );
      break;
    }
    case "title":
      body = (
        <span className="font-semibold leading-tight" style={{ fontSize }}>
          {title}
          {location ? ` · ${location}` : ""}
        </span>
      );
      break;
    case "countdown":
      body = (
        <span style={{ fontSize, color: "inherit", opacity: 0.9 }}>
          {countdown}
        </span>
      );
      break;
    case "text":
      body = (
        <span style={{ fontSize }}>{layer.text_content ?? "Text"}</span>
      );
      break;
  }

  return (
    <span
      className={`reminder-layer ${editMode ? "reminder-layer--editable" : ""} ${
        selected ? "reminder-layer--selected" : ""
      }`}
      style={{ left: `${layer.x}%`, top: `${layer.y}%`, zIndex: layer.z_index }}
      onPointerDown={(event) => {
        onSelect();
        onPointerDown(event);
      }}
    >
      {body}
    </span>
  );
}

function clampPercent(value: number): number {
  return Math.min(98, Math.max(2, Math.round(value * 10) / 10));
}

export const reminderBubbleStyles = `
  .reminder-canvas {
    position: relative;
    display: block;
    background: transparent;
    border: none;
    padding: 0;
    overflow: visible;
  }
  .reminder-canvas--interactive {
    cursor: pointer;
  }
  .reminder-canvas--edit {
    cursor: default;
  }
  .reminder-layer {
    position: absolute;
    transform: translate(-50%, -50%);
    max-width: 80%;
    white-space: nowrap;
    pointer-events: auto;
  }
  .reminder-layer--editable {
    cursor: grab;
  }
  .reminder-layer--editable.reminder-layer--selected {
    outline: 1px dashed rgba(129, 140, 248, 0.95);
    outline-offset: 4px;
    border-radius: 6px;
    padding: 2px 4px;
  }
  .reminder-layer--editable:active {
    cursor: grabbing;
  }
  .layer-editor-canvas {
    background-color: #0f172a;
    background-image:
      linear-gradient(45deg, rgba(255,255,255,0.04) 25%, transparent 25%),
      linear-gradient(-45deg, rgba(255,255,255,0.04) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, rgba(255,255,255,0.04) 75%),
      linear-gradient(-45deg, transparent 75%, rgba(255,255,255,0.04) 75%);
    background-size: 16px 16px;
    background-position: 0 0, 0 8px, 8px -8px, -8px 0;
  }
`;
