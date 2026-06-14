import type {
  ComplianceStatus,
  EvaluationReport,
  EvaluationResult,
  Pillar,
  PillarScore,
  ProcurementConfig,
  RequirementSeverity,
  ScoringConfig,
  Vendor,
  VendorPricing,
} from "../types";

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

export function normalizeTag(tag: string): string {
  return tag.trim().toLowerCase();
}

/** When a tag filter is active, only tagged requirements sharing a filter tag are in scope. */
export function requirementMatchesTags(
  requirementTags: string[] | undefined,
  activeTags: Set<string>,
): boolean {
  if (activeTags.size === 0) return true;
  const req = (requirementTags ?? []).map(normalizeTag).filter(Boolean);
  if (req.length === 0) return false;
  const active = new Set([...activeTags].map(normalizeTag));
  return req.some((t) => active.has(t));
}

export function isRequirementInScope(
  requirementTags: string[] | undefined,
  vendorTags: string[] | undefined,
  activeTags: Set<string>,
): boolean {
  if (activeTags.size === 0) {
    return requirementAppliesToVendor(requirementTags, vendorTags);
  }
  return (
    requirementMatchesTags(requirementTags, activeTags)
    && requirementAppliesToVendor(requirementTags, vendorTags)
  );
}

/** Vendors that apply to at least one criterion carrying the selected criteria tags. */
export function vendorMatchesCriteriaTagFilter(
  vendor: EvaluationResult,
  activeTags: Set<string>,
  reqTags: Map<string, string[]>,
): boolean {
  if (activeTags.size === 0) return true;
  for (const tags of reqTags.values()) {
    if (!requirementMatchesTags(tags, activeTags)) continue;
    if (requirementAppliesToVendor(tags, vendor.vendor.tags)) return true;
  }
  return false;
}

export function requirementTagsFromPillars(pillars: Pillar[]): Map<string, string[]> {
  const map = new Map<string, string[]>();
  for (const pillar of pillars) {
    for (const req of pillar.requirements) {
      map.set(req.id, req.tags ?? []);
    }
  }
  return map;
}

function requirementScore(
  scoring: ScoringConfig,
  status: ComplianceStatus,
  severity: RequirementSeverity,
): { earned: number; max: number } {
  const statusPoints: Record<ComplianceStatus, number> = {
    compliant: scoring.compliant_points,
    partial: scoring.partial_points,
    non_compliant: scoring.non_compliant_points,
    untested: scoring.untested_points,
  };
  const severityWeight: Record<RequirementSeverity, number> = {
    critical: scoring.critical_weight,
    high: scoring.high_weight,
    medium: scoring.medium_weight,
  };
  const earned = statusPoints[status];
  const max = scoring.compliant_points;
  if (scoring.use_severity_weighting) {
    const w = severityWeight[severity];
    return { earned: earned * w, max: max * w };
  }
  return { earned, max };
}

function scorePillarFromRequirements(
  pillarId: string,
  requirements: EvaluationResult["pillars"][0]["requirements"],
  scoring: ScoringConfig,
): PillarScore {
  let compliant = 0;
  let partial = 0;
  let nonCompliant = 0;
  let untested = 0;
  let earned = 0;
  let maxPossible = 0;

  for (const req of requirements) {
    switch (req.status) {
      case "compliant":
        compliant += 1;
        break;
      case "partial":
        partial += 1;
        break;
      case "non_compliant":
        nonCompliant += 1;
        break;
      case "untested":
        untested += 1;
        break;
    }
    const { earned: e, max: m } = requirementScore(scoring, req.status, req.severity);
    earned += e;
    maxPossible += m;
  }

  const total = requirements.length;
  const scorePercent = maxPossible === 0 ? 0 : (earned / maxPossible) * 100;

  return {
    pillar_id: pillarId,
    compliant,
    partial,
    non_compliant: nonCompliant,
    untested,
    total,
    score_percent: scorePercent,
  };
}

function computeOverallScore(vendor: Vendor, pillars: EvaluationResult["pillars"]): EvaluationResult["overall_score"] {
  const pillarScores = pillars.map((p) => p.score);
  const scoringPillars = pillarScores.filter((s) => s.total > 0);
  const overallScorePercent =
    scoringPillars.length === 0
      ? 0
      : scoringPillars.reduce((sum, s) => sum + s.score_percent, 0) / scoringPillars.length;

  const criticalGaps = pillars
    .flatMap((p) => p.requirements)
    .filter(
      (r) =>
        r.applicable !== false
        && r.severity === "critical"
        && (r.status === "non_compliant" || r.status === "untested"),
    )
    .map((r) => `${r.requirement_id}: ${r.title}`);

  return {
    vendor,
    pillar_scores: pillarScores,
    overall_score_percent: overallScorePercent,
    critical_gaps: criticalGaps,
  };
}

function recomputeVendorForTagFilter(
  result: EvaluationResult,
  criteriaTags: Set<string>,
  reqTags: Map<string, string[]>,
  scoring: ScoringConfig,
): EvaluationResult {
  const vendorTags = result.vendor.tags;
  const pillars = result.pillars.map((pillar) => {
    const requirements = pillar.requirements.map((req) => {
      const tags = reqTags.get(req.requirement_id) ?? [];
      const vendorOk = requirementAppliesToVendor(tags, vendorTags);
      const tagOk =
        criteriaTags.size === 0 || requirementMatchesTags(tags, criteriaTags);
      const applicable = vendorOk && tagOk;
      return { ...req, applicable };
    });
    const scored = requirements.filter((r) => r.applicable);
    const score = scorePillarFromRequirements(pillar.pillar_id, scored, scoring);
    return { ...pillar, requirements, score };
  });
  const overall_score = computeOverallScore(result.vendor, pillars);
  return { ...result, pillars, overall_score };
}

/** A requirement applies when untagged (universal) or it shares at least one tag with the vendor. */
export function requirementAppliesToVendor(
  requirementTags: string[] | undefined,
  vendorTags: string[] | undefined,
): boolean {
  const req = (requirementTags ?? []).map(normalizeTag).filter(Boolean);
  if (req.length === 0) return true;
  const vendor = new Set((vendorTags ?? []).map(normalizeTag).filter(Boolean));
  if (vendor.size === 0) return false;
  return req.some((t) => vendor.has(t));
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

/** Tags defined on criteria — used for the global tag filter chips. */
export function collectCriteriaTags(pillars: Pillar[]): string[] {
  const tags = new Set<string>();
  for (const pillar of pillars) {
    for (const req of pillar.requirements) {
      for (const tag of req.tags ?? []) {
        const t = tag.trim();
        if (t) tags.add(t);
      }
    }
  }
  return [...tags].sort((a, b) => a.localeCompare(b));
}

/** Tags on criteria and vendors — only tags you set in the Criteria and Vendors tabs. */
export function collectFilterTags(evaluation: EvaluationReport, pillars: Pillar[]): string[] {
  const tags = new Set<string>();
  for (const tag of collectVendorTags(evaluation)) {
    tags.add(tag);
  }
  for (const tag of collectCriteriaTags(pillars)) {
    tags.add(tag);
  }
  return [...tags].sort((a, b) => a.localeCompare(b));
}

function tagOnAnyCriterion(tag: string, reqTags: Map<string, string[]>): boolean {
  const normalized = normalizeTag(tag);
  for (const tags of reqTags.values()) {
    if (tags.some((t) => normalizeTag(t) === normalized)) return true;
  }
  return false;
}

/** Split selected chips into tags that exist on criteria vs vendor-only labels. */
export function splitActiveTags(
  activeTags: Set<string>,
  reqTags: Map<string, string[]>,
): { criteriaTags: Set<string>; vendorTags: Set<string> } {
  const criteriaTags = new Set<string>();
  const vendorTags = new Set<string>();
  for (const tag of activeTags) {
    if (tagOnAnyCriterion(tag, reqTags)) {
      criteriaTags.add(tag);
    } else {
      vendorTags.add(tag);
    }
  }
  return { criteriaTags, vendorTags };
}

export function vendorInTagFilterScope(
  vendor: EvaluationResult,
  activeTags: Set<string>,
  reqTags: Map<string, string[]>,
): boolean {
  if (activeTags.size === 0) return true;
  const { criteriaTags, vendorTags } = splitActiveTags(activeTags, reqTags);
  if (vendorTags.size > 0 && vendorMatchesTags(vendor, vendorTags)) return true;
  if (criteriaTags.size > 0 && vendorMatchesCriteriaTagFilter(vendor, criteriaTags, reqTags)) {
    return true;
  }
  return false;
}

export function criterionInTagFilterScope(
  requirementTags: string[] | undefined,
  activeTags: Set<string>,
  reqTags: Map<string, string[]>,
): boolean {
  if (activeTags.size === 0) return true;
  const { criteriaTags, vendorTags } = splitActiveTags(activeTags, reqTags);
  if (criteriaTags.size > 0) {
    return requirementMatchesTags(requirementTags, criteriaTags);
  }
  // Vendor-only filter: criteria are narrowed by vendor applicability in the matrix.
  return vendorTags.size > 0;
}

export function vendorMatchesTags(vendor: EvaluationResult, activeTags: Set<string>): boolean {
  if (activeTags.size === 0) return true;
  const active = new Set([...activeTags].map(normalizeTag));
  const tags = vendor.vendor.tags ?? [];
  return tags.some((t) => active.has(normalizeTag(t)));
}

export function buildFilteredEvaluation(
  evaluation: EvaluationReport,
  selectedVendorIds: Set<string>,
  activeTags: Set<string>,
  pillars?: Pillar[],
): EvaluationReport {
  let vendors = evaluation.vendors.filter((v) => selectedVendorIds.has(v.vendor.id));
  const reqTags = pillars ? requirementTagsFromPillars(pillars) : new Map<string, string[]>();
  const { criteriaTags } = splitActiveTags(activeTags, reqTags);
  if (activeTags.size > 0) {
    vendors = vendors.filter((v) => vendorInTagFilterScope(v, activeTags, reqTags));
  }
  vendors = vendors.map((v) =>
    recomputeVendorForTagFilter(v, criteriaTags, reqTags, evaluation.scoring),
  );
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

export function reportTagsQuery(tags: Iterable<string>): string {
  const list = [...tags];
  if (list.length === 0) return "";
  return `tags=${encodeURIComponent(list.join(","))}`;
}
