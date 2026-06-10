import { useState } from "react";
import { useLocale } from "../i18n/LocaleContext";
import type { BillingPeriod, EvaluationReport, Vendor, VendorPricing } from "../types";
import { formatTagsInput, parseTagsInput } from "../utils/comparisonFilter";
import { formatMoney, hasPricingInput, pricingSummary } from "../utils/pricing";

interface VendorEditorProps {
  evaluation: EvaluationReport;
  onAdd: (vendor: Vendor) => Promise<void>;
  onUpdate: (id: string, vendor: Vendor) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
}

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

export function VendorEditor({
  evaluation,
  onAdd,
  onUpdate,
  onDelete,
}: VendorEditorProps) {
  const { t, format } = useLocale();
  const vendors = evaluation.vendors.map((v) => v.vendor);
  const [mode, setMode] = useState<"closed" | "add" | "edit">("closed");
  const [editId, setEditId] = useState<string | null>(null);
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [website, setWebsite] = useState("");
  const [enablePricing, setEnablePricing] = useState(false);
  const [currency, setCurrency] = useState("USD");
  const [billingPeriod, setBillingPeriod] = useState<BillingPeriod>("monthly");
  const [pricePerDevice, setPricePerDevice] = useState("");
  const [globalPrice, setGlobalPrice] = useState("");
  const [pricingNotes, setPricingNotes] = useState("");
  const [tagsInput, setTagsInput] = useState("");
  const [saving, setSaving] = useState(false);

  const loadPricing = (pricing?: VendorPricing | null) => {
    if (pricing && hasPricingInput(pricing)) {
      setEnablePricing(true);
      setCurrency(pricing.currency || "USD");
      setBillingPeriod(pricing.billing_period || "monthly");
      setPricePerDevice(pricing.price_per_device?.toString() ?? "");
      setGlobalPrice(pricing.global_price?.toString() ?? "");
      setPricingNotes(pricing.notes ?? "");
    } else {
      setEnablePricing(false);
      setCurrency("USD");
      setBillingPeriod("monthly");
      setPricePerDevice("");
      setGlobalPrice("");
      setPricingNotes("");
    }
  };

  const resetForm = () => {
    setId("");
    setName("");
    setDescription("");
    setWebsite("");
    loadPricing(null);
    setTagsInput("");
    setEditId(null);
    setMode("closed");
  };

  const openAdd = () => {
    resetForm();
    setMode("add");
  };

  const openEdit = (vendor: Vendor) => {
    setEditId(vendor.id);
    setId(vendor.id);
    setName(vendor.name);
    setDescription(vendor.description);
    setWebsite(vendor.website ?? "");
    loadPricing(vendor.pricing);
    setTagsInput(formatTagsInput(vendor.tags));
    setMode("edit");
  };

  const buildPricing = (): VendorPricing | null => {
    if (!enablePricing) return null;
    const per = pricePerDevice.trim() ? Number(pricePerDevice) : null;
    const global = globalPrice.trim() ? Number(globalPrice) : null;
    if (per == null && global == null) return null;
    return {
      currency: currency.trim() || "USD",
      billing_period: billingPeriod,
      price_per_device: per,
      global_price: global,
      notes: pricingNotes.trim() || null,
    };
  };

  const handleNameChange = (value: string) => {
    setName(value);
    if (mode === "add" && !id) {
      setId(slugify(value));
    }
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id.trim() || !name.trim()) return;
    setSaving(true);
    try {
      const vendor: Vendor = {
        id: id.trim(),
        name: name.trim(),
        description: description.trim(),
        website: website.trim() || null,
        pricing: buildPricing(),
        tags: parseTagsInput(tagsInput),
      };
      if (mode === "edit" && editId) {
        await onUpdate(editId, vendor);
      } else {
        await onAdd(vendor);
      }
      resetForm();
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="vendor-editor">
      <div className="toolbar">
        <div>
          <h2 className="section-title">{t.vendors.title}</h2>
          <p className="section-intro">{t.vendors.intro}</p>
        </div>
        {mode === "closed" && (
          <button type="button" className="btn btn-primary" onClick={openAdd}>
            {t.vendors.add}
          </button>
        )}
      </div>

      {(mode === "add" || mode === "edit") && (
        <form className="form-panel" onSubmit={submit}>
          <h3>{mode === "edit" ? t.vendors.edit : t.vendors.new}</h3>
          <div className="form-row">
            <label>
              {t.common.id}
              <input
                value={id}
                onChange={(e) => setId(e.target.value)}
                placeholder={t.vendors.placeholderId}
                required
                disabled={mode === "edit"}
              />
            </label>
            <label>
              {t.common.name}
              <input
                value={name}
                onChange={(e) => handleNameChange(e.target.value)}
                placeholder={t.vendors.placeholderName}
                required
              />
            </label>
          </div>
          <label>
            {t.common.description}
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              placeholder={t.vendors.placeholderDescription}
            />
          </label>
          <label>
            {t.common.website}
            <input
              value={website}
              onChange={(e) => setWebsite(e.target.value)}
              placeholder={t.vendors.placeholderWebsite}
              type="url"
            />
          </label>

          <label>
            {t.vendors.tags}
            <input
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder={t.vendors.placeholderTags}
            />
            <span className="field-hint">{t.vendors.tagsHint}</span>
          </label>

          <fieldset className="pricing-fieldset">
            <legend>{t.vendors.pricing}</legend>
            <label className="toggle-row">
              <input
                type="checkbox"
                checked={enablePricing}
                onChange={(e) => setEnablePricing(e.target.checked)}
              />
              <span>{t.vendors.enablePricing}</span>
            </label>
            {enablePricing && (
              <div className="pricing-grid">
                <label>
                  {t.vendors.currency}
                  <input value={currency} onChange={(e) => setCurrency(e.target.value)} />
                </label>
                <label>
                  {t.vendors.billingPeriod}
                  <select
                    value={billingPeriod}
                    onChange={(e) => setBillingPeriod(e.target.value as BillingPeriod)}
                  >
                    <option value="monthly">{t.procurement.monthly}</option>
                    <option value="annual">{t.procurement.annual}</option>
                  </select>
                </label>
                <label>
                  {t.vendors.pricePerDevice}
                  <input
                    type="number"
                    min={0}
                    step="0.01"
                    value={pricePerDevice}
                    onChange={(e) => setPricePerDevice(e.target.value)}
                    placeholder={t.vendors.placeholderPricePerDevice}
                  />
                </label>
                <label>
                  {t.vendors.globalPrice}
                  <input
                    type="number"
                    min={0}
                    step="1"
                    value={globalPrice}
                    onChange={(e) => setGlobalPrice(e.target.value)}
                    placeholder={t.vendors.placeholderGlobalPrice}
                  />
                </label>
                <label className="pricing-notes">
                  {t.vendors.pricingNotes}
                  <input
                    value={pricingNotes}
                    onChange={(e) => setPricingNotes(e.target.value)}
                    placeholder={t.vendors.placeholderPricingNotes}
                  />
                </label>
              </div>
            )}
          </fieldset>

          <div className="form-actions">
            <button type="submit" className="btn btn-primary" disabled={saving}>
              {saving
                ? t.common.saving
                : mode === "edit"
                  ? t.common.saveChanges
                  : t.vendors.add}
            </button>
            <button type="button" className="btn btn-secondary" onClick={resetForm}>
              {t.common.cancel}
            </button>
          </div>
        </form>
      )}

      <div className="data-table-wrap">
        <table className="data-table">
          <thead>
            <tr>
              <th>{t.vendors.vendor}</th>
              <th>{t.common.description}</th>
              <th>{t.common.score}</th>
              <th>{t.vendors.columnPricing}</th>
              <th>{t.common.actions}</th>
            </tr>
          </thead>
          <tbody>
            {vendors.map((vendor) => {
              const result = evaluation.vendors.find((v) => v.vendor.id === vendor.id);
              const score = result?.overall_score.overall_score_percent ?? 0;
              const os = result?.overall_score;
              const pricingLabels = {
                perDevice: t.procurement.perDevice,
                global: t.procurement.globalPrice,
                monthly: t.procurement.monthly,
                annual: t.procurement.annual,
              };
              return (
                <tr key={vendor.id}>
                  <td>
                    <strong>{vendor.name}</strong>
                    <code style={{ display: "block", marginTop: "0.2rem" }}>{vendor.id}</code>
                    {vendor.website && (
                      <a
                        href={vendor.website}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="vendor-link"
                      >
                        {vendor.website.replace(/^https?:\/\//, "")}
                      </a>
                    )}
                    {(vendor.tags ?? []).length > 0 && (
                      <span className="vendor-tag-list">
                        {(vendor.tags ?? []).map((tag) => (
                          <span key={tag} className="vendor-tag">
                            {tag}
                          </span>
                        ))}
                      </span>
                    )}
                  </td>
                  <td className="desc-cell">{vendor.description || t.common.none}</td>
                  <td>
                    <span className="vendor-score-val">{score.toFixed(1)}%</span>
                  </td>
                  <td className="pricing-cell">
                    {vendor.pricing && hasPricingInput(vendor.pricing) ? (
                      <>
                        <span className="pricing-quote">
                          {pricingSummary(vendor.pricing, pricingLabels)}
                        </span>
                        {os?.annual_cost_per_device != null && (
                          <span className="pricing-est">
                            {t.procurement.costPerDeviceAnnual}:{" "}
                            {formatMoney(os.annual_cost_per_device, os.price_currency ?? "USD")}
                          </span>
                        )}
                      </>
                    ) : (
                      <span className="muted">{t.procurement.noPricing}</span>
                    )}
                  </td>
                  <td>
                    <div className="row-actions">
                      <button
                        type="button"
                        className="btn btn-ghost btn-icon"
                        onClick={() => openEdit(vendor)}
                        title={t.common.edit}
                      >
                        ✎
                      </button>
                      <button
                        type="button"
                        className="btn btn-danger btn-icon"
                        onClick={() => {
                          if (confirm(format(t.vendors.confirmRemove, { name: vendor.name }))) {
                            onDelete(vendor.id);
                          }
                        }}
                        title={t.common.remove}
                      >
                        ×
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <style>{`
        .desc-cell { color: var(--mad-text-muted); max-width: 36ch; }
        .vendor-link {
          display: block;
          font-size: 0.78rem;
          color: var(--mad-cyan-dim);
          margin-top: 0.2rem;
        }
        .vendor-score-val {
          font-family: var(--font-mono);
          font-weight: 700;
          color: var(--mad-navy);
        }
        .pricing-fieldset {
          border: 1px solid var(--mad-border);
          border-radius: 8px;
          padding: 0.75rem 1rem;
          margin: 0.5rem 0 0;
        }
        .pricing-fieldset legend {
          font-weight: 600;
          color: var(--mad-navy);
          padding: 0 0.35rem;
        }
        .pricing-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
          gap: 0.75rem;
          margin-top: 0.5rem;
        }
        .pricing-notes { grid-column: 1 / -1; }
        .pricing-cell { font-size: 0.78rem; max-width: 28ch; }
        .pricing-quote { display: block; color: var(--mad-navy); font-weight: 600; }
        .pricing-est { display: block; color: var(--mad-text-muted); margin-top: 0.2rem; }
        .muted { color: var(--mad-text-muted); }
        .field-hint {
          display: block;
          margin-top: 0.25rem;
          font-size: 0.75rem;
          color: var(--mad-text-muted);
          font-weight: 400;
        }
        .vendor-tag-list { display: flex; flex-wrap: wrap; gap: 0.25rem; margin-top: 0.35rem; }
        .vendor-tag {
          font-size: 0.68rem;
          font-weight: 600;
          padding: 0.1rem 0.4rem;
          border-radius: 4px;
          background: rgba(0, 180, 216, 0.12);
          color: var(--mad-cyan-dim);
        }
        .row-actions { display: flex; gap: 0.25rem; }
      `}</style>
    </section>
  );
}
