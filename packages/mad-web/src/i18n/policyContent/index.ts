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
import type { PolicyContentCatalog, RequirementTranslation } from "./types";

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
  locale: Locale,
): typeof fields {
  const catalog = getPolicyContentCatalog(locale);
  const tr: RequirementTranslation | undefined = catalog?.requirements[id];
  if (!tr) return fields;
  return {
    title: tr.title,
    description: tr.description ?? fields.description,
    evaluation_method: tr.evaluation_method ?? fields.evaluation_method ?? undefined,
    technical_criteria: tr.technical_criteria ?? fields.technical_criteria ?? undefined,
  };
}

export function localizeRequirement(req: Requirement, locale: Locale): Requirement {
  const tr = translateRequirementFields(
    req.id,
    {
      title: req.title,
      description: req.description,
      evaluation_method: req.evaluation_method,
      technical_criteria: req.technical_criteria,
    },
    locale,
  );
  return {
    ...req,
    title: tr.title,
    description: tr.description ?? req.description,
    evaluation_method: tr.evaluation_method ?? req.evaluation_method,
    technical_criteria: tr.technical_criteria ?? req.technical_criteria,
  };
}

export function localizeRequirementResult(
  req: RequirementResult,
  locale: Locale,
): RequirementResult {
  const tr = translateRequirementFields(
    req.requirement_id,
    { title: req.title },
    locale,
  );
  return { ...req, title: tr.title };
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
    requirements: pillar.requirements.map((r) => localizeRequirement(r, locale)),
  };
}

export function localizePillars(pillars: Pillar[], locale: Locale): Pillar[] {
  return pillars.map((p) => localizePillar(p, locale));
}

export function translatePillarName(pillarId: PillarId, fallback: string, locale: Locale): string {
  return builtinPillarTranslation(getPolicyContentCatalog(locale), pillarId)?.name ?? fallback;
}
