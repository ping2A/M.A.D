import type { BuiltinPillarId } from "../../types";

export interface RequirementTranslation {
  title: string;
  description: string;
  evaluation_method?: string;
  technical_criteria?: string;
}

export interface PillarTranslation {
  name: string;
  description: string;
}

export interface PolicyContentCatalog {
  pillars: Record<BuiltinPillarId, PillarTranslation>;
  requirements: Record<string, RequirementTranslation>;
}
