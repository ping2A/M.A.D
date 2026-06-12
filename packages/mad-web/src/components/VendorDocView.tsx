import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { VendorDocColorPicker } from "./VendorDocColorPicker";
import { useLocale } from "../i18n/LocaleContext";
import type { EvaluationReport, EvaluationWorkspace, VendorDocItem, VendorDocSection } from "../types";
import {
  countHighlighted,
  docColorLabel,
  hasDuplicateItemIds,
  itemCardStyle,
  mdmPrivacyTemplateSection,
  newVendorDocItem,
  normalizeDocColor,
  normalizeVendorDocSection,
  resolveDocColorHex,
  uniqueGroups,
} from "../utils/vendorDoc";

interface VendorDocViewProps {
  evaluation: EvaluationReport;
  vendorDocs: Record<string, VendorDocSection[]>;
  saving?: boolean;
  onSave: (
    vendorId: string,
    docId: string,
    name: string,
    section: Omit<VendorDocSection, "id" | "name">,
  ) => Promise<void>;
  onCreate: (vendorId: string, name: string) => Promise<EvaluationWorkspace>;
  onDelete: (vendorId: string, docId: string) => Promise<void>;
}

type GroupFilter = "all" | string;

export function VendorDocView({
  evaluation,
  vendorDocs,
  saving,
  onSave,
  onCreate,
  onDelete,
}: VendorDocViewProps) {
  const { t } = useLocale();
  const vendors = evaluation.vendors.map((v) => v.vendor);
  const [vendorId, setVendorId] = useState(vendors[0]?.id ?? "");
  const [docId, setDocId] = useState("");
  const [filter, setFilter] = useState<GroupFilter>("all");
  const [compactView, setCompactView] = useState(false);
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const sections = vendorId ? (vendorDocs[vendorId] ?? []) : [];
  const activeSection = sections.find((section) => section.id === docId);
  const [local, setLocal] = useState<VendorDocSection | null>(activeSection ?? null);

  useEffect(() => {
    if (!vendorId) return;
    const entries = vendorDocs[vendorId] ?? [];
    if (!entries.some((section) => section.id === docId)) {
      setDocId(entries[0]?.id ?? "");
    }
  }, [vendorId, vendorDocs, docId]);

  useEffect(() => {
    const raw = sections.find((entry) => entry.id === docId) ?? null;
    if (!raw) {
      setLocal(null);
      return;
    }
    const needsDedupe = hasDuplicateItemIds(raw.items);
    const section = needsDedupe ? normalizeVendorDocSection(raw) : raw;
    setLocal(section);
    setFilter("all");
    setExpandedIds(new Set());
    if (needsDedupe && vendorId) {
      void onSave(vendorId, section.id, section.name, {
        color: section.color,
        overview: section.overview,
        items: section.items,
      });
    }
  }, [docId, sections, vendorId, onSave]);

  useEffect(
    () => () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    },
    [],
  );

  const scheduleSave = useCallback(
    (updater: VendorDocSection | ((prev: VendorDocSection) => VendorDocSection)) => {
      setLocal((prev) => {
        if (!prev) return prev;
        const next =
          typeof updater === "function"
            ? normalizeVendorDocSection(updater(prev))
            : normalizeVendorDocSection(updater);
        if (saveTimer.current) clearTimeout(saveTimer.current);
        saveTimer.current = setTimeout(() => {
          void onSave(vendorId, next.id, next.name, {
            color: next.color,
            overview: next.overview,
            items: next.items,
          });
        }, 600);
        return next;
      });
    },
    [vendorId, onSave],
  );

  const selectedVendor = vendors.find((v) => v.id === vendorId);

  const groupTabs = useMemo(() => {
    if (!local) return ["all"];
    const groups = uniqueGroups(local.items).filter(Boolean);
    return ["all", ...groups];
  }, [local]);

  const filteredItems = useMemo(() => {
    if (!local) return [];
    if (filter === "all") return local.items;
    return local.items.filter((item) => (item.group?.trim() ?? "") === filter);
  }, [filter, local]);

  const highlightedCount = local ? countHighlighted(local.items) : 0;

  const updateSectionMeta = (patch: Partial<Pick<VendorDocSection, "name" | "overview" | "color">>) => {
    scheduleSave((prev) => ({ ...prev, ...patch }));
  };

  const updateItem = (id: string, patch: Partial<VendorDocItem>) => {
    scheduleSave((prev) => ({
      ...prev,
      items: prev.items.map((item) => (item.id === id ? { ...item, ...patch } : item)),
    }));
  };

  const removeItem = (id: string) => {
    scheduleSave((prev) => ({
      ...prev,
      items: prev.items.filter((item) => item.id !== id),
    }));
  };

  const addItem = () => {
    const defaultGroup = filter !== "all" ? filter : "";
    const item = { ...newVendorDocItem(), group: defaultGroup };
    scheduleSave((prev) => ({
      ...prev,
      items: [...prev.items, item],
    }));
    setExpandedIds((prev) => new Set(prev).add(item.id));
  };

  const handleCompactViewChange = (enabled: boolean) => {
    setCompactView(enabled);
    setExpandedIds(new Set());
  };

  const toggleExpanded = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleCreateSection = async () => {
    const name = prompt(t.vendorDocs.newSectionPrompt, t.vendorDocs.defaultSectionName);
    if (!name?.trim() || !vendorId) return;
    const ws = await onCreate(vendorId, name.trim());
    const created = ws.vendor_docs?.[vendorId]?.at(-1);
    if (created) setDocId(created.id);
  };

  const handleCreatePrivacyTemplate = async () => {
    if (!vendorId) return;
    const template = mdmPrivacyTemplateSection();
    const ws = await onCreate(vendorId, template.name);
    const created = ws.vendor_docs?.[vendorId]?.at(-1);
    if (!created) return;
    setDocId(created.id);
    await onSave(vendorId, created.id, template.name, {
      color: template.color,
      overview: template.overview,
      items: template.items,
    });
  };

  const handleDeleteSection = async () => {
    if (!vendorId || !docId || !local) return;
    if (!confirm(t.vendorDocs.deleteSectionConfirm)) return;
    await onDelete(vendorId, docId);
  };

  if (vendors.length === 0) {
    return (
      <section className="section vendor-doc-page">
        <p className="muted">{t.vendorDocs.noVendors}</p>
      </section>
    );
  }

  return (
    <section className="section vendor-doc-page">
      <div className="vendor-doc-header">
        <div>
          <h2 className="section-title">{t.vendorDocs.title}</h2>
          <p className="section-intro">{t.vendorDocs.intro}</p>
        </div>
        <span className="vendor-doc-badge">{t.vendorDocs.notScored}</span>
      </div>

      <div className="vendor-doc-shell">
        <div className="vendor-doc-toolbar-card">
          <div className="vendor-doc-toolbar-pickers">
            <label className="vendor-doc-field">
              <span>{t.vendorDocs.vendor}</span>
              <select value={vendorId} onChange={(e) => setVendorId(e.target.value)}>
                {vendors.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.name}
                  </option>
                ))}
              </select>
            </label>

            <label className="vendor-doc-field">
              <span>{t.vendorDocs.sectionName}</span>
              <input
                type="text"
                value={local?.name ?? ""}
                placeholder={t.vendorDocs.sectionNamePlaceholder}
                onChange={(e) => updateSectionMeta({ name: e.target.value })}
              />
            </label>

            {local && (
              <VendorDocColorPicker
                value={local.color}
                onChange={(color) => updateSectionMeta({ color })}
                compact
              />
            )}
          </div>

          <div className="vendor-doc-toolbar-actions">
            <button type="button" className="btn btn-secondary btn-sm" onClick={() => void handleCreateSection()}>
              {t.vendorDocs.newSection}
            </button>
            <button
              type="button"
              className="btn btn-secondary btn-sm"
              onClick={() => void handleCreatePrivacyTemplate()}
            >
              {t.vendorDocs.addPrivacyExample}
            </button>
            <button
              type="button"
              className="btn btn-danger btn-sm"
              onClick={() => void handleDeleteSection()}
              disabled={!docId}
            >
              {t.vendorDocs.deleteSection}
            </button>
            {saving && <span className="vendor-doc-saving">{t.vendorDocs.saving}</span>}
          </div>
        </div>

        {sections.length > 1 && (
          <div className="vendor-doc-section-tabs">
            {sections.map((section) => {
              const hex = resolveDocColorHex(section.color);
              return (
                <button
                  key={section.id}
                  type="button"
                  className={`vendor-doc-section-tab${section.id === docId ? " active" : ""}`}
                  onClick={() => setDocId(section.id)}
                >
                  {hex && (
                    <span className="vendor-doc-section-dot" style={{ background: hex }} />
                  )}
                  {section.name || t.vendorDocs.unnamedSection}
                </button>
              );
            })}
          </div>
        )}

        {selectedVendor && (
          <p className="vendor-doc-vendor-name muted">{selectedVendor.description}</p>
        )}

        {!local ? (
          <p className="muted vendor-doc-empty">{t.vendorDocs.noSections}</p>
        ) : (
          <div className="vendor-doc-body">
            <div className="vendor-doc-overview-card">
              <label>
                <span>{t.vendorDocs.overview}</span>
                <textarea
                  rows={3}
                  value={local.overview ?? ""}
                  placeholder={t.vendorDocs.overviewPlaceholder}
                  onChange={(e) => updateSectionMeta({ overview: e.target.value })}
                />
              </label>
              <div className="vendor-doc-stats">
                <span>{t.vendorDocs.itemCount.replace("{count}", String(local.items.length))}</span>
                {highlightedCount > 0 && (
                  <span className="vendor-doc-stats-highlight">
                    {t.vendorDocs.highlightedCount.replace(
                      "{count}",
                      String(highlightedCount),
                    )}
                  </span>
                )}
              </div>
            </div>

            <div className="vendor-doc-items-panel">
              <div className="vendor-doc-items-toolbar">
                <div className="vendor-doc-filter-tabs">
                  {groupTabs.map((key) => {
                    const count =
                      key === "all"
                        ? local.items.length
                        : local.items.filter((i) => (i.group?.trim() ?? "") === key).length;
                    const hasHighlight =
                      key === "all"
                        ? highlightedCount > 0
                        : local.items.some(
                            (i) =>
                              (i.group?.trim() ?? "") === key && normalizeDocColor(i.color),
                          );
                    return (
                      <button
                        key={key}
                        type="button"
                        className={`vendor-doc-filter-tab${filter === key ? " active" : ""}`}
                        onClick={() => setFilter(key)}
                      >
                        {key === "all" ? t.vendorDocs.filterAll : key}
                        <span className="vendor-doc-filter-count">{count}</span>
                        {hasHighlight && <span className="vendor-doc-filter-dot" />}
                      </button>
                    );
                  })}
                </div>
                <div className="vendor-doc-items-actions">
                  <label className="vendor-doc-compact-toggle">
                    <input
                      type="checkbox"
                      checked={compactView}
                      onChange={(e) => handleCompactViewChange(e.target.checked)}
                    />
                    {t.vendorDocs.compactView}
                  </label>
                  <button type="button" className="btn btn-secondary btn-sm" onClick={addItem}>
                    {t.vendorDocs.addItem}
                  </button>
                </div>
              </div>

              <div className="vendor-doc-items">
                {filteredItems.length === 0 ? (
                  <p className="muted vendor-doc-empty">{t.vendorDocs.emptyItems}</p>
                ) : (
                  filteredItems.map((item) => {
                    const expanded = compactView ? expandedIds.has(item.id) : true;
                    const hex = resolveDocColorHex(item.color);
                    const descriptionPreview = item.description?.trim() ?? "";
                    const notesPreview = item.notes?.trim() ?? "";
                    return (
                      <article
                        key={item.id}
                        className={`vendor-doc-item-card${hex ? " highlighted" : ""}${expanded ? " expanded" : " collapsed"}${compactView ? " compact-mode" : ""}`}
                        style={itemCardStyle(item.color)}
                      >
                        <div className="vendor-doc-item-head">
                          <button
                            type="button"
                            className="vendor-doc-item-summary"
                            onClick={() => compactView && toggleExpanded(item.id)}
                            aria-expanded={expanded}
                          >
                            {compactView && (
                              <span className="vendor-doc-item-chevron" aria-hidden>
                                {expanded ? "▾" : "▸"}
                              </span>
                            )}
                            {hex && (
                              <span
                                className="vendor-doc-item-accent"
                                style={{ background: hex }}
                                title={docColorLabel(item.color, t.vendorDocs.colors)}
                              />
                            )}
                            <span className="vendor-doc-item-title-preview">
                              {item.title.trim() || t.vendorDocs.untitledItem}
                            </span>
                            {item.group?.trim() && (
                              <span className="vendor-doc-item-group-pill">{item.group}</span>
                            )}
                            {hex && (
                              <span className="vendor-doc-item-color-label">
                                {docColorLabel(item.color, t.vendorDocs.colors)}
                              </span>
                            )}
                            {compactView && !expanded && descriptionPreview && (
                              <span className="vendor-doc-item-preview muted">
                                {descriptionPreview}
                              </span>
                            )}
                            {compactView && !expanded && notesPreview && (
                              <span className="vendor-doc-item-notes-preview">
                                {notesPreview}
                              </span>
                            )}
                          </button>
                          <button
                            type="button"
                            className="btn btn-danger btn-sm"
                            onClick={() => removeItem(item.id)}
                          >
                            {t.vendorDocs.removeItem}
                          </button>
                        </div>

                        {expanded && (
                          <div className="vendor-doc-item-fields">
                            <VendorDocColorPicker
                              value={item.color}
                              onChange={(color) => updateItem(item.id, { color })}
                              compact
                            />
                            <div className="vendor-doc-field-grid">
                              <label>
                                <span>{t.vendorDocs.itemGroup}</span>
                                <input
                                  type="text"
                                  value={item.group ?? ""}
                                  placeholder={t.vendorDocs.itemGroupPlaceholder}
                                  onChange={(e) => updateItem(item.id, { group: e.target.value })}
                                />
                              </label>
                              <label>
                                <span>{t.vendorDocs.itemTitle}</span>
                                <input
                                  type="text"
                                  value={item.title}
                                  onChange={(e) => updateItem(item.id, { title: e.target.value })}
                                />
                              </label>
                            </div>
                            <label>
                              <span>{t.vendorDocs.itemDescription}</span>
                              <textarea
                                rows={2}
                                value={item.description ?? ""}
                                onChange={(e) =>
                                  updateItem(item.id, { description: e.target.value })
                                }
                              />
                            </label>
                            <label>
                              <span>{t.vendorDocs.itemNotes}</span>
                              <textarea
                                rows={3}
                                value={item.notes ?? ""}
                                placeholder={t.vendorDocs.itemNotesPlaceholder}
                                onChange={(e) => updateItem(item.id, { notes: e.target.value })}
                              />
                            </label>
                          </div>
                        )}
                      </article>
                    );
                  })
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
