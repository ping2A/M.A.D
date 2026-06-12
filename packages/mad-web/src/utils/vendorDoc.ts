import type { CSSProperties } from "react";
import type { VendorDocItem, VendorDocSection } from "../types";

export const DOC_COLOR_PRESETS = [
  { id: "", label: "None", hex: "" },
  { id: "critical", label: "Critical", hex: "#dc3545" },
  { id: "warning", label: "Warning", hex: "#e6a800" },
  { id: "info", label: "Info", hex: "#00b4d8" },
  { id: "success", label: "OK", hex: "#28a745" },
  { id: "neutral", label: "Note", hex: "#6c757d" },
] as const;

const PRESET_HEX: Record<string, string> = Object.fromEntries(
  DOC_COLOR_PRESETS.filter((p) => p.id).map((p) => [p.id, p.hex]),
);

export function normalizeDocColor(color?: string | null): string | null {
  const trimmed = color?.trim();
  return trimmed ? trimmed : null;
}

export function isCustomDocColor(color?: string | null): boolean {
  const c = normalizeDocColor(color);
  return !!c && c.startsWith("#");
}

export function resolveDocColorHex(color?: string | null): string | null {
  const c = normalizeDocColor(color);
  if (!c) return null;
  if (c.startsWith("#")) return c;
  return PRESET_HEX[c] ?? null;
}

export function docColorLabel(
  color: string | null | undefined,
  t: Record<string, string>,
): string {
  const c = normalizeDocColor(color);
  if (!c) return t.colorNone ?? "None";
  const preset = DOC_COLOR_PRESETS.find((p) => p.id === c);
  if (preset) return t[preset.id] ?? preset.label;
  return t.customColor ?? "Custom";
}

export function itemCardStyle(color?: string | null): CSSProperties | undefined {
  const hex = resolveDocColorHex(color);
  if (!hex) return undefined;
  return {
    borderLeftColor: hex,
    background: `linear-gradient(90deg, color-mix(in srgb, ${hex} 12%, transparent), transparent 48px)`,
  };
}

let docIdSeq = 0;

function uniqueDocSuffix(): string {
  docIdSeq += 1;
  return `${Date.now()}-${docIdSeq}-${Math.random().toString(36).slice(2, 8)}`;
}

export function newVendorDocId(): string {
  return `vdoc-${uniqueDocSuffix()}`;
}

export function newVendorDocItemId(): string {
  return `vdoc-item-${uniqueDocSuffix()}`;
}

/** Ensures every item has a unique id (fixes legacy templates created in the same millisecond). */
export function dedupeVendorDocItemIds(items: VendorDocItem[]): VendorDocItem[] {
  const seen = new Set<string>();
  let changed = false;
  const next = items.map((item) => {
    if (!seen.has(item.id)) {
      seen.add(item.id);
      return item;
    }
    changed = true;
    const id = newVendorDocItemId();
    seen.add(id);
    return { ...item, id };
  });
  return changed ? next : items;
}

export function hasDuplicateItemIds(items: VendorDocItem[]): boolean {
  const seen = new Set<string>();
  for (const item of items) {
    if (seen.has(item.id)) return true;
    seen.add(item.id);
  }
  return false;
}

export function normalizeVendorDocSection(section: VendorDocSection): VendorDocSection {
  const items = dedupeVendorDocItemIds(section.items);
  if (items === section.items) return section;
  return { ...section, items };
}

export function emptyVendorDocSection(name = ""): VendorDocSection {
  return { id: newVendorDocId(), name, color: null, overview: "", items: [] };
}

export function mdmPrivacyTemplateSection(): VendorDocSection {
  return {
    id: newVendorDocId(),
    name: "Privacy",
    color: "info",
    overview:
      "Document privacy posture for this MDM vendor. This section is informational only and does not affect capability scores.",
    items: [
      {
        id: newVendorDocItemId(),
        group: "Application",
        color: "critical",
        title: "Admin console authentication",
        description:
          "MFA, SSO/SAML, session timeout, and privileged access controls for administrators.",
        notes: "",
      },
      {
        id: newVendorDocItemId(),
        group: "Application",
        color: "warning",
        title: "Application telemetry & logging",
        description:
          "What operational telemetry the vendor collects from the management platform and retention periods.",
        notes: "",
      },
      {
        id: newVendorDocItemId(),
        group: "Application",
        color: "warning",
        title: "Subprocessors & data residency",
        description:
          "List of subprocessors, regions where admin/tenant data is stored, and DPA availability.",
        notes: "",
      },
      {
        id: newVendorDocItemId(),
        group: "End user",
        color: "info",
        title: "Device inventory data",
        description:
          "Which device attributes are collected (hardware IDs, installed apps, OS version) and purpose.",
        notes: "",
      },
      {
        id: newVendorDocItemId(),
        group: "End user",
        color: "critical",
        title: "Location & usage data",
        description:
          "Whether geolocation or usage analytics are collected from end-user devices.",
        notes: "",
      },
      {
        id: newVendorDocItemId(),
        group: "End user",
        color: "success",
        title: "Employee transparency & consent",
        description:
          "Privacy notice, work-profile boundaries, and consent mechanisms for supervised devices.",
        notes: "",
      },
    ],
  };
}

export function newVendorDocItem(): VendorDocItem {
  return {
    id: newVendorDocItemId(),
    group: "",
    color: null,
    title: "",
    description: "",
    notes: "",
  };
}

export function uniqueGroups(items: VendorDocItem[]): string[] {
  const seen = new Set<string>();
  const groups: string[] = [];
  for (const item of items) {
    const group = item.group?.trim() ?? "";
    if (!seen.has(group)) {
      seen.add(group);
      groups.push(group);
    }
  }
  return groups;
}

export function sectionIsEmpty(section: VendorDocSection): boolean {
  const overview = section.overview?.trim() ?? "";
  return section.name.trim() === "" && overview === "" && section.items.length === 0;
}

export function countHighlighted(items: VendorDocItem[]): number {
  return items.filter((item) => normalizeDocColor(item.color)).length;
}
