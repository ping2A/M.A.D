import {
  BaseEdge,
  EdgeLabelRenderer,
  getSmoothStepPath,
  type EdgeProps,
} from "@xyflow/react";
import { memo } from "react";
import { useFormatDuration } from "../../i18n/useFormatDuration";
import { flowTypeConfig, type VsmEdgeData } from "../../utils/valueStream";
import { useVsmFlowTypes } from "./VsmFlowTypesContext";

export const VsmEdge = memo(
  ({
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    label,
    data,
    selected,
    markerEnd,
  }: EdgeProps) => {
    const flowTypes = useVsmFlowTypes();
    const formatDuration = useFormatDuration();
    const edgeData = data as VsmEdgeData | undefined;
    const edgeType = edgeData?.edgeType ?? "material";
    const cfg = flowTypeConfig(edgeType, flowTypes);
    const durationLabel =
      edgeData?.durationMinutes != null && edgeData.durationMinutes > 0
        ? formatDuration(edgeData.durationMinutes, "compact")
        : null;
    const displayLabel = [label, durationLabel].filter(Boolean).join(" · ");
    const [edgePath, labelX, labelY] = getSmoothStepPath({
      sourceX,
      sourceY,
      targetX,
      targetY,
      sourcePosition,
      targetPosition,
      borderRadius: 12,
    });

    return (
      <>
        <BaseEdge
          id={id}
          path={edgePath}
          markerEnd={markerEnd}
          style={{
            stroke: cfg.color,
            strokeWidth: selected ? 2.5 : 2,
            strokeDasharray: cfg.dash,
            opacity: selected ? 1 : 0.88,
          }}
        />
        {displayLabel && (
          <EdgeLabelRenderer>
            <div
              className={`vsm-edge-label${selected ? " vsm-edge-label-selected" : ""}`}
              style={{
                transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              }}
            >
              {displayLabel}
            </div>
          </EdgeLabelRenderer>
        )}
      </>
    );
  },
);
VsmEdge.displayName = "VsmEdge";

export const vsmEdgeTypes = {
  vsm: VsmEdge,
};
