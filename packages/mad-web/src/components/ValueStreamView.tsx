import {
  addEdge,
  Background,
  BackgroundVariant,
  Controls,
  MiniMap,
  Panel,
  ReactFlow,
  ReactFlowProvider,
  useEdgesState,
  useNodesState,
  useReactFlow,
  type Connection,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type {
  EvaluationReport,
  EvaluationWorkspace,
  ValueStreamEntry,
  ValueStreamMap,
  Vendor,
  VsmFlowTypeDef,
  VsmMessage,
  VsmNodeType,
} from "../types";
import {
  collectVsmAuthors,
  createVsmNode,
  DEFAULT_FLOW_TYPES,
  emptyValueStream,
  entryToMap,
  flowToVsm,
  builtinFlowTypeDefault,
  isBuiltinFlowType,
  isFlowTypeOverridden,
  mdmEnrollmentTemplate,
  newEdgeId,
  newMessageId,
  newNodeId,
  NODE_PALETTE,
  slugifyFlowTypeId,
  type VsmEdgeData,
  type VsmNodeData,
  valueStreamMapsEqual,
  vsmToFlow,
} from "../utils/valueStream";
import { VsmDurationInput } from "./vsm/VsmDurationInput";
import { vsmEdgeTypes } from "./vsm/VsmEdges";
import { VsmFlowTypesProvider } from "./vsm/VsmFlowTypesContext";
import { vsmNodeTypes } from "./vsm/VsmNodes";
import { VsmTimeline } from "./vsm/VsmTimeline";

interface ValueStreamViewProps {
  evaluation: EvaluationReport;
  valueStreams: Record<string, ValueStreamEntry[]>;
  saving?: boolean;
  onSave: (
    vendorId: string,
    streamId: string,
    name: string,
    map: ValueStreamMap,
  ) => Promise<void>;
  onCreate: (vendorId: string, name: string) => Promise<EvaluationWorkspace>;
  onDelete: (vendorId: string, streamId: string) => Promise<void>;
}

type InspectorTab = "properties" | "messages" | "legend";

function nodeDefaultLabel(type: VsmNodeType, t: ReturnType<typeof useLocale>["t"]): string {
  const map: Record<VsmNodeType, string> = {
    process: t.vsm.newProcess,
    decision: t.vsm.newDecision,
    info: t.vsm.newInfo,
    delay: t.vsm.newDelay,
    external: t.vsm.newExternal,
    customer: t.vsm.newCustomer,
    supplier: t.vsm.newSupplier,
    inventory: t.vsm.newInventory,
    kaizen: t.vsm.newKaizen,
  };
  return map[type];
}

function nodeTypeLabel(type: VsmNodeType, t: ReturnType<typeof useLocale>["t"]): string {
  const map: Record<VsmNodeType, string> = {
    process: t.vsm.paletteProcess,
    decision: t.vsm.paletteDecision,
    info: t.vsm.paletteInfo,
    delay: t.vsm.paletteDelay,
    external: t.vsm.paletteExternal,
    customer: t.vsm.paletteCustomer,
    supplier: t.vsm.paletteSupplier,
    inventory: t.vsm.paletteInventory,
    kaizen: t.vsm.paletteKaizen,
  };
  return map[type];
}

function displayFlowTypeLabel(
  type: VsmFlowTypeDef,
  t: ReturnType<typeof useLocale>["t"],
): string {
  if (type.id === "material") return t.vsm.edgeMaterial;
  if (type.id === "information") return t.vsm.edgeInformation;
  if (type.id === "electronic") return t.vsm.edgeElectronic;
  return type.label;
}

function nodeSize(node: Node): { width: number; height: number } {
  return {
    width: typeof node.style?.width === "number" ? node.style.width : 180,
    height: typeof node.style?.height === "number" ? node.style.height : 72,
  };
}

function ValueStreamEditor({
  vendor,
  initialMap,
  onSave,
  saving,
}: {
  vendor: Vendor;
  initialMap: ValueStreamMap;
  onSave: (map: ValueStreamMap) => Promise<void>;
  saving?: boolean;
}) {
  const { t } = useLocale();
  const { screenToFlowPosition, fitView, fitBounds } = useReactFlow();
  const initial = useMemo(() => vsmToFlow(initialMap), [initialMap]);
  const [nodes, setNodes, onNodesChange] = useNodesState(initial.nodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initial.edges);
  const [messages, setMessages] = useState<VsmMessage[]>(initialMap.messages);
  const [customFlowTypes, setCustomFlowTypes] = useState<VsmFlowTypeDef[]>(
    () => initialMap.flow_types ?? [],
  );
  const [newFlowTypeLabel, setNewFlowTypeLabel] = useState("");
  const [newFlowTypeColor, setNewFlowTypeColor] = useState("#607d8b");
  const [newFlowTypeDashed, setNewFlowTypeDashed] = useState(false);
  const [editingFlowTypeId, setEditingFlowTypeId] = useState<string | null>(null);
  const [editFlowTypeLabel, setEditFlowTypeLabel] = useState("");
  const [editFlowTypeColor, setEditFlowTypeColor] = useState("#607d8b");
  const [editFlowTypeDashed, setEditFlowTypeDashed] = useState(false);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [messageDraft, setMessageDraft] = useState("");
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>("properties");
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const skipSave = useRef(true);
  const prevVendorIdRef = useRef(vendor.id);
  const flowStateRef = useRef({
    nodes: initial.nodes,
    edges: initial.edges,
    messages: initialMap.messages,
    customFlowTypes: initialMap.flow_types ?? [],
  });

  const allFlowTypes = useMemo(() => {
    const merged = new Map(DEFAULT_FLOW_TYPES.map((type) => [type.id, { ...type }]));
    for (const custom of customFlowTypes) {
      const existing = merged.get(custom.id);
      merged.set(custom.id, existing ? { ...existing, ...custom } : { ...custom });
    }
    for (const edge of edges) {
      const edgeType = (edge.data as VsmEdgeData).edgeType ?? "material";
      if (!merged.has(edgeType)) {
        merged.set(edgeType, { id: edgeType, label: edgeType, color: "#78909c" });
      }
    }
    return [...merged.values()];
  }, [customFlowTypes, edges]);

  flowStateRef.current = { nodes, edges, messages, customFlowTypes };

  useEffect(() => {
    const vendorChanged = prevVendorIdRef.current !== vendor.id;
    prevVendorIdRef.current = vendor.id;

    if (!vendorChanged) {
      const localMap = flowToVsm(
        flowStateRef.current.nodes,
        flowStateRef.current.edges,
        flowStateRef.current.messages,
        flowStateRef.current.customFlowTypes,
      );
      if (valueStreamMapsEqual(localMap, initialMap)) {
        skipSave.current = true;
        return;
      }
    }

    const flow = vsmToFlow(initialMap);
    setNodes(flow.nodes);
    setEdges(flow.edges);
    setMessages(initialMap.messages);
    setCustomFlowTypes(initialMap.flow_types ?? []);
    skipSave.current = true;

    if (vendorChanged) {
      setSelectedNodeId(null);
      setSelectedEdgeId(null);
      requestAnimationFrame(() => fitView({ padding: 0.15, duration: 200 }));
    }
  }, [vendor.id, initialMap, setNodes, setEdges, fitView]);

  useEffect(() => {
    if (skipSave.current) {
      skipSave.current = false;
      return;
    }
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      onSave(flowToVsm(nodes, edges, messages, customFlowTypes));
    }, 900);
    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, [nodes, edges, messages, customFlowTypes, onSave]);

  const clearGraphFocus = useCallback(() => {
    setNodes((nds) =>
      nds.map((node) => ({ ...node, selected: false, className: undefined })),
    );
    setEdges((eds) => eds.map((edge) => ({ ...edge, selected: false })));
  }, [setNodes, setEdges]);

  const focusOnGraph = useCallback(
    (opts: { edgeId?: string; nodeId?: string }) => {
      if (opts.edgeId) {
        const edge = edges.find((item) => item.id === opts.edgeId);
        if (!edge) return;
        const source = nodes.find((node) => node.id === edge.source);
        const target = nodes.find((node) => node.id === edge.target);
        if (!source || !target) return;

        setNodes((nds) =>
          nds.map((node) => ({
            ...node,
            selected: node.id === edge.source || node.id === edge.target,
            className:
              node.id === edge.source || node.id === edge.target
                ? "vsm-graph-focus"
                : undefined,
          })),
        );
        setEdges((eds) =>
          eds.map((item) => ({ ...item, selected: item.id === opts.edgeId })),
        );

        const sourceSize = nodeSize(source);
        const targetSize = nodeSize(target);
        const x1 = source.position.x;
        const y1 = source.position.y;
        const x2 = target.position.x;
        const y2 = target.position.y;
        fitBounds(
          {
            x: Math.min(x1, x2) - 48,
            y: Math.min(y1, y2) - 48,
            width: Math.max(x1 + sourceSize.width, x2 + targetSize.width) - Math.min(x1, x2) + 96,
            height: Math.max(y1 + sourceSize.height, y2 + targetSize.height) - Math.min(y1, y2) + 96,
          },
          { padding: 0.2, duration: 350 },
        );
        return;
      }

      if (opts.nodeId) {
        const node = nodes.find((item) => item.id === opts.nodeId);
        if (!node) return;
        const size = nodeSize(node);
        setNodes((nds) =>
          nds.map((item) => ({
            ...item,
            selected: item.id === opts.nodeId,
            className: item.id === opts.nodeId ? "vsm-graph-focus" : undefined,
          })),
        );
        setEdges((eds) => eds.map((edge) => ({ ...edge, selected: false })));
        fitBounds(
          {
            x: node.position.x - 40,
            y: node.position.y - 40,
            width: size.width + 80,
            height: size.height + 80,
          },
          { padding: 0.22, duration: 350 },
        );
      }
    },
    [edges, nodes, fitBounds, setEdges, setNodes],
  );

  const onConnect = useCallback(
    (connection: Connection) => {
      setEdges((eds) =>
        addEdge(
          {
            ...connection,
            id: newEdgeId(),
            type: "vsm",
            data: { edgeType: "material", durationMinutes: null },
          },
          eds,
        ),
      );
    },
    [setEdges],
  );

  const addNode = (nodeType: VsmNodeType) => {
    const center = screenToFlowPosition({
      x: window.innerWidth / 2,
      y: window.innerHeight / 2,
    });
    const label = nodeDefaultLabel(nodeType, t);
    const vsmNode = createVsmNode(nodeType, label, center.x - 80, center.y - 28);
    const flowNode: Node = {
      id: vsmNode.id,
      type: nodeType,
      position: { x: vsmNode.x, y: vsmNode.y },
      data: {
        label: vsmNode.label,
        notes: "",
        role: "",
        author: "",
        leadTimeMinutes: null,
        cycleTimeMinutes: null,
      } satisfies VsmNodeData,
      style: { width: vsmNode.width, height: vsmNode.height },
    };
    setNodes((nds) => [...nds, flowNode]);
  };

  const deleteSelected = () => {
    if (selectedNodeId) {
      setNodes((nds) => nds.filter((n) => n.id !== selectedNodeId));
      setEdges((eds) =>
        eds.filter((e) => e.source !== selectedNodeId && e.target !== selectedNodeId),
      );
      setMessages((msgs) => msgs.filter((m) => m.node_id !== selectedNodeId));
      setSelectedNodeId(null);
    } else if (selectedEdgeId) {
      setEdges((eds) => eds.filter((e) => e.id !== selectedEdgeId));
      setMessages((msgs) => msgs.filter((m) => m.edge_id !== selectedEdgeId));
      setSelectedEdgeId(null);
    }
  };

  const duplicateSelected = () => {
    const node = nodes.find((n) => n.id === selectedNodeId);
    if (!node) return;
    const copy: Node = {
      ...node,
      id: newNodeId(),
      position: { x: node.position.x + 48, y: node.position.y + 48 },
      selected: false,
    };
    setNodes((nds) => [...nds, copy]);
    setSelectedNodeId(copy.id);
  };

  const clearAll = () => {
    if (nodes.length === 0 && edges.length === 0) return;
    if (!window.confirm(t.vsm.clearConfirm)) return;
    setNodes([]);
    setEdges([]);
    setMessages([]);
    setSelectedNodeId(null);
    setSelectedEdgeId(null);
  };

  const loadTemplate = () => {
    if (nodes.length > 0 && !window.confirm(t.vsm.templateConfirm)) return;
    const flow = vsmToFlow(mdmEnrollmentTemplate());
    setNodes(flow.nodes);
    setEdges(flow.edges);
    setMessages([]);
    setSelectedNodeId(null);
    setSelectedEdgeId(null);
    requestAnimationFrame(() => fitView({ padding: 0.15, duration: 300 }));
  };

  const selectedNode = nodes.find((n) => n.id === selectedNodeId);
  const selectedEdge = edges.find((e) => e.id === selectedEdgeId);
  const nodeData = selectedNode?.data as VsmNodeData | undefined;

  const updateNodeData = (patch: Partial<VsmNodeData>) => {
    if (!selectedNodeId) return;
    setNodes((nds) =>
      nds.map((n) =>
        n.id === selectedNodeId ? { ...n, data: { ...n.data, ...patch } } : n,
      ),
    );
  };

  const updateEdgeLabel = (label: string) => {
    if (!selectedEdgeId) return;
    setEdges((eds) =>
      eds.map((e) => (e.id === selectedEdgeId ? { ...e, label } : e)),
    );
  };

  const updateEdgeType = (edgeType: string) => {
    if (!selectedEdgeId) return;
    setEdges((eds) =>
      eds.map((e) =>
        e.id === selectedEdgeId ? { ...e, data: { ...e.data, edgeType } } : e,
      ),
    );
  };

  const addCustomFlowType = () => {
    const label = newFlowTypeLabel.trim();
    if (!label) return;
    const id = slugifyFlowTypeId(label);
    if (allFlowTypes.some((type) => type.id === id)) return;
    setCustomFlowTypes((prev) => [
      ...prev,
      {
        id,
        label,
        color: newFlowTypeColor,
        dash: newFlowTypeDashed ? "6 4" : undefined,
      },
    ]);
    setNewFlowTypeLabel("");
  };

  const upsertFlowType = (id: string, patch: Partial<VsmFlowTypeDef>) => {
    setCustomFlowTypes((prev) => {
      const existing = prev.find((type) => type.id === id);
      if (existing) {
        return prev.map((type) => (type.id === id ? { ...type, ...patch } : type));
      }
      const builtin = builtinFlowTypeDefault(id);
      const base = builtin ?? allFlowTypes.find((type) => type.id === id);
      if (!base) return prev;
      return [...prev, { ...base, ...patch }];
    });
  };

  const startEditFlowType = (flowType: VsmFlowTypeDef) => {
    setEditingFlowTypeId(flowType.id);
    setEditFlowTypeLabel(flowType.label);
    setEditFlowTypeColor(flowType.color);
    setEditFlowTypeDashed(Boolean(flowType.dash));
  };

  const saveFlowTypeEdit = () => {
    if (!editingFlowTypeId) return;
    const isBuiltin = isBuiltinFlowType(editingFlowTypeId);
    upsertFlowType(editingFlowTypeId, {
      id: editingFlowTypeId,
      label: isBuiltin
        ? (builtinFlowTypeDefault(editingFlowTypeId)?.label ?? editingFlowTypeId)
        : editFlowTypeLabel.trim() || editingFlowTypeId,
      color: editFlowTypeColor,
      dash: editFlowTypeDashed ? "6 4" : undefined,
    });
    setEditingFlowTypeId(null);
  };

  const resetFlowTypeToDefault = (id: string) => {
    setCustomFlowTypes((prev) => prev.filter((type) => type.id !== id));
    if (editingFlowTypeId === id) setEditingFlowTypeId(null);
  };

  const removeCustomFlowType = (id: string) => {
    if (isBuiltinFlowType(id)) return;
    setCustomFlowTypes((prev) => prev.filter((type) => type.id !== id));
    if (editingFlowTypeId === id) setEditingFlowTypeId(null);
    setEdges((eds) =>
      eds.map((edge) => {
        const data = edge.data as VsmEdgeData;
        return data.edgeType === id
          ? { ...edge, data: { ...data, edgeType: "material" } }
          : edge;
      }),
    );
  };

  const updateEdgeDuration = (durationMinutes: number | null) => {
    if (!selectedEdgeId) return;
    setEdges((eds) =>
      eds.map((e) =>
        e.id === selectedEdgeId ? { ...e, data: { ...e.data, durationMinutes } } : e,
      ),
    );
  };

  const selectEdge = (edgeId: string) => {
    setSelectedEdgeId(edgeId);
    setSelectedNodeId(null);
    setInspectorTab("properties");
    focusOnGraph({ edgeId });
  };

  const selectNode = (nodeId: string) => {
    setSelectedNodeId(nodeId);
    setSelectedEdgeId(null);
    setInspectorTab("properties");
    focusOnGraph({ nodeId });
  };

  const addMessage = () => {
    if (!messageDraft.trim()) return;
    const msg: VsmMessage = {
      id: newMessageId(),
      text: messageDraft.trim(),
      node_id: selectedNodeId ?? undefined,
      edge_id: selectedEdgeId ?? undefined,
    };
    setMessages((m) => [...m, msg]);
    setMessageDraft("");
  };

  const removeMessage = (id: string) => {
    setMessages((m) => m.filter((x) => x.id !== id));
  };

  const stats = useMemo(
    () => ({
      nodes: nodes.length,
      edges: edges.length,
      messages: messages.length,
    }),
    [nodes.length, edges.length, messages.length],
  );

  const knownAuthors = useMemo(() => collectVsmAuthors(nodes), [nodes]);

  return (
    <VsmFlowTypesProvider flowTypes={allFlowTypes}>
    <div className="vsm-layout">
      <div className="vsm-toolbar card">
        <div className="vsm-toolbar-left">
          <span className="vsm-vendor-name">{vendor.name}</span>
          <span className="vsm-stats">
            {stats.nodes} {t.vsm.statNodes} · {stats.edges} {t.vsm.statEdges}
          </span>
        </div>
        <div className="vsm-toolbar-actions">
          <button type="button" className="btn btn-secondary btn-sm" onClick={loadTemplate}>
            {t.vsm.loadTemplate}
          </button>
          <button
            type="button"
            className="btn btn-secondary btn-sm"
            onClick={() => fitView({ padding: 0.15, duration: 300 })}
          >
            {t.vsm.fitView}
          </button>
          <button
            type="button"
            className="btn btn-secondary btn-sm"
            onClick={duplicateSelected}
            disabled={!selectedNodeId}
          >
            {t.vsm.duplicate}
          </button>
          <button
            type="button"
            className="btn btn-danger btn-sm"
            onClick={deleteSelected}
            disabled={!selectedNodeId && !selectedEdgeId}
          >
            {t.vsm.deleteSelected}
          </button>
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            onClick={clearAll}
            disabled={nodes.length === 0}
          >
            {t.vsm.clearAll}
          </button>
          {saving && (
            <span className="saving-indicator">
              <span className="saving-dot" /> {t.vsm.saving}
            </span>
          )}
        </div>
      </div>

      <div className="vsm-main">
        <aside className="vsm-palette card">
          <h3>{t.vsm.palette}</h3>
          <p className="vsm-hint">{t.vsm.paletteHint}</p>
          <div className="vsm-palette-grid">
            {NODE_PALETTE.map((item) => (
              <button
                key={item.type}
                type="button"
                className="vsm-palette-item"
                style={{ "--vsm-accent": item.color } as React.CSSProperties}
                onClick={() => addNode(item.type)}
                title={nodeTypeLabel(item.type, t)}
              >
                <span className="vsm-palette-icon">{item.icon}</span>
                <span className="vsm-palette-label">{nodeTypeLabel(item.type, t)}</span>
              </button>
            ))}
          </div>
        </aside>

        <div className="vsm-canvas">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={vsmNodeTypes}
            edgeTypes={vsmEdgeTypes}
            defaultEdgeOptions={{
              type: "vsm",
              data: { edgeType: "material", durationMinutes: null },
            }}
            fitView
            snapToGrid
            snapGrid={[16, 16]}
            onNodeClick={(_, node) => {
              selectNode(node.id);
            }}
            onEdgeClick={(_, edge) => {
              selectEdge(edge.id);
            }}
            onPaneClick={() => {
              setSelectedNodeId(null);
              setSelectedEdgeId(null);
              clearGraphFocus();
            }}
          >
            <Background variant={BackgroundVariant.Dots} gap={20} size={1} color="#c5d0dc" />
            <Controls showInteractive={false} />
            <MiniMap
              zoomable
              pannable
              nodeColor={(n) =>
                NODE_PALETTE.find((p) => p.type === n.type)?.color ?? "#90a4ae"
              }
              maskColor="rgba(10, 22, 40, 0.08)"
            />
            <Panel position="top-left" className="vsm-canvas-hint">
              {t.vsm.canvasHint}
            </Panel>
          </ReactFlow>
        </div>

        <aside className="vsm-sidebar card">
          <div className="vsm-inspector-tabs">
            {(["properties", "messages", "legend"] as InspectorTab[]).map((tab) => (
              <button
                key={tab}
                type="button"
                className={`vsm-inspector-tab${inspectorTab === tab ? " active" : ""}`}
                onClick={() => setInspectorTab(tab)}
              >
                {tab === "properties"
                  ? t.vsm.tabProperties
                  : tab === "messages"
                    ? t.vsm.tabMessages
                    : t.vsm.tabLegend}
              </button>
            ))}
          </div>

          {inspectorTab === "properties" && (
            <div className="vsm-inspector-panel">
              {!selectedNode && !selectedEdge && (
                <p className="vsm-hint">{t.vsm.selectHint}</p>
              )}
              {selectedNode && nodeData && (
                <>
                  <div className="vsm-inspector-type">
                    {nodeTypeLabel((selectedNode.type as VsmNodeType) ?? "process", t)}
                  </div>
                  <label>
                    {t.vsm.nodeLabel}
                    <input
                      value={String(nodeData.label ?? "")}
                      onChange={(e) => updateNodeData({ label: e.target.value })}
                    />
                  </label>
                  <label>
                    {t.vsm.nodeAuthor}
                    <input
                      list="vsm-author-suggestions"
                      value={String(nodeData.author ?? "")}
                      onChange={(e) => updateNodeData({ author: e.target.value })}
                      placeholder={t.vsm.nodeAuthorPlaceholder}
                    />
                  </label>
                  <label>
                    {t.vsm.nodeRole}
                    <input
                      value={String(nodeData.role ?? "")}
                      onChange={(e) => updateNodeData({ role: e.target.value })}
                      placeholder={t.vsm.nodeRolePlaceholder}
                    />
                  </label>
                  <div className="vsm-time-row">
                    <label>
                      {t.vsm.leadTime}
                      <VsmDurationInput
                        inputKey={`${selectedNodeId}-lt`}
                        minutes={nodeData.leadTimeMinutes}
                        onChange={(value) => updateNodeData({ leadTimeMinutes: value })}
                      />
                    </label>
                    <label>
                      {t.vsm.cycleTime}
                      <VsmDurationInput
                        inputKey={`${selectedNodeId}-ct`}
                        minutes={nodeData.cycleTimeMinutes}
                        onChange={(value) => updateNodeData({ cycleTimeMinutes: value })}
                      />
                    </label>
                  </div>
                  <label>
                    {t.vsm.nodeNotes}
                    <textarea
                      rows={2}
                      value={String(nodeData.notes ?? "")}
                      onChange={(e) => updateNodeData({ notes: e.target.value })}
                      placeholder={t.vsm.nodeNotesPlaceholder}
                    />
                  </label>
                </>
              )}
              {selectedEdge && (
                <>
                  <label>
                    {t.vsm.edgeLabel}
                    <input
                      value={String(selectedEdge.label ?? "")}
                      onChange={(e) => updateEdgeLabel(e.target.value)}
                      placeholder={t.vsm.edgeLabelPlaceholder}
                    />
                  </label>
                  <label>
                    {t.vsm.edgeDuration}
                    <VsmDurationInput
                      inputKey={selectedEdgeId ?? undefined}
                      minutes={(selectedEdge.data as VsmEdgeData)?.durationMinutes}
                      onChange={updateEdgeDuration}
                    />
                  </label>
                  <div className="vsm-flow-type-field">
                    <span className="vsm-field-label">{t.vsm.edgeType}</span>
                    <div className="vsm-flow-type-picker">
                      {allFlowTypes.map((flowType) => {
                        const active =
                          ((selectedEdge.data as VsmEdgeData)?.edgeType ?? "material") ===
                          flowType.id;
                        return (
                          <button
                            key={flowType.id}
                            type="button"
                            className={`vsm-flow-type-chip${active ? " active" : ""}`}
                            style={
                              {
                                "--chip-color": flowType.color,
                                borderStyle: flowType.dash ? "dashed" : "solid",
                              } as React.CSSProperties
                            }
                            onClick={() => updateEdgeType(flowType.id)}
                            title={displayFlowTypeLabel(flowType, t)}
                          >
                            {displayFlowTypeLabel(flowType, t)}
                          </button>
                        );
                      })}
                    </div>
                  </div>
                </>
              )}
            </div>
          )}

          {inspectorTab === "messages" && (
            <div className="vsm-inspector-panel">
              <p className="vsm-hint">{t.vsm.messagesHint}</p>
              <textarea
                value={messageDraft}
                onChange={(e) => setMessageDraft(e.target.value)}
                rows={3}
                placeholder={t.vsm.messagePlaceholder}
              />
              <button
                type="button"
                className="btn btn-primary btn-sm"
                onClick={addMessage}
                disabled={!messageDraft.trim()}
              >
                {t.vsm.addMessage}
              </button>
              <ul className="vsm-message-list">
                {messages.length === 0 && (
                  <li className="vsm-message-empty">{t.vsm.noMessages}</li>
                )}
                {messages.map((m) => (
                  <li key={m.id}>
                    <span>{m.text}</span>
                    {(m.node_id || m.edge_id) && (
                      <code className="vsm-message-link">{m.node_id ?? m.edge_id}</code>
                    )}
                    <button
                      type="button"
                      className="btn btn-ghost btn-icon"
                      onClick={() => removeMessage(m.id)}
                      title={t.common.remove}
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {inspectorTab === "legend" && (
            <div className="vsm-inspector-panel vsm-legend">
              <h4>{t.vsm.legendNodes}</h4>
              <ul className="vsm-legend-list">
                {NODE_PALETTE.map((item) => (
                  <li key={item.type}>
                    <span
                      className="vsm-legend-swatch"
                      style={{ background: item.color }}
                    />
                    <span>{nodeTypeLabel(item.type, t)}</span>
                  </li>
                ))}
              </ul>
              <h4>{t.vsm.legendEdges}</h4>
              <ul className="vsm-legend-list vsm-flow-type-list">
                {allFlowTypes.map((flowType) => (
                  <li
                    key={flowType.id}
                    className={editingFlowTypeId === flowType.id ? "editing" : ""}
                  >
                    <span
                      className="vsm-legend-line"
                      style={{
                        borderColor: flowType.color,
                        borderStyle: flowType.dash ? "dashed" : "solid",
                      }}
                    />
                    <span className="vsm-legend-flow-label">
                      {displayFlowTypeLabel(flowType, t)}
                      {isFlowTypeOverridden(flowType) && (
                        <span className="vsm-flow-type-customized">{t.vsm.flowTypeCustomized}</span>
                      )}
                    </span>
                    <button
                      type="button"
                      className="btn btn-ghost btn-sm vsm-flow-type-edit"
                      onClick={() => startEditFlowType(flowType)}
                    >
                      {t.vsm.editFlowType}
                    </button>
                    {!isBuiltinFlowType(flowType.id) && (
                      <button
                        type="button"
                        className="btn btn-ghost btn-icon vsm-flow-type-remove"
                        onClick={() => removeCustomFlowType(flowType.id)}
                        title={t.common.remove}
                      >
                        ×
                      </button>
                    )}
                  </li>
                ))}
              </ul>
              {editingFlowTypeId && (
                <div className="vsm-edit-flow-type card">
                  <h4>{t.vsm.editFlowTypeTitle}</h4>
                  {!isBuiltinFlowType(editingFlowTypeId) && (
                    <label>
                      {t.vsm.flowTypeLabel}
                      <input
                        value={editFlowTypeLabel}
                        onChange={(e) => setEditFlowTypeLabel(e.target.value)}
                      />
                    </label>
                  )}
                  {isBuiltinFlowType(editingFlowTypeId) && (
                    <p className="vsm-hint">{displayFlowTypeLabel(
                      allFlowTypes.find((type) => type.id === editingFlowTypeId)!,
                      t,
                    )}</p>
                  )}
                  <div className="vsm-add-flow-type-row">
                    <label>
                      {t.vsm.flowTypeColor}
                      <input
                        type="color"
                        value={editFlowTypeColor}
                        onChange={(e) => setEditFlowTypeColor(e.target.value)}
                      />
                    </label>
                    <label className="vsm-flow-type-dash">
                      <input
                        type="checkbox"
                        checked={editFlowTypeDashed}
                        onChange={(e) => setEditFlowTypeDashed(e.target.checked)}
                      />
                      {t.vsm.flowTypeDashed}
                    </label>
                  </div>
                  <div className="vsm-edit-flow-type-actions">
                    <button
                      type="button"
                      className="btn btn-primary btn-sm"
                      onClick={saveFlowTypeEdit}
                    >
                      {t.vsm.saveFlowType}
                    </button>
                    {isBuiltinFlowType(editingFlowTypeId) &&
                      customFlowTypes.some((type) => type.id === editingFlowTypeId) && (
                        <button
                          type="button"
                          className="btn btn-ghost btn-sm"
                          onClick={() => resetFlowTypeToDefault(editingFlowTypeId)}
                        >
                          {t.vsm.resetFlowType}
                        </button>
                      )}
                    <button
                      type="button"
                      className="btn btn-ghost btn-sm"
                      onClick={() => setEditingFlowTypeId(null)}
                    >
                      {t.common.cancel}
                    </button>
                  </div>
                </div>
              )}
              <div className="vsm-add-flow-type">
                <p className="vsm-hint">{t.vsm.addFlowTypeHint}</p>
                <label>
                  {t.vsm.flowTypeLabel}
                  <input
                    value={newFlowTypeLabel}
                    onChange={(e) => setNewFlowTypeLabel(e.target.value)}
                    placeholder={t.vsm.flowTypeLabelPlaceholder}
                  />
                </label>
                <div className="vsm-add-flow-type-row">
                  <label>
                    {t.vsm.flowTypeColor}
                    <input
                      type="color"
                      value={newFlowTypeColor}
                      onChange={(e) => setNewFlowTypeColor(e.target.value)}
                    />
                  </label>
                  <label className="vsm-flow-type-dash">
                    <input
                      type="checkbox"
                      checked={newFlowTypeDashed}
                      onChange={(e) => setNewFlowTypeDashed(e.target.checked)}
                    />
                    {t.vsm.flowTypeDashed}
                  </label>
                </div>
                <button
                  type="button"
                  className="btn btn-secondary btn-sm"
                  onClick={addCustomFlowType}
                  disabled={!newFlowTypeLabel.trim()}
                >
                  {t.vsm.addFlowType}
                </button>
              </div>
            </div>
          )}
        </aside>
      </div>

      <datalist id="vsm-author-suggestions">
        {knownAuthors.map((author) => (
          <option key={author} value={author} />
        ))}
      </datalist>

      <VsmTimeline
        nodes={nodes}
        edges={edges}
        selectedEdgeId={selectedEdgeId}
        onSelectEdge={selectEdge}
        onSelectNode={selectNode}
      />
    </div>
    </VsmFlowTypesProvider>
  );
}

export function ValueStreamView({
  evaluation,
  valueStreams,
  saving,
  onSave,
  onCreate,
  onDelete,
}: ValueStreamViewProps) {
  const { t } = useLocale();
  const vendors = evaluation.vendors.map((v) => v.vendor);
  const [vendorId, setVendorId] = useState(vendors[0]?.id ?? "");
  const [streamId, setStreamId] = useState("");
  const [streamName, setStreamName] = useState("");
  const ensuredVendorRef = useRef<string | null>(null);

  const entries = vendorId ? (valueStreams[vendorId] ?? []) : [];
  const entryIds = entries.map((entry) => entry.id).join(",");
  const activeEntry = entries.find((entry) => entry.id === streamId);
  const onCreateRef = useRef(onCreate);
  onCreateRef.current = onCreate;

  useEffect(() => {
    if (vendors.length > 0 && !vendors.some((v) => v.id === vendorId)) {
      setVendorId(vendors[0].id);
    }
  }, [vendors, vendorId]);

  useEffect(() => {
    if (!vendorId) return;

    if (entries.length === 0) {
      if (ensuredVendorRef.current === vendorId) return;
      ensuredVendorRef.current = vendorId;
      void onCreateRef.current(vendorId, t.vsm.defaultStreamName).then((ws) => {
        const created = ws.value_streams?.[vendorId]?.at(-1);
        if (created) setStreamId(created.id);
      });
      return;
    }

    ensuredVendorRef.current = vendorId;
    if (!entries.some((entry) => entry.id === streamId)) {
      setStreamId(entries[0].id);
    }
  }, [vendorId, entryIds, streamId, entries, t.vsm.defaultStreamName]);

  useEffect(() => {
    setStreamName(activeEntry?.name ?? "");
  }, [activeEntry?.id, activeEntry?.name]);

  const vendor = vendors.find((v) => v.id === vendorId);
  const map = activeEntry ? entryToMap(activeEntry) : emptyValueStream();

  const handleNewStream = async () => {
    if (!vendorId) return;
    const name = t.vsm.newStreamName.replace("{n}", String(entries.length + 1));
    const ws = await onCreate(vendorId, name);
    const created = ws.value_streams?.[vendorId]?.at(-1);
    if (created) setStreamId(created.id);
  };

  const handleDeleteStream = async () => {
    if (!vendorId || !streamId) return;
    if (!window.confirm(t.vsm.deleteStreamConfirm)) return;
    await onDelete(vendorId, streamId);
    ensuredVendorRef.current = null;
  };

  const handleStreamNameBlur = () => {
    const trimmed = streamName.trim();
    if (!vendorId || !streamId || !trimmed || trimmed === activeEntry?.name) return;
    void onSave(vendorId, streamId, trimmed, map);
  };

  if (vendors.length === 0) {
    return (
      <section className="vsm-page">
        <h2 className="section-title">{t.vsm.title}</h2>
        <div className="empty-state card">
          <p className="empty-state-title">{t.vsm.noVendors}</p>
        </div>
      </section>
    );
  }

  return (
    <section className="vsm-page">
      <div className="toolbar">
        <div>
          <h2 className="section-title">{t.vsm.title}</h2>
          <p className="section-intro">{t.vsm.intro}</p>
        </div>
        <div className="vsm-toolbar-pickers">
          <label className="vsm-vendor-picker">
            {t.vsm.vendor}
            <select value={vendorId} onChange={(e) => setVendorId(e.target.value)}>
              {vendors.map((v) => (
                <option key={v.id} value={v.id}>
                  {v.name}
                </option>
              ))}
            </select>
          </label>
          {entries.length > 0 && (
            <label className="vsm-stream-picker">
              {t.vsm.stream}
              <select value={streamId} onChange={(e) => setStreamId(e.target.value)}>
                {entries.map((entry) => (
                  <option key={entry.id} value={entry.id}>
                    {entry.name}
                  </option>
                ))}
              </select>
            </label>
          )}
          <label className="vsm-stream-name">
            {t.vsm.streamName}
            <input
              value={streamName}
              onChange={(e) => setStreamName(e.target.value)}
              onBlur={handleStreamNameBlur}
              placeholder={t.vsm.defaultStreamName}
            />
          </label>
          <div className="vsm-stream-actions">
            <button
              type="button"
              className="btn btn-secondary btn-sm"
              onClick={() => void handleNewStream()}
              disabled={saving}
            >
              {t.vsm.newStream}
            </button>
            <button
              type="button"
              className="btn btn-ghost btn-sm"
              onClick={() => void handleDeleteStream()}
              disabled={saving || entries.length <= 1}
              title={entries.length <= 1 ? t.vsm.deleteStreamDisabled : t.vsm.deleteStream}
            >
              {t.vsm.deleteStream}
            </button>
          </div>
        </div>
      </div>

      {vendor && streamId && (
        <ReactFlowProvider>
          <ValueStreamEditor
            key={`${vendor.id}-${streamId}`}
            vendor={vendor}
            initialMap={map}
            onSave={(m) => onSave(vendor.id, streamId, streamName.trim() || activeEntry?.name || t.vsm.defaultStreamName, m)}
            saving={saving}
          />
        </ReactFlowProvider>
      )}
    </section>
  );
}
