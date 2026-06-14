import type { Edge, Node } from "@xyflow/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useFormatDuration } from "../../i18n/useFormatDuration";
import { useLocale } from "../../i18n/LocaleContext";
import {
  abbreviateTimelineLabel,
  authorColor,
  buildVsmTimeline,
  computeTimelineChartLayout,
  edgeTypeShort,
  flowTypeConfig,
  NODE_PALETTE,
  type VsmTimelineSegment,
} from "../../utils/valueStream";
import { useVsmFlowTypes } from "./VsmFlowTypesContext";

interface VsmTimelineProps {
  nodes: Node[];
  edges: Edge[];
  selectedEdgeId: string | null;
  onSelectEdge: (edgeId: string) => void;
  onSelectNode?: (nodeId: string) => void;
}

function nodeAccent(nodeType?: string): string {
  return NODE_PALETTE.find((p) => p.type === nodeType)?.color ?? "#78909c";
}

function segmentTitle(segment: VsmTimelineSegment): string {
  const flow = segment.edgeLabel
    ? `${segment.fromLabel} → ${segment.toLabel} (${segment.edgeLabel})`
    : `${segment.fromLabel} → ${segment.toLabel}`;
  return flow;
}

export function VsmTimeline({
  nodes,
  edges,
  selectedEdgeId,
  onSelectEdge,
  onSelectNode,
}: VsmTimelineProps) {
  const { t } = useLocale();
  const formatDuration = useFormatDuration();
  const flowTypes = useVsmFlowTypes();
  const viewportRef = useRef<HTMLDivElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);
  const [showDetails, setShowDetails] = useState(true);

  const timeline = useMemo(() => buildVsmTimeline(nodes, edges), [nodes, edges]);
  const { segments, milestones, rulerTicks, totalMinutes, stats } = timeline;

  useEffect(() => {
    const element = viewportRef.current;
    if (!element) return undefined;

    const measure = () => setContainerWidth(element.clientWidth);
    measure();

    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [edges.length]);

  const layout = useMemo(() => {
    if (containerWidth <= 0) return null;
    return computeTimelineChartLayout(
      segments,
      milestones,
      totalMinutes,
      rulerTicks,
      containerWidth,
    );
  }, [segments, milestones, totalMinutes, rulerTicks, containerWidth]);

  const barLayoutById = useMemo(
    () => new Map(layout?.bars.map((bar) => [bar.edgeId, bar]) ?? []),
    [layout],
  );

  const milestoneLayoutById = useMemo(
    () => new Map(layout?.milestones.map((m) => [m.nodeId, m]) ?? []),
    [layout],
  );

  useEffect(() => {
    if (!selectedEdgeId || !layout?.scrollable || !scrollRef.current) return;
    const bar = barLayoutById.get(selectedEdgeId);
    if (!bar) return;
    const scrollEl = scrollRef.current;
    const center = bar.leftPx + bar.widthPx / 2;
    scrollEl.scrollTo({
      left: Math.max(0, center - scrollEl.clientWidth / 2),
      behavior: "smooth",
    });
  }, [selectedEdgeId, layout, barLayoutById]);

  const flowTypeLabel = (edgeType: string) => {
    if (edgeType === "material") return t.vsm.edgeMaterial;
    if (edgeType === "information") return t.vsm.edgeInformation;
    if (edgeType === "electronic") return t.vsm.edgeElectronic;
    return flowTypeConfig(edgeType, flowTypes).label;
  };

  const edgeColor = (edgeType: string) => flowTypeConfig(edgeType, flowTypes).color;

  const timelineAuthors = useMemo(() => {
    const names = new Set<string>();
    for (const segment of segments) {
      if (segment.targetAuthor) names.add(segment.targetAuthor);
      if (segment.sourceAuthor) names.add(segment.sourceAuthor);
    }
    for (const milestone of milestones) {
      if (milestone.author) names.add(milestone.author);
    }
    return [...names].sort((a, b) => a.localeCompare(b));
  }, [segments, milestones]);

  const renderAuthor = (author?: string) => {
    if (!author) return <span className="vsm-timeline-author vsm-timeline-author-empty">—</span>;
    const color = authorColor(author);
    return (
      <span
        className="vsm-timeline-author"
        style={{
          background: `${color}22`,
          color,
          borderColor: `${color}55`,
        }}
      >
        <span className="vsm-timeline-author-dot" style={{ background: color }} />
        {author}
      </span>
    );
  };

  if (edges.length === 0) {
    return (
      <div className="vsm-timeline card vsm-timeline-empty">
        <div className="vsm-timeline-empty-icon">⏱</div>
        <p className="vsm-timeline-empty-title">{t.vsm.timelineEmpty}</p>
        <p className="vsm-hint">{t.vsm.timelineEmptyHint}</p>
      </div>
    );
  }

  const longestSegment = segments.find((s) => s.edgeId === stats.longestEdgeId);
  const milestoneRows = layout?.milestones.some((m) => m.row === 1) ? 2 : 1;

  return (
    <div className="vsm-timeline card">
      <div className="vsm-timeline-header">
        <div>
          <h3>{t.vsm.timelineTitle}</h3>
          <p className="vsm-hint">{t.vsm.timelineHint}</p>
        </div>
        <button
          type="button"
          className="btn btn-ghost btn-sm vsm-timeline-toggle"
          onClick={() => setShowDetails((value) => !value)}
        >
          {showDetails ? t.vsm.timelineHideDetails : t.vsm.timelineShowDetails}
        </button>
      </div>

      <div className="vsm-timeline-stats">
        <div className="vsm-timeline-stat vsm-timeline-stat-primary">
          <span className="vsm-timeline-stat-label">{t.vsm.timelineTotal}</span>
          <strong>{formatDuration(totalMinutes) || "—"}</strong>
        </div>
        <div className="vsm-timeline-stat">
          <span className="vsm-timeline-stat-label">{t.vsm.timelineFlows}</span>
          <strong>
            {stats.timedFlowCount}/{stats.flowCount}
          </strong>
          <span className="vsm-timeline-stat-sub">{t.vsm.timelineTimed}</span>
        </div>
        <div className="vsm-timeline-stat">
          <span className="vsm-timeline-stat-label">{t.vsm.timelineCoverage}</span>
          <strong>{stats.coveragePercent}%</strong>
          <div className="vsm-timeline-coverage-bar">
            <span style={{ width: `${stats.coveragePercent}%` }} />
          </div>
        </div>
        {longestSegment && stats.longestDuration > 0 && (
          <div className="vsm-timeline-stat vsm-timeline-stat-accent">
            <span className="vsm-timeline-stat-label">{t.vsm.timelineLongest}</span>
            <strong>{formatDuration(stats.longestDuration)}</strong>
            <span className="vsm-timeline-stat-sub">{longestSegment.toLabel}</span>
          </div>
        )}
      </div>

      {timelineAuthors.length > 0 && (
        <div className="vsm-timeline-authors-legend">
          <span className="vsm-timeline-authors-title">{t.vsm.timelineAuthors}</span>
          <div className="vsm-timeline-authors-chips">
            {timelineAuthors.map((name) => (
              <span
                key={name}
                className="vsm-timeline-author-chip"
                style={{
                  background: `${authorColor(name)}18`,
                  color: authorColor(name),
                  borderColor: `${authorColor(name)}44`,
                }}
              >
                <span
                  className="vsm-timeline-author-dot"
                  style={{ background: authorColor(name) }}
                />
                {name}
              </span>
            ))}
          </div>
        </div>
      )}

      {stats.untimedFlowCount > 0 && (
        <div className="vsm-timeline-warning" role="status">
          <span className="vsm-timeline-warning-icon">!</span>
          <span>
            {t.vsm.timelineUntimedWarning.replace("{count}", String(stats.untimedFlowCount))}
          </span>
        </div>
      )}

      <div ref={viewportRef} className="vsm-timeline-chart-viewport">
        {layout?.scrollable && (
          <p className="vsm-timeline-scroll-hint" role="note">
            <span aria-hidden>↔</span>
            {t.vsm.timelineScrollHint}
          </p>
        )}

        <div
          ref={scrollRef}
          className={`vsm-timeline-chart-scroll${layout?.scrollable ? " is-scrollable" : ""}${layout?.compactMode ? " is-compact" : ""}`}
        >
          {layout && (
            <div
              className="vsm-timeline-chart-inner"
              style={{ width: layout.chartWidthPx }}
            >
              <div className="vsm-timeline-chart-grid">
                {layout.rulerTicks.map((tick) => (
                  <div
                    key={tick.value}
                    className="vsm-timeline-grid-line"
                    style={{ left: tick.leftPx }}
                  />
                ))}
              </div>

              <div className="vsm-timeline-chart-track" role="list" aria-label={t.vsm.timelineTitle}>
                {segments.map((segment) => {
                  const bar = barLayoutById.get(segment.edgeId);
                  if (!bar) return null;

                  const hasDuration = segment.durationMinutes > 0;
                  const isSelected = segment.edgeId === selectedEdgeId;
                  const isLongest = segment.edgeId === stats.longestEdgeId && hasDuration;
                  const color = edgeColor(segment.edgeType);

                  return (
                    <button
                      key={segment.edgeId}
                      type="button"
                      role="listitem"
                      className={`vsm-timeline-bar${isSelected ? " selected" : ""}${hasDuration ? "" : " untimed"}${isLongest ? " longest" : ""}${bar.showLabel ? "" : " no-label"}`}
                      style={{
                        left: bar.leftPx,
                        width: bar.widthPx,
                        background: hasDuration ? color : undefined,
                        borderColor: color,
                      }}
                      title={segmentTitle(segment)}
                      onClick={() => onSelectEdge(segment.edgeId)}
                    >
                      <span className="vsm-timeline-bar-type" title={flowTypeLabel(segment.edgeType)}>
                        {edgeTypeShort(segment.edgeType, flowTypes)}
                      </span>
                      {bar.showLabel && (
                        <span className="vsm-timeline-bar-label">
                          {hasDuration
                          ? formatDuration(segment.durationMinutes, "compact")
                          : "—"}
                          {bar.showPercent && (
                            <span className="vsm-timeline-bar-pct">{segment.percentOfTotal}%</span>
                          )}
                        </span>
                      )}
                    </button>
                  );
                })}
              </div>

              <div className="vsm-timeline-ruler">
                {layout.rulerTicks.map((tick) => (
                  <span
                    key={tick.value}
                    className="vsm-timeline-ruler-tick"
                    style={{ left: tick.leftPx }}
                  >
                    {formatDuration(tick.value, "compact") || "0"}
                  </span>
                ))}
              </div>

              {milestones.length > 0 && (
                <div className="vsm-timeline-milestones">
                  <span className="vsm-timeline-milestones-title">{t.vsm.timelineMilestones}</span>
                  <div
                    className="vsm-timeline-milestone-lane"
                    style={{
                      minHeight: layout.compactMode
                        ? "2.5rem"
                        : milestoneRows > 1
                          ? "5.25rem"
                          : "3.75rem",
                    }}
                  >
                    {milestones.map((milestone) => {
                      const milestoneLayout = milestoneLayoutById.get(milestone.nodeId);
                      const compact = milestoneLayout?.compact ?? layout.compactMode;
                      const row = layout.compactMode ? 0 : (milestoneLayout?.row ?? 0);
                      const title = milestone.author
                        ? `${milestone.label} · ${milestone.author}`
                        : milestone.label;

                      return (
                        <button
                          key={milestone.nodeId}
                          type="button"
                          className={`vsm-timeline-milestone${compact ? " compact" : ""} row-${row}`}
                          style={{
                            left: milestoneLayout?.leftPx ?? 0,
                            "--milestone-color": nodeAccent(milestone.nodeType),
                          } as React.CSSProperties}
                          onClick={() => onSelectNode?.(milestone.nodeId)}
                          title={title}
                        >
                          <span className="vsm-timeline-milestone-pin" />
                          {!compact && (
                            <span className="vsm-timeline-milestone-time">
                              {formatDuration(milestone.offsetMinutes, "compact") || "0"}
                            </span>
                          )}
                          {!compact && (
                            <>
                              <span className="vsm-timeline-milestone-label">
                                {abbreviateTimelineLabel(milestone.label)}
                              </span>
                              {milestone.author && (
                                <span
                                  className="vsm-timeline-milestone-author"
                                  style={{ color: authorColor(milestone.author) }}
                                >
                                  {milestone.author}
                                </span>
                              )}
                            </>
                          )}
                        </button>
                      );
                    })}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {showDetails && (
        <div className="vsm-timeline-details">
          <div className="vsm-timeline-details-head">
            <span>{t.vsm.timelineOffset}</span>
            <span>{t.vsm.timelineFlow}</span>
            <span>{t.vsm.timelineDuration}</span>
            <span>{t.vsm.nodeAuthor}</span>
          </div>
          <ul className="vsm-timeline-list">
            {segments.map((segment) => {
              const isSelected = segment.edgeId === selectedEdgeId;
              const hasDuration = segment.durationMinutes > 0;
              const color = edgeColor(segment.edgeType);
              return (
                <li key={segment.edgeId} className={isSelected ? "selected" : ""}>
                  <button
                    type="button"
                    className="vsm-timeline-row"
                    onClick={() => onSelectEdge(segment.edgeId)}
                  >
                    <span className="vsm-timeline-offset">
                      {formatDuration(segment.startOffset, "compact") || "0"}
                    </span>
                    <span className="vsm-timeline-flow-cell">
                      <span className="vsm-timeline-flow-line">
                        <span
                          className="vsm-timeline-flow-type"
                          style={{ color, borderColor: color }}
                        >
                          {edgeTypeShort(segment.edgeType, flowTypes)}
                        </span>
                        <span className="vsm-timeline-flow">
                          <strong>{segment.fromLabel}</strong>
                          <span className="vsm-timeline-arrow">→</span>
                          <strong>{segment.toLabel}</strong>
                          {segment.edgeLabel && (
                            <span className="vsm-timeline-edge-label">{segment.edgeLabel}</span>
                          )}
                        </span>
                      </span>
                      {hasDuration && (
                        <span className="vsm-timeline-row-bar">
                          <span
                            className="vsm-timeline-row-bar-fill"
                            style={{
                              width: `${segment.percentOfTotal}%`,
                              background: color,
                            }}
                          />
                        </span>
                      )}
                    </span>
                    <span className="vsm-timeline-duration">
                      {hasDuration ? (
                        <>
                          {formatDuration(segment.durationMinutes)}
                          <span className="vsm-timeline-pct">{segment.percentOfTotal}%</span>
                        </>
                      ) : (
                        t.vsm.timelineNoDuration
                      )}
                    </span>
                    {renderAuthor(segment.targetAuthor)}
                  </button>
                </li>
              );
            })}
          </ul>
        </div>
      )}
    </div>
  );
}
