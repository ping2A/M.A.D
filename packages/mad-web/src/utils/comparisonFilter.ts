import type { EvaluationReport, EvaluationResult, ProcurementConfig, VendorPricing } from "../types";

function annualize(amount: number, period: "monthly" | "annual"): number {
  return period === "monthly" ? amount * 12 : amount;
}

function computeAnnualCosts(
  pricing: VendorPricing,
  deviceCount: number,
): { perDevice: number | null; total: number | null } {
  const devices = deviceCount;
  const perDevice = pricing.price_per_device != null
    ? annualize(pricing.price_per_device, pricing.billing_period)
    : null;
  const globalAnnual =
    pricing.global_price != null ? annualize(pricing.global_price, pricing.billing_period) : null;

  let total: number | null = null;
  if (perDevice != null && globalAnnual != null && devices > 0) {
    total = globalAnnual + perDevice * devices;
  } else if (perDevice != null && devices > 0) {
    total = perDevice * devices;
  } else if (globalAnnual != null) {
    total = globalAnnual;
  } else if (perDevice != null) {
    total = perDevice * Math.max(devices, 1);
  }

  let perDeviceEffective: number | null = null;
  if (devices > 0) {
    if (perDevice != null && globalAnnual != null) {
      perDeviceEffective = perDevice + globalAnnual / devices;
    } else if (perDevice != null) {
      perDeviceEffective = perDevice;
    } else if (globalAnnual != null) {
      perDeviceEffective = globalAnnual / devices;
    }
  } else {
    perDeviceEffective = perDevice;
  }

  return { perDevice: perDeviceEffective, total };
}

/** Recompute price/composite scores for a vendor subset (e.g. comparison filter). */
export function recomputeSubsetScores(
  vendors: EvaluationResult[],
  procurement: ProcurementConfig,
): EvaluationResult[] {
  const deviceCount = procurement.device_count;
  const cloned = vendors.map((v) => ({
    ...v,
    overall_score: { ...v.overall_score },
  }));

  for (const result of cloned) {
    result.overall_score.price_score_percent = null;
    result.overall_score.composite_score_percent = null;
    result.overall_score.annual_cost_per_device = null;
    result.overall_score.total_annual_cost = null;
    result.overall_score.price_currency = null;

    const pricing = result.vendor.pricing;
    if (!pricing) continue;

    const { perDevice, total } = computeAnnualCosts(pricing, deviceCount);
    result.overall_score.annual_cost_per_device = perDevice;
    result.overall_score.total_annual_cost = total;
    result.overall_score.price_currency = pricing.currency;
  }

  if (!procurement.use_price_in_ranking) {
    return cloned;
  }

  const costs = cloned.map((r) => r.overall_score.annual_cost_per_device);
  const valid = costs.filter((c): c is number => c != null);
  if (valid.length === 0) return cloned;

  const min = Math.min(...valid);
  const max = Math.max(...valid);
  const weight = Math.min(100, Math.max(0, procurement.price_weight_percent)) / 100;

  for (const result of cloned) {
    const cost = result.overall_score.annual_cost_per_device;
    if (cost == null) continue;
    const priceScore = Math.abs(max - min) < 1e-9 ? 100 : (100 * (max - cost)) / (max - min);
    const capability = result.overall_score.overall_score_percent;
    result.overall_score.price_score_percent = priceScore;
    result.overall_score.composite_score_percent = (1 - weight) * capability + weight * priceScore;
  }

  return cloned;
}

export function collectVendorTags(evaluation: EvaluationReport): string[] {
  const tags = new Set<string>();
  for (const v of evaluation.vendors) {
    for (const tag of v.vendor.tags ?? []) {
      const t = tag.trim();
      if (t) tags.add(t);
    }
  }
  return [...tags].sort((a, b) => a.localeCompare(b));
}

export function vendorMatchesTags(vendor: EvaluationResult, activeTags: Set<string>): boolean {
  if (activeTags.size === 0) return true;
  const tags = vendor.vendor.tags ?? [];
  return tags.some((t) => activeTags.has(t));
}

export function buildFilteredEvaluation(
  evaluation: EvaluationReport,
  selectedVendorIds: Set<string>,
  activeTags: Set<string>,
): EvaluationReport {
  let vendors = evaluation.vendors.filter((v) => selectedVendorIds.has(v.vendor.id));
  if (activeTags.size > 0) {
    vendors = vendors.filter((v) => vendorMatchesTags(v, activeTags));
  }
  vendors = recomputeSubsetScores(vendors, evaluation.procurement);
  return { ...evaluation, vendors };
}

export function parseTagsInput(raw: string): string[] {
  const seen = new Set<string>();
  const tags: string[] = [];
  for (const part of raw.split(",")) {
    const t = part.trim().toLowerCase();
    if (t && !seen.has(t)) {
      seen.add(t);
      tags.push(t);
    }
  }
  return tags;
}

export function formatTagsInput(tags?: string[] | null): string {
  return (tags ?? []).join(", ");
}
