import { useCallback } from "react";
import { useLocale } from "../i18n/LocaleContext";
import {
  localizePillar,
  localizePillars,
  localizeRequirement,
  localizeRequirementResult,
  translatePillarName,
  translateRequirementFields,
} from "../i18n/policyContent";
import type { EvaluationReport, Pillar, Requirement, RequirementResult } from "../types";

export function usePolicyContent() {
  const { locale } = useLocale();

  const localizePillarMemo = useCallback(
    (pillar: Pillar) => localizePillar(pillar, locale),
    [locale],
  );

  const localizePillarsMemo = useCallback(
    (pillars: Pillar[]) => localizePillars(pillars, locale),
    [locale],
  );

  const localizeRequirementMemo = useCallback(
    (req: Requirement) => localizeRequirement(req, locale),
    [locale],
  );

  const localizeRequirementResultMemo = useCallback(
    (req: RequirementResult) => localizeRequirementResult(req, locale),
    [locale],
  );

  const translatePillarNameMemo = useCallback(
    (pillarId: Parameters<typeof translatePillarName>[0], fallback: string) =>
      translatePillarName(pillarId, fallback, locale),
    [locale],
  );

  const translateRequirementFieldsMemo = useCallback(
    (
      id: string,
      fields: Parameters<typeof translateRequirementFields>[1],
    ) => translateRequirementFields(id, fields, locale),
    [locale],
  );

  const localizeEvaluation = useCallback(
    (evaluation: EvaluationReport): EvaluationReport => ({
      ...evaluation,
      vendors: evaluation.vendors.map((v) => ({
        ...v,
        pillars: v.pillars.map((p) => ({
          ...p,
          pillar_name: translatePillarName(p.pillar_id, p.pillar_name, locale),
          requirements: p.requirements.map((r) => localizeRequirementResult(r, locale)),
        })),
      })),
    }),
    [locale],
  );

  return {
    locale,
    localizePillar: localizePillarMemo,
    localizePillars: localizePillarsMemo,
    localizeRequirement: localizeRequirementMemo,
    localizeRequirementResult: localizeRequirementResultMemo,
    translatePillarName: translatePillarNameMemo,
    translateRequirementFields: translateRequirementFieldsMemo,
    localizeEvaluation,
  };
}
