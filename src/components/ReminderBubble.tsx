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
  canvasRef?: React.RefObject<HTMLDivElement | HTMLButtonElement | null>;
  allowOverflow?: boolean;
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
  canvasRef,
  allowOverflow = false,
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
      canvasWidth={composition.canvas_width}
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
        ref={canvasRef as React.RefObject<HTMLButtonElement>}
        type="button"
        className={`reminder-canvas reminder-canvas--interactive${
          allowOverflow ? " reminder-canvas--float" : ""
        } ${className}`}
        style={canvasStyle}
        onClick={onClick}
      >
        {content}
      </button>
    );
  }

  return (
    <div
      ref={canvasRef as React.RefObject<HTMLDivElement>}
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
  canvasWidth,
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
  canvasWidth: number;
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
        <TextBox
          fontSize={fontSize}
          width={layer.width ?? Math.min(220, canvasWidth - 24)}
          height={layer.height ?? 52}
          className="font-semibold leading-tight"
        >
          {title}
          {location ? ` · ${location}` : ""}
        </TextBox>
      );
      break;
    case "countdown":
      body = (
        <span className="reminder-layer__inline" style={{ fontSize, opacity: 0.9 }}>
          {countdown}
        </span>
      );
      break;
    case "text":
      body = (
        <TextBox
          fontSize={fontSize}
          width={layer.width ?? 140}
          height={layer.height ?? 40}
        >
          {layer.text_content ?? "Text"}
        </TextBox>
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

function TextBox({
  children,
  fontSize,
  width,
  height,
  className = "",
}: {
  children: React.ReactNode;
  fontSize: number;
  width: number;
  height: number;
  className?: string;
}) {
  return (
    <span
      className={`reminder-text-box ${className}`}
      style={{ fontSize, width, height }}
    >
      <span className="reminder-text-box__content">{children}</span>
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
    overflow: hidden;
  }
  .reminder-canvas--interactive {
    cursor: pointer;
  }
  .reminder-canvas--float {
    overflow: visible;
  }
  .reminder-canvas--edit {
    cursor: default;
    background: rgba(15, 23, 42, 0.22);
    box-shadow:
      inset 0 0 0 1px rgba(148, 163, 184, 0.4),
      0 0 0 2px rgba(99, 102, 241, 0.85);
  }
  .reminder-layer {
    position: absolute;
    transform: translate(-50%, -50%);
    pointer-events: auto;
    max-width: 100%;
  }
  .reminder-layer__inline {
    white-space: nowrap;
  }
  .reminder-text-box {
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    overflow: hidden;
    border-radius: 6px;
    padding: 4px 6px;
    background: transparent;
  }
  .reminder-text-box__content {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 4;
    overflow: hidden;
    overflow-wrap: anywhere;
    word-break: break-word;
    white-space: normal;
    text-align: center;
    line-height: 1.25;
    width: 100%;
    max-height: 100%;
  }
  .reminder-canvas--edit .reminder-text-box {
    background: rgba(15, 23, 42, 0.35);
    outline: 1px dashed rgba(100, 116, 139, 0.45);
  }
  .reminder-layer--editable {
    cursor: grab;
  }
  .reminder-layer--editable.reminder-layer--selected {
    outline: 1px dashed rgba(129, 140, 248, 0.95);
    outline-offset: 4px;
    border-radius: 6px;
  }
  .reminder-layer--editable:active {
    cursor: grabbing;
  }
  .layer-editor-workspace {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: max(160px, calc(var(--canvas-height, 88px) + 48px));
    padding: 24px;
    overflow: auto;
    border-radius: 1rem;
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
