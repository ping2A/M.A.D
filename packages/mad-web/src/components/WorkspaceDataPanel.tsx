import { useRef, useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type { VendorImportMode } from "../types";

type ImportKind = "full" | "vendors" | "unknown";

function detectImportKind(data: unknown): ImportKind {
  if (typeof data !== "object" || data === null) return "unknown";
  const obj = data as Record<string, unknown>;
  if ("workspace" in obj || "pillars" in obj) return "full";
  if ("vendors" in obj) return "vendors";
  return "unknown";
}

interface WorkspaceDataPanelProps {
  onExportWorkspace: () => Promise<void>;
  onExportVendors: () => Promise<void>;
  onImportWorkspace: (
    json: string,
    vendorMode: VendorImportMode,
  ) => Promise<{ kind: string; pillars: number; requirements: number; vendors: number }>;
  stats?: { pillars: number; requirements: number; vendors: number };
}

export function WorkspaceDataPanel({
  onExportWorkspace,
  onExportVendors,
  onImportWorkspace,
  stats,
}: WorkspaceDataPanelProps) {
  const { t, format } = useLocale();
  const fileRef = useRef<HTMLInputElement>(null);
  const pendingVendorMode = useRef<VendorImportMode>("replace");
  const [collapsed, setCollapsed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [lastMessage, setLastMessage] = useState<string | null>(null);

  const openImport = (vendorMode: VendorImportMode) => {
    pendingVendorMode.current = vendorMode;
    fileRef.current?.click();
  };

  const handleFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file) return;

    setBusy(true);
    setLastMessage(null);
    try {
      const text = await file.text();
      let kind: ImportKind = "unknown";
      try {
        kind = detectImportKind(JSON.parse(text));
      } catch {
        alert(t.workspaceData.importError);
        return;
      }

      if (kind === "full") {
        if (!confirm(t.workspaceData.importFullConfirm)) return;
        const result = await onImportWorkspace(text, "replace");
        setLastMessage(
          format(t.workspaceData.importFullSuccess, {
            pillars: result.pillars,
            requirements: result.requirements,
            vendors: result.vendors,
          }),
        );
      } else if (kind === "vendors") {
        const mode = pendingVendorMode.current;
        if (mode === "replace" && !confirm(t.workspaceData.importVendorsReplaceConfirm)) {
          return;
        }
        const result = await onImportWorkspace(text, mode);
        setLastMessage(
          format(t.workspaceData.importVendorsSuccess, {
            vendors: result.vendors,
          }),
        );
      } else {
        alert(t.workspaceData.importError);
      }
    } catch {
      alert(t.workspaceData.importError);
    } finally {
      setBusy(false);
    }
  };

  const handleExportWorkspace = async () => {
    setBusy(true);
    try {
      await onExportWorkspace();
      setLastMessage(t.workspaceData.exportSuccess);
    } catch {
      alert(t.workspaceData.exportError);
    } finally {
      setBusy(false);
    }
  };

  const handleExportVendors = async () => {
    setBusy(true);
    try {
      await onExportVendors();
    } catch {
      alert(t.workspaceData.exportError);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={`workspace-data ${collapsed ? "collapsed" : ""}`}>
      <div className="workspace-data-header">
        <div>
          <h3>{t.workspaceData.title}</h3>
          {stats && (
            <p className="workspace-data-stats">
              {format(t.workspaceData.currentStats, {
                pillars: stats.pillars,
                requirements: stats.requirements,
                vendors: stats.vendors,
              })}
            </p>
          )}
        </div>
        <button
          type="button"
          className="toggle-btn"
          onClick={() => setCollapsed(!collapsed)}
          aria-expanded={!collapsed}
        >
          {collapsed ? t.common.expand : t.common.collapse}
        </button>
      </div>

      {!collapsed && (
        <div className="workspace-data-body">
          <p className="workspace-data-intro">{t.workspaceData.intro}</p>

          <div className="workspace-data-primary">
            <button
              type="button"
              className="btn btn-primary"
              onClick={handleExportWorkspace}
              disabled={busy}
            >
              {t.workspaceData.exportFull}
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={() => openImport("replace")}
              disabled={busy}
            >
              {t.workspaceData.importFull}
            </button>
          </div>

          <details className="workspace-data-advanced">
            <summary>{t.workspaceData.advancedTitle}</summary>
            <p className="workspace-data-hint">{t.workspaceData.advancedHint}</p>
            <div className="workspace-data-secondary">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={handleExportVendors}
                disabled={busy}
              >
                {t.workspaceData.exportVendorsOnly}
              </button>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => openImport("merge")}
                disabled={busy}
              >
                {t.workspaceData.importVendorsMerge}
              </button>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => openImport("replace")}
                disabled={busy}
              >
                {t.workspaceData.importVendorsReplace}
              </button>
            </div>
          </details>

          {lastMessage && <p className="workspace-data-message">{lastMessage}</p>}

          <input
            ref={fileRef}
            type="file"
            accept="application/json,.json"
            className="sr-only"
            onChange={handleFile}
          />
        </div>
      )}

      <style>{`
        .workspace-data {
          background: white;
          border-radius: var(--mad-radius);
          padding: 1rem 1.25rem;
          margin-bottom: 1rem;
          box-shadow: var(--mad-shadow);
          border-left: 4px solid var(--mad-cyan);
        }
        .workspace-data-header {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 1rem;
        }
        .workspace-data-header h3 {
          margin: 0;
          color: var(--mad-navy);
          font-size: 0.95rem;
        }
        .workspace-data-stats {
          margin: 0.25rem 0 0;
          font-size: 0.78rem;
          color: var(--mad-text-muted);
        }
        .workspace-data-intro, .workspace-data-hint {
          margin: 0 0 0.75rem;
          font-size: 0.82rem;
          color: var(--mad-text-muted);
          line-height: 1.5;
        }
        .workspace-data-primary, .workspace-data-secondary {
          display: flex;
          flex-wrap: wrap;
          gap: 0.5rem;
        }
        .workspace-data-advanced {
          margin-top: 1rem;
          font-size: 0.85rem;
          color: var(--mad-navy);
        }
        .workspace-data-advanced summary {
          cursor: pointer;
          font-weight: 600;
          margin-bottom: 0.5rem;
        }
        .workspace-data-message {
          margin: 0.75rem 0 0;
          font-size: 0.82rem;
          color: var(--mad-compliant);
          font-weight: 600;
        }
        .sr-only {
          position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px;
          overflow: hidden; clip: rect(0,0,0,0); border: 0;
        }
      `}</style>
    </div>
  );
}
