import { MarkerType, type Edge, type Node } from "@xyflow/react";
import type {
  ValueStreamEntry,
  ValueStreamMap,
  VsmEdge,
  VsmFlowTypeDef,
  VsmMessage,
  VsmNode,
  VsmNodeType,
} from "../types";

export interface VsmNodeData extends Record<string, unknown> {
  label: string;
  notes?: string;
  role?: string;
  author?: string;
  leadTimeMinutes?: number | null;
  cycleTimeMinutes?: number | null;
}

export interface VsmEdgeData extends Record<string, unknown> {
  edgeType: string;
  durationMinutes?: number | null;
}

export interface VsmTimelineSegment {
  edgeId: string;
  fromId: string;
  toId: string;
  fromLabel: string;
  toLabel: string;
  edgeLabel?: string;
  edgeType: string;
  durationMinutes: number;
  startOffset: number;
  endOffset: number;
  percentOfTotal: number;
  sourceAuthor?: string;
  targetAuthor?: string;
}

export interface VsmTimelineMilestone {
  nodeId: string;
  label: string;
  author?: string;
  offsetMinutes: number;
  nodeType?: string;
}

export const MINUTES_PER_HOUR = 60;
export const MINUTES_PER_DAY = 60 * 24;
export const MINUTES_PER_WEEK = 60 * 24 * 7;

export type DurationUnit = "minutes" | "hours" | "days" | "weeks";

export interface VsmTimelineStats {
  flowCount: number;
  timedFlowCount: number;
  untimedFlowCount: number;
  coveragePercent: number;
  longestEdgeId?: string;
  longestDuration: number;
}

export interface VsmTimelineResult {
  segments: VsmTimelineSegment[];
  milestones: VsmTimelineMilestone[];
  rulerTicks: number[];
  totalMinutes: number;
  stats: VsmTimelineStats;
}

export interface PaletteItem {
  type: VsmNodeType;
  icon: string;
  color: string;
}

export const NODE_PALETTE: PaletteItem[] = [
  { type: "process", icon: "▢", color: "#1e88e5" },
  { type: "decision", icon: "◇", color: "#f9a825" },
  { type: "info", icon: "≋", color: "#43a047" },
  { type: "delay", icon: "⏱", color: "#fb8c00" },
  { type: "external", icon: "⬡", color: "#78909c" },
  { type: "customer", icon: "👤", color: "#5c6bc0" },
  { type: "supplier", icon: "🏭", color: "#8d6e63" },
  { type: "inventory", icon: "▤", color: "#26a69a" },
  { type: "kaizen", icon: "✦", color: "#e91e63" },
];

export const DEFAULT_FLOW_TYPES: VsmFlowTypeDef[] = [
  { id: "material", label: "Material flow", color: "#37474f" },
  { id: "information", label: "Information flow", color: "#1e88e5", dash: "6 4" },
  { id: "electronic", label: "Electronic flow", color: "#8e24aa", dash: "2 4" },
];

export const BUILTIN_FLOW_TYPE_IDS = new Set(DEFAULT_FLOW_TYPES.map((t) => t.id));

export function isBuiltinFlowType(id: string): boolean {
  return BUILTIN_FLOW_TYPE_IDS.has(id);
}

export function resolveFlowTypes(map: ValueStreamMap): VsmFlowTypeDef[] {
  const merged = new Map(DEFAULT_FLOW_TYPES.map((type) => [type.id, { ...type }]));
  for (const custom of map.flow_types ?? []) {
    const existing = merged.get(custom.id);
    merged.set(custom.id, existing ? { ...existing, ...custom } : { ...custom });
  }
  for (const edge of map.edges) {
    const id = edge.edge_type ?? "material";
    if (!merged.has(id)) {
      merged.set(id, { id, label: id, color: "#78909c" });
    }
  }
  return [...merged.values()];
}

export function flowTypeConfig(
  id: string,
  flowTypes: VsmFlowTypeDef[],
): VsmFlowTypeDef {
  return (
    flowTypes.find((type) => type.id === id) ?? {
      id,
      label: id,
      color: "#78909c",
    }
  );
}

export function slugifyFlowTypeId(label: string): string {
  const slug = label
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  return slug || `flow-${crypto.randomUUID().slice(0, 6)}`;
}

export function flowTypesToPersist(types: VsmFlowTypeDef[]): VsmFlowTypeDef[] {
  return types.filter((type) => {
    if (!isBuiltinFlowType(type.id)) return true;
    const builtin = DEFAULT_FLOW_TYPES.find((item) => item.id === type.id);
    if (!builtin) return true;
    return builtin.color !== type.color || builtin.dash !== type.dash;
  });
}

/** @deprecated Use flowTypesToPersist */
export function customFlowTypesOnly(types: VsmFlowTypeDef[]): VsmFlowTypeDef[] {
  return flowTypesToPersist(types);
}

const AUTHOR_COLORS = [
  "#5c6bc0",
  "#26a69a",
  "#ef6c00",
  "#8e24aa",
  "#c62828",
  "#2e7d32",
  "#1565c0",
  "#6d4c41",
  "#00838f",
  "#ad1457",
  "#4527a0",
  "#558b2f",
];

export function authorColor(author: string): string {
  const trimmed = author.trim();
  if (!trimmed) return "#78909c";
  let hash = 0;
  for (let i = 0; i < trimmed.length; i += 1) {
    hash = trimmed.charCodeAt(i) + ((hash << 5) - hash);
  }
  return AUTHOR_COLORS[Math.abs(hash) % AUTHOR_COLORS.length];
}

export function builtinFlowTypeDefault(id: string): VsmFlowTypeDef | undefined {
  return DEFAULT_FLOW_TYPES.find((type) => type.id === id);
}

export function isFlowTypeOverridden(type: VsmFlowTypeDef): boolean {
  if (!isBuiltinFlowType(type.id)) return false;
  const builtin = builtinFlowTypeDefault(type.id);
  if (!builtin) return false;
  return builtin.color !== type.color || builtin.dash !== type.dash;
}

export function nodeDimensions(type: VsmNodeType): { width: number; height: number } {
  switch (type) {
    case "decision":
      return { width: 110, height: 110 };
    case "customer":
    case "supplier":
      return { width: 150, height: 64 };
    case "inventory":
      return { width: 120, height: 80 };
    case "kaizen":
      return { width: 100, height: 100 };
    case "delay":
      return { width: 140, height: 52 };
    default:
      return { width: 180, height: 72 };
  }
}

export function vsmToFlow(
  map: ValueStreamMap,
  flowTypes: VsmFlowTypeDef[] = resolveFlowTypes(map),
): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = map.nodes.map((n) => ({
    id: n.id,
    type: n.node_type,
    position: { x: n.x, y: n.y },
    data: {
      label: n.label,
      notes: n.notes ?? "",
      role: n.role ?? "",
      author: n.author ?? "",
      leadTimeMinutes: n.lead_time_minutes ?? null,
      cycleTimeMinutes: n.cycle_time_minutes ?? null,
    } satisfies VsmNodeData,
    style: { width: n.width, height: n.height },
  }));
  const edges: Edge[] = map.edges.map((e) => {
    const edgeType = e.edge_type ?? "material";
    const cfg = flowTypeConfig(edgeType, flowTypes);
    return {
      id: e.id,
      source: e.from,
      target: e.to,
      label: e.label ?? undefined,
      type: "vsm",
      data: {
        edgeType,
        durationMinutes: e.duration_minutes ?? null,
      } satisfies VsmEdgeData,
      markerEnd: {
        type: MarkerType.ArrowClosed,
        color: cfg.color,
        width: 18,
        height: 18,
      },
    };
  });
  return { nodes, edges };
}

export function flowToVsm(
  nodes: Node[],
  edges: Edge[],
  messages: VsmMessage[],
  customFlowTypes: VsmFlowTypeDef[] = [],
): ValueStreamMap {
  return {
    flow_types: flowTypesToPersist(customFlowTypes),
    nodes: nodes.map((n) => {
      const d = n.data as VsmNodeData;
      return {
        id: n.id,
        label: String(d.label ?? ""),
        node_type: (n.type as VsmNodeType) ?? "process",
        x: n.position.x,
        y: n.position.y,
        width: typeof n.style?.width === "number" ? n.style.width : 180,
        height: typeof n.style?.height === "number" ? n.style.height : 72,
        notes: d.notes?.trim() ? String(d.notes) : undefined,
        role: d.role?.trim() ? String(d.role) : undefined,
        author: d.author?.trim() ? String(d.author) : undefined,
        lead_time_minutes: d.leadTimeMinutes ?? undefined,
        cycle_time_minutes: d.cycleTimeMinutes ?? undefined,
      };
    }),
    edges: edges.map((e) => {
      const d = (e.data ?? {}) as VsmEdgeData;
      return {
        id: e.id,
        from: e.source,
        to: e.target,
        label: e.label ? String(e.label) : undefined,
        edge_type: d.edgeType ?? "material",
        duration_minutes: d.durationMinutes ?? undefined,
      };
    }),
    messages,
  };
}

function buildRulerTicks(totalMinutes: number, tickCount = 5): number[] {
  if (totalMinutes <= 0) return [0];

  const step = pickRulerStep(totalMinutes, tickCount - 1);
  const ticks = new Set<number>([0]);
  for (let value = step; value < totalMinutes; value += step) {
    ticks.add(Math.round(value));
  }
  ticks.add(Math.round(totalMinutes));
  return [...ticks].sort((a, b) => a - b);
}

function pickRulerStep(totalMinutes: number, divisions: number): number {
  const raw = totalMinutes / Math.max(divisions, 1);
  const candidates =
    totalMinutes >= MINUTES_PER_WEEK
      ? [MINUTES_PER_WEEK, MINUTES_PER_DAY, MINUTES_PER_HOUR * 12, MINUTES_PER_HOUR]
      : totalMinutes >= MINUTES_PER_DAY
        ? [MINUTES_PER_DAY, MINUTES_PER_HOUR * 6, MINUTES_PER_HOUR * 3, MINUTES_PER_HOUR]
        : totalMinutes >= MINUTES_PER_HOUR
          ? [MINUTES_PER_HOUR, 30, 15, 5]
          : [15, 10, 5, 1];

  for (const candidate of candidates) {
    if (raw <= candidate * 1.5) return candidate;
  }
  return Math.max(1, Math.round(raw));
}

function buildMilestones(
  segments: VsmTimelineSegment[],
  nodeMap: Map<string, Node>,
): VsmTimelineMilestone[] {
  if (segments.length === 0) return [];

  const milestones: VsmTimelineMilestone[] = [];
  const seen = new Set<string>();

  const pushMilestone = (nodeId: string, offsetMinutes: number) => {
    if (seen.has(nodeId)) return;
    seen.add(nodeId);
    const node = nodeMap.get(nodeId);
    const data = node?.data as VsmNodeData | undefined;
    milestones.push({
      nodeId,
      label: String(data?.label ?? nodeId),
      author: data?.author?.trim() || undefined,
      offsetMinutes,
      nodeType: node?.type ? String(node.type) : undefined,
    });
  };

  pushMilestone(segments[0].fromId, 0);
  for (const segment of segments) {
    pushMilestone(segment.toId, segment.endOffset);
  }

  return milestones;
}

/** Order flows left-to-right and build a cumulative timeline from edge durations. */
export function buildVsmTimeline(nodes: Node[], edges: Edge[]): VsmTimelineResult {
  const nodeMap = new Map(nodes.map((n) => [n.id, n]));

  const orderedEdges = [...edges].sort((a, b) => {
    const aX = nodeMap.get(a.source)?.position.x ?? 0;
    const bX = nodeMap.get(b.source)?.position.x ?? 0;
    if (aX !== bX) return aX - bX;
    const aTx = nodeMap.get(a.target)?.position.x ?? 0;
    const bTx = nodeMap.get(b.target)?.position.x ?? 0;
    return aTx - bTx;
  });

  let offset = 0;
  const segments: VsmTimelineSegment[] = orderedEdges.map((edge) => {
    const d = (edge.data ?? {}) as VsmEdgeData;
    const duration = d.durationMinutes ?? 0;
    const source = nodeMap.get(edge.source);
    const target = nodeMap.get(edge.target);
    const sourceData = source?.data as VsmNodeData | undefined;
    const targetData = target?.data as VsmNodeData | undefined;
    const segment: VsmTimelineSegment = {
      edgeId: edge.id,
      fromId: edge.source,
      toId: edge.target,
      fromLabel: String(sourceData?.label ?? edge.source),
      toLabel: String(targetData?.label ?? edge.target),
      edgeLabel: edge.label ? String(edge.label) : undefined,
      edgeType: d.edgeType ?? "material",
      durationMinutes: duration,
      startOffset: offset,
      endOffset: offset + duration,
      percentOfTotal: 0,
      sourceAuthor: sourceData?.author?.trim() || undefined,
      targetAuthor: targetData?.author?.trim() || undefined,
    };
    offset += duration;
    return segment;
  });

  const totalMinutes = offset;
  for (const segment of segments) {
    segment.percentOfTotal =
      totalMinutes > 0 ? Math.round((segment.durationMinutes / totalMinutes) * 100) : 0;
  }

  const timedFlowCount = segments.filter((s) => s.durationMinutes > 0).length;
  const untimedFlowCount = segments.length - timedFlowCount;
  let longestEdgeId: string | undefined;
  let longestDuration = 0;
  for (const segment of segments) {
    if (segment.durationMinutes > longestDuration) {
      longestDuration = segment.durationMinutes;
      longestEdgeId = segment.edgeId;
    }
  }

  return {
    segments,
    milestones: buildMilestones(segments, nodeMap),
    rulerTicks: buildRulerTicks(totalMinutes),
    totalMinutes,
    stats: {
      flowCount: segments.length,
      timedFlowCount,
      untimedFlowCount,
      coveragePercent:
        segments.length > 0 ? Math.round((timedFlowCount / segments.length) * 100) : 0,
      longestEdgeId,
      longestDuration,
    },
  };
}

export const TIMELINE_BAR_GAP_PX = 3;
export const TIMELINE_MIN_BAR_PX = 76;
export const TIMELINE_MIN_UNTIMED_BAR_PX = 52;

export interface TimelineBarLayout {
  edgeId: string;
  leftPx: number;
  widthPx: number;
  showLabel: boolean;
  showPercent: boolean;
}

export interface TimelineMilestoneLayout {
  nodeId: string;
  leftPx: number;
  compact: boolean;
  row: 0 | 1;
}

export interface TimelineRulerTickLayout {
  value: number;
  leftPx: number;
}

export interface TimelineChartLayout {
  chartWidthPx: number;
  scrollable: boolean;
  compactMode: boolean;
  bars: TimelineBarLayout[];
  milestones: TimelineMilestoneLayout[];
  rulerTicks: TimelineRulerTickLayout[];
}

function fitTimelineUnitWidths(
  segments: VsmTimelineSegment[],
  chartWidthPx: number,
): number[] {
  if (segments.length === 0) return [];

  const untimedCount = segments.filter((segment) => segment.durationMinutes <= 0).length;
  const timedTotal = segments.reduce(
    (sum, segment) => sum + Math.max(segment.durationMinutes, 0),
    0,
  );

  let unitWidths: number[];
  if (timedTotal > 0) {
    const untimedPool =
      untimedCount > 0
        ? Math.min(chartWidthPx * 0.06 * untimedCount, chartWidthPx * 0.22)
        : 0;
    const timedPool = chartWidthPx - untimedPool;
    const perUntimed = untimedCount > 0 ? untimedPool / untimedCount : 0;
    unitWidths = segments.map((segment) =>
      segment.durationMinutes > 0
        ? (segment.durationMinutes / timedTotal) * timedPool
        : perUntimed,
    );
  } else {
    unitWidths = segments.map(() => chartWidthPx / segments.length);
  }

  const sum = unitWidths.reduce((total, width) => total + width, 0);
  if (sum <= 0) return unitWidths;
  return unitWidths.map((width) => (width / sum) * chartWidthPx);
}

/** Timeline chart: fits when possible, otherwise expands with horizontal scroll. */
export function computeTimelineChartLayout(
  segments: VsmTimelineSegment[],
  milestones: VsmTimelineMilestone[],
  totalMinutes: number,
  rulerTickValues: number[],
  containerWidth: number,
): TimelineChartLayout {
  const gap = TIMELINE_BAR_GAP_PX;
  const viewportWidth = Math.max(containerWidth, 320);

  let unitWidths: number[];
  if (totalMinutes > 0) {
    unitWidths = segments.map((segment) => {
      const proportional = (segment.durationMinutes / totalMinutes) * viewportWidth;
      const minimum =
        segment.durationMinutes > 0 ? TIMELINE_MIN_BAR_PX : TIMELINE_MIN_UNTIMED_BAR_PX;
      return Math.max(proportional, minimum);
    });
  } else {
    unitWidths = segments.map(() =>
      Math.max(TIMELINE_MIN_UNTIMED_BAR_PX, viewportWidth / Math.max(segments.length, 1)),
    );
  }

  const naturalWidth = unitWidths.reduce((sum, width) => sum + width, 0);
  const scrollable = naturalWidth > viewportWidth + 2;
  const chartWidthPx = scrollable ? naturalWidth : viewportWidth;

  if (!scrollable) {
    unitWidths = fitTimelineUnitWidths(segments, viewportWidth);
  }

  const avgBarWidth = chartWidthPx / Math.max(segments.length, 1);
  const compactMode = !scrollable && (segments.length >= 5 || avgBarWidth < 48);

  let left = 0;
  const bars: TimelineBarLayout[] = segments.map((segment, index) => {
    const slotWidth = unitWidths[index] ?? 0;
    const widthPx = Math.max(slotWidth - gap, 6);
    const hasDuration = segment.durationMinutes > 0;
    const bar: TimelineBarLayout = {
      edgeId: segment.edgeId,
      leftPx: left,
      widthPx,
      showLabel: hasDuration && widthPx >= (compactMode ? 54 : 44),
      showPercent: hasDuration && widthPx >= (compactMode ? 92 : 78),
    };
    left += slotWidth;
    return bar;
  });

  const milestoneLeftPx = new Map<string, number>();
  if (segments.length > 0) {
    milestoneLeftPx.set(segments[0].fromId, 0);
    segments.forEach((segment, index) => {
      const bar = bars[index];
      milestoneLeftPx.set(segment.toId, bar.leftPx + bar.widthPx + gap / 2);
    });
  }

  const avgMilestoneGap =
    milestones.length > 1 ? chartWidthPx / (milestones.length - 1) : chartWidthPx;
  const denseMilestones = milestones.length >= 6 || avgMilestoneGap < 80;

  const milestoneLayouts: TimelineMilestoneLayout[] = milestones.map((milestone, index) => {
    const fallback =
      totalMinutes > 0
        ? (milestone.offsetMinutes / totalMinutes) * chartWidthPx
        : milestones.length <= 1
          ? 0
          : (index / (milestones.length - 1)) * chartWidthPx;
    return {
      nodeId: milestone.nodeId,
      leftPx: milestoneLeftPx.get(milestone.nodeId) ?? fallback,
      compact: denseMilestones,
      row: 0 as const,
    };
  });

  if (!denseMilestones) {
    for (let i = 1; i < milestoneLayouts.length; i += 1) {
      const previous = milestoneLayouts[i - 1];
      const current = milestoneLayouts[i];
      const delta = current.leftPx - previous.leftPx;
      if (delta < 88) {
        current.compact = true;
        current.row = previous.row === 0 ? 1 : 0;
        if (!previous.compact && delta < 56) {
          previous.compact = true;
        }
      }
    }
  }

  const tickValues =
    compactMode && rulerTickValues.length > 3
      ? [rulerTickValues[0], rulerTickValues[Math.floor(rulerTickValues.length / 2)], rulerTickValues[rulerTickValues.length - 1]]
      : rulerTickValues;

  const rulerTicks: TimelineRulerTickLayout[] = tickValues.map((value) => ({
    value,
    leftPx: totalMinutes > 0 ? (value / totalMinutes) * chartWidthPx : 0,
  }));

  return {
    chartWidthPx,
    scrollable,
    compactMode,
    bars,
    milestones: milestoneLayouts,
    rulerTicks,
  };
}

export function abbreviateTimelineLabel(label: string, maxLength = 12): string {
  const trimmed = label.trim();
  if (trimmed.length <= maxLength) return trimmed;
  return `${trimmed.slice(0, maxLength - 1)}…`;
}

export function timelineOffsetPercent(
  offsetMinutes: number,
  totalMinutes: number,
  fallbackIndex: number,
  fallbackCount: number,
): number {
  if (totalMinutes > 0) return (offsetMinutes / totalMinutes) * 100;
  if (fallbackCount <= 1) return 0;
  return (fallbackIndex / (fallbackCount - 1)) * 100;
}

export function edgeTypeShort(edgeType: string, flowTypes?: VsmFlowTypeDef[]): string {
  if (edgeType === "information") return "I";
  if (edgeType === "electronic") return "E";
  if (edgeType === "material") return "M";
  const label = flowTypeConfig(edgeType, flowTypes ?? DEFAULT_FLOW_TYPES).label;
  return label.charAt(0).toUpperCase() || "?";
}

export function collectVsmAuthors(nodes: Node[]): string[] {
  const authors = new Set<string>();
  for (const node of nodes) {
    const author = (node.data as VsmNodeData).author?.trim();
    if (author) authors.add(author);
  }
  return [...authors].sort((a, b) => a.localeCompare(b));
}

export function newNodeId(): string {
  return `node-${crypto.randomUUID().slice(0, 8)}`;
}

export function newEdgeId(): string {
  return `edge-${crypto.randomUUID().slice(0, 8)}`;
}

export function newMessageId(): string {
  return `msg-${crypto.randomUUID().slice(0, 8)}`;
}

export function createVsmNode(
  nodeType: VsmNodeType,
  label: string,
  x: number,
  y: number,
): VsmNode {
  const { width, height } = nodeDimensions(nodeType);
  return {
    id: newNodeId(),
    label,
    node_type: nodeType,
    x,
    y,
    width,
    height,
  };
}

export const EMPTY_VALUE_STREAM: ValueStreamMap = {
  nodes: [],
  edges: [],
  messages: [],
  flow_types: [],
};

export function emptyValueStream(): ValueStreamMap {
  return EMPTY_VALUE_STREAM;
}

export function entryToMap(entry: ValueStreamEntry): ValueStreamMap {
  return {
    nodes: entry.nodes,
    edges: entry.edges,
    messages: entry.messages,
    flow_types: entry.flow_types,
  };
}

export function mapToEntryBody(name: string, map: ValueStreamMap): Omit<ValueStreamEntry, "id"> {
  return {
    name,
    nodes: map.nodes,
    edges: map.edges,
    messages: map.messages,
    flow_types: map.flow_types,
  };
}

export function newVsmId(): string {
  return `vsm-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function valueStreamMapsEqual(a: ValueStreamMap, b: ValueStreamMap): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function formatDurationWithLabels(
  minutes: number | null | undefined,
  style: "auto" | "compact",
  labels: DurationFormatLabels,
): string {
  if (minutes == null || Number.isNaN(minutes)) return "";
  const total = Math.round(minutes);
  if (total <= 0) return "0";

  const part = (value: number, unit: string) =>
    labels.sep ? `${value}${labels.sep}${unit}` : `${value}${unit}`;

  if (total < MINUTES_PER_HOUR) {
    return part(total, labels.minute);
  }

  if (total < MINUTES_PER_DAY) {
    const hours = Math.floor(total / MINUTES_PER_HOUR);
    const mins = total % MINUTES_PER_HOUR;
    if (mins === 0 || style === "compact") return part(hours, labels.hour);
    return `${part(hours, labels.hour)} ${part(mins, labels.minute)}`;
  }

  if (total < MINUTES_PER_WEEK) {
    const days = Math.floor(total / MINUTES_PER_DAY);
    const rem = total % MINUTES_PER_DAY;
    const hours = Math.floor(rem / MINUTES_PER_HOUR);
    const mins = rem % MINUTES_PER_HOUR;
    if (hours === 0 && mins === 0) return part(days, labels.day);
    if (mins === 0 || style === "compact") {
      return hours > 0
        ? `${part(days, labels.day)} ${part(hours, labels.hour)}`
        : part(days, labels.day);
    }
    if (hours === 0) return `${part(days, labels.day)} ${part(mins, labels.minute)}`;
    return `${part(days, labels.day)} ${part(hours, labels.hour)}`;
  }

  const weeks = Math.floor(total / MINUTES_PER_WEEK);
  const rem = total % MINUTES_PER_WEEK;
  const days = Math.floor(rem / MINUTES_PER_DAY);
  const hours = Math.floor((rem % MINUTES_PER_DAY) / MINUTES_PER_HOUR);
  if (days === 0 && hours === 0) return part(weeks, labels.week);
  if (hours === 0 || style === "compact") {
    return days > 0
      ? `${part(weeks, labels.week)} ${part(days, labels.day)}`
      : part(weeks, labels.week);
  }
  return days > 0
    ? `${part(weeks, labels.week)} ${part(days, labels.day)}`
    : `${part(weeks, labels.week)} ${part(hours, labels.hour)}`;
}

export interface DurationFormatLabels {
  minute: string;
  hour: string;
  day: string;
  week: string;
  sep: string;
}

const EN_DURATION_LABELS: DurationFormatLabels = {
  minute: "m",
  hour: "h",
  day: "d",
  week: "w",
  sep: "",
};

export function formatDuration(
  minutes: number | null | undefined,
  style: "auto" | "compact" = "auto",
): string {
  return formatDurationWithLabels(minutes, style, EN_DURATION_LABELS);
}

/** @deprecated Use formatDuration */
export function formatMinutes(minutes: number | null | undefined): string {
  return formatDuration(minutes);
}

export function bestDurationUnit(minutes: number | null | undefined): DurationUnit {
  if (minutes == null || minutes < MINUTES_PER_HOUR) return "minutes";
  if (minutes < MINUTES_PER_DAY) return "hours";
  if (minutes < MINUTES_PER_WEEK) return "days";
  return "weeks";
}

export function minutesToUnitValue(
  minutes: number | null | undefined,
  unit: DurationUnit,
): number | "" {
  if (minutes == null || Number.isNaN(minutes)) return "";
  switch (unit) {
    case "minutes":
      return minutes;
    case "hours":
      return Math.round((minutes / MINUTES_PER_HOUR) * 100) / 100;
    case "days":
      return Math.round((minutes / MINUTES_PER_DAY) * 100) / 100;
    case "weeks":
      return Math.round((minutes / MINUTES_PER_WEEK) * 100) / 100;
    default:
      return minutes;
  }
}

export function unitValueToMinutes(value: number, unit: DurationUnit): number {
  switch (unit) {
    case "minutes":
      return value;
    case "hours":
      return value * MINUTES_PER_HOUR;
    case "days":
      return value * MINUTES_PER_DAY;
    case "weeks":
      return value * MINUTES_PER_WEEK;
    default:
      return value;
  }
}

export function parseDurationInput(raw: string, unit: DurationUnit): number | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;
  const n = Number(trimmed);
  if (!Number.isFinite(n) || n < 0) return null;
  return Math.round(unitValueToMinutes(n, unit));
}

/** Starter MDM enrollment value stream template. */
export function mdmEnrollmentTemplate(): ValueStreamMap {
  const n = (id: string, label: string, type: VsmNodeType, x: number, y: number): VsmNode => {
    const { width, height } = nodeDimensions(type);
    return { id, label, node_type: type, x, y, width, height };
  };
  const nodes: VsmNode[] = [
    { ...n("tpl-customer", "Business request", "customer", 40, 120), author: "Business owner" },
    { ...n("tpl-procure", "Vendor selection", "process", 240, 100), author: "Procurement" },
    { ...n("tpl-decide", "Approved?", "decision", 460, 90), author: "Security board" },
    { ...n("tpl-enroll", "Device enrollment", "process", 660, 60), author: "IT Ops" },
    { ...n("tpl-mdm", "MDM platform", "external", 660, 200), author: "MDM admin" },
    { ...n("tpl-info", "Policy sync", "info", 880, 130), author: "Security" },
    { ...n("tpl-inv", "Device fleet", "inventory", 1080, 100), author: "IT Ops" },
  ];
  const edges: VsmEdge[] = [
    { id: "e1", from: "tpl-customer", to: "tpl-procure", edge_type: "information", duration_minutes: 480 },
    { id: "e2", from: "tpl-procure", to: "tpl-decide", edge_type: "information", duration_minutes: 2400 },
    { id: "e3", from: "tpl-decide", to: "tpl-enroll", label: "Yes", edge_type: "material", duration_minutes: 120 },
    { id: "e4", from: "tpl-enroll", to: "tpl-mdm", edge_type: "electronic", duration_minutes: 30 },
    { id: "e5", from: "tpl-mdm", to: "tpl-info", edge_type: "electronic", duration_minutes: 15 },
    { id: "e6", from: "tpl-info", to: "tpl-inv", edge_type: "material", duration_minutes: 60 },
  ];
  return { nodes, edges, messages: [] };
}
