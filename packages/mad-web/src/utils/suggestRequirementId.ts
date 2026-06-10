import type { Pillar } from "../types";

const PREFIX_BY_PILLAR: Record<string, string> = {
  cybersecurity_dlp: "dlp",
  dfir: "dfir",
  platform_os: "plat",
};

export function requirementIdPrefix(pillarId: string): string {
  return PREFIX_BY_PILLAR[pillarId] ?? pillarId.replace(/_/g, "-").slice(0, 16);
}

export function suggestNextRequirementId(pillarId: string, pillars: Pillar[]): string {
  const prefix = requirementIdPrefix(pillarId);
  const escaped = prefix.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const pattern = new RegExp(`^${escaped}-(\\d+)$`);
  let max = 0;
  for (const pillar of pillars) {
    for (const req of pillar.requirements) {
      const match = req.id.match(pattern);
      if (match) {
        max = Math.max(max, parseInt(match[1], 10));
      }
    }
  }
  return `${prefix}-${String(max + 1).padStart(3, "0")}`;
}

export function slugifyPillarId(name: string): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return slug || "custom_group";
}
