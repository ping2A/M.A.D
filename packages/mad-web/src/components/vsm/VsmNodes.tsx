import { Handle, Position, type Node, type NodeProps } from "@xyflow/react";
import { memo } from "react";
import { formatDuration, NODE_PALETTE, type VsmNodeData } from "../../utils/valueStream";

type VsmFlowNode = Node<VsmNodeData>;
type VsmNodeProps = NodeProps<VsmFlowNode>;

function paletteColor(nodeType: string): string {
  return NODE_PALETTE.find((p) => p.type === nodeType)?.color ?? "#78909c";
}

function AuthorBadge({ author }: { author?: string }) {
  if (!author?.trim()) return null;
  return <div className="vsm-node-author">{author}</div>;
}

function MetricsStrip({ data }: { data: VsmNodeData }) {
  const lt = formatDuration(data.leadTimeMinutes, "compact");
  const ct = formatDuration(data.cycleTimeMinutes, "compact");
  if (!lt && !ct) return null;
  return (
    <div className="vsm-node-metrics">
      {lt && <span>LT {lt}</span>}
      {ct && <span>CT {ct}</span>}
    </div>
  );
}

function BaseVsmNode({
  className,
  data,
  selected,
  nodeType,
  children,
}: {
  className: string;
  data: VsmNodeData;
  selected?: boolean;
  nodeType: string;
  children?: React.ReactNode;
}) {
  const color = paletteColor(nodeType);
  return (
    <div
      className={`vsm-node ${className}${selected ? " vsm-node-selected" : ""}`}
      style={{ "--vsm-accent": color } as React.CSSProperties}
    >
      <Handle type="target" position={Position.Left} className="vsm-handle" />
      <div className="vsm-node-meta">
        {data.role && <div className="vsm-node-role">{data.role}</div>}
        <AuthorBadge author={data.author} />
      </div>
      <div className="vsm-node-label">{data.label}</div>
      {data.notes && <div className="vsm-node-notes">{data.notes}</div>}
      <MetricsStrip data={data} />
      {children}
      <Handle type="source" position={Position.Right} className="vsm-handle" />
    </div>
  );
}

export const ProcessNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-process" data={data} selected={selected} nodeType="process" />
));
ProcessNode.displayName = "ProcessNode";

export const DecisionNode = memo(({ data, selected }: VsmNodeProps) => (
  <div
    className={`vsm-node vsm-node-decision-wrap${selected ? " vsm-node-selected" : ""}`}
    style={{ "--vsm-accent": paletteColor("decision") } as React.CSSProperties}
  >
    <Handle type="target" position={Position.Top} className="vsm-handle" />
    <AuthorBadge author={data.author} />
    <div className="vsm-node-decision">
      <span>{data.label}</span>
    </div>
    <MetricsStrip data={data} />
    <Handle type="source" position={Position.Bottom} id="yes" className="vsm-handle" />
    <Handle type="source" position={Position.Right} id="no" className="vsm-handle" />
  </div>
));
DecisionNode.displayName = "DecisionNode";

export const InfoNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-info" data={data} selected={selected} nodeType="info">
    <span className="vsm-info-icon">≋</span>
  </BaseVsmNode>
));
InfoNode.displayName = "InfoNode";

export const DelayNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-delay" data={data} selected={selected} nodeType="delay" />
));
DelayNode.displayName = "DelayNode";

export const ExternalNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-external" data={data} selected={selected} nodeType="external" />
));
ExternalNode.displayName = "ExternalNode";

export const CustomerNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-customer" data={data} selected={selected} nodeType="customer">
    <span className="vsm-shape-icon">👤</span>
  </BaseVsmNode>
));
CustomerNode.displayName = "CustomerNode";

export const SupplierNode = memo(({ data, selected }: VsmNodeProps) => (
  <BaseVsmNode className="vsm-node-supplier" data={data} selected={selected} nodeType="supplier">
    <span className="vsm-shape-icon">🏭</span>
  </BaseVsmNode>
));
SupplierNode.displayName = "SupplierNode";

export const InventoryNode = memo(({ data, selected }: VsmNodeProps) => (
  <div
    className={`vsm-node vsm-node-inventory-wrap${selected ? " vsm-node-selected" : ""}`}
    style={{ "--vsm-accent": paletteColor("inventory") } as React.CSSProperties}
  >
    <Handle type="target" position={Position.Left} className="vsm-handle" />
    <div className="vsm-node-inventory">
      <span>{data.label}</span>
      <AuthorBadge author={data.author} />
    </div>
    <MetricsStrip data={data} />
    <Handle type="source" position={Position.Right} className="vsm-handle" />
  </div>
));
InventoryNode.displayName = "InventoryNode";

export const KaizenNode = memo(({ data, selected }: VsmNodeProps) => (
  <div
    className={`vsm-node vsm-node-kaizen-wrap${selected ? " vsm-node-selected" : ""}`}
    style={{ "--vsm-accent": paletteColor("kaizen") } as React.CSSProperties}
  >
    <Handle type="target" position={Position.Top} className="vsm-handle" />
    <div className="vsm-node-kaizen">
      <span className="vsm-kaizen-icon">✦</span>
      <span className="vsm-kaizen-label">{data.label}</span>
      <AuthorBadge author={data.author} />
    </div>
    <Handle type="source" position={Position.Bottom} className="vsm-handle" />
  </div>
));
KaizenNode.displayName = "KaizenNode";

export const vsmNodeTypes = {
  process: ProcessNode,
  decision: DecisionNode,
  info: InfoNode,
  delay: DelayNode,
  external: ExternalNode,
  customer: CustomerNode,
  supplier: SupplierNode,
  inventory: InventoryNode,
  kaizen: KaizenNode,
};
