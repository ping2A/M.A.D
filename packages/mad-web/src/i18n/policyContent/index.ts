import type { Locale } from "../types";
import type {
  BuiltinPillarId,
  Pillar,
  PillarId,
  Requirement,
  RequirementResult,
} from "../../types";
import { BUILTIN_PILLAR_IDS } from "../../types";
import { policyContentFr } from "./fr";
import type { PolicyContentCatalog } from "./types";

const catalogs: Partial<Record<Locale, PolicyContentCatalog>> = {
  fr: policyContentFr,
};

export function getPolicyContentCatalog(locale: Locale): PolicyContentCatalog | null {
  return catalogs[locale] ?? null;
}

export function translateRequirementFields(
  id: string,
  fields: {
    title: string;
    description?: string;
    evaluation_method?: string | null;
    technical_criteria?: string | null;
  },
  _locale: Locale,
): typeof fields {
  void id;
  return fields;
}

/** Criteria text is user-authored — never substitute catalog translations. */
export function localizeRequirement(req: Requirement, _locale: Locale): Requirement {
  return req;
}

export function localizeRequirementResult(req: RequirementResult, _locale: Locale): RequirementResult {
  return req;
}

function builtinPillarTranslation(
  catalog: PolicyContentCatalog | null,
  pillarId: string,
): PolicyContentCatalog["pillars"][BuiltinPillarId] | undefined {
  if (!(BUILTIN_PILLAR_IDS as readonly string[]).includes(pillarId)) {
    return undefined;
  }
  return catalog?.pillars[pillarId as BuiltinPillarId];
}

export function localizePillar(pillar: Pillar, locale: Locale): Pillar {
  const catalog = getPolicyContentCatalog(locale);
  const pillarTr = builtinPillarTranslation(catalog, pillar.id);
  return {
    ...pillar,
    name: pillarTr?.name ?? pillar.name,
    description: pillarTr?.description ?? pillar.description,
    requirements: pillar.requirements,
  };
}

export function localizePillars(pillars: Pillar[], locale: Locale): Pillar[] {
  return pillars.map((p) => localizePillar(p, locale));
}

export function translatePillarName(pillarId: PillarId, fallback: string, locale: Locale): string {
  return builtinPillarTranslation(getPolicyContentCatalog(locale), pillarId)?.name ?? fallback;
}
