use std::borrow::Cow;

use crate::pillar::{Pillar, Requirement, RequirementSeverity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportLocale {
    #[default]
    En,
    Fr,
}

impl ReportLocale {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "fr" | "fr-fr" | "fr_fr" => Self::Fr,
            _ => Self::En,
        }
    }

    pub fn html_lang(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
        }
    }
}

pub struct ReportStrings {
    pub html_lang: &'static str,
    pub page_title: &'static str,
    pub subtitle: &'static str,
    pub meta_policy: &'static str,
    pub meta_requirements: &'static str,
    pub meta_vendors: &'static str,
    pub meta_generated: &'static str,
    pub meta_scope: &'static str,
    pub meta_tags_filter: &'static str,
    pub section_methodology: &'static str,
    pub col_status: &'static str,
    pub status_na: &'static str,
    pub col_weight: &'static str,
    pub col_meaning: &'static str,
    pub meaning_compliant: &'static str,
    pub meaning_partial: &'static str,
    pub meaning_non_compliant: &'static str,
    pub meaning_untested: &'static str,
    pub section_requirements: &'static str,
    pub requirements_count: &'static str,
    pub platforms: &'static str,
    pub evaluation_method: &'static str,
    pub technical_criteria: &'static str,
    pub section_results: &'static str,
    pub col_id: &'static str,
    pub col_requirement: &'static str,
    pub col_notes: &'static str,
    pub critical_gaps: &'static str,
    pub collapse: &'static str,
    pub expand: &'static str,
    pub section_vsm: &'static str,
    pub vsm_intro: &'static str,
    pub section_docs: &'static str,
    pub docs_intro: &'static str,
    pub footer: &'static str,
    pub nav_method: &'static str,
    pub nav_requirements: &'static str,
    pub nav_results: &'static str,
    pub nav_vsm: &'static str,
    pub nav_docs: &'static str,
    pub toc_heading: &'static str,
    pub toc_methodology: &'static str,
    pub toc_requirements: &'static str,
    pub toc_results: &'static str,
    pub toc_vsm: &'static str,
    pub toc_docs: &'static str,
    pub filter_all_vendors: &'static str,
    pub filter_vendor_aria: &'static str,
    pub critical_gaps_count: &'static str,
    pub vsm_nodes: &'static str,
    pub vsm_flows: &'static str,
    pub vsm_total_lead: &'static str,
    pub vsm_timed_pct: &'static str,
    pub vsm_tab_diagram: &'static str,
    pub vsm_tab_timeline: &'static str,
    pub vsm_tab_steps: &'static str,
    pub vsm_zoom_out: &'static str,
    pub vsm_zoom_reset: &'static str,
    pub vsm_zoom_in: &'static str,
    pub vsm_flow_types: &'static str,
    pub vsm_process_timeline: &'static str,
    pub vsm_flow_details: &'static str,
    pub vsm_col_from: &'static str,
    pub vsm_col_to: &'static str,
    pub vsm_col_type: &'static str,
    pub vsm_col_duration: &'static str,
    pub vsm_col_author: &'static str,
    pub vsm_timeline_total: &'static str,
    pub vsm_timeline_timed: &'static str,
    pub vsm_timeline_coverage: &'static str,
    pub vsm_timeline_aria: &'static str,
    pub vsm_milestones: &'static str,
    pub vsm_process_steps: &'static str,
    pub vsm_col_step: &'static str,
    pub vsm_col_lead: &'static str,
    pub vsm_col_cycle: &'static str,
    pub vsm_messages: &'static str,
    pub js_close: &'static str,
    pub js_type: &'static str,
    pub js_author: &'static str,
    pub js_role: &'static str,
    pub js_lead_time: &'static str,
    pub js_cycle_time: &'static str,
    pub js_flow_type: &'static str,
    pub js_duration: &'static str,
    pub js_label: &'static str,
    pub duration_short_minute: &'static str,
    pub duration_short_hour: &'static str,
    pub duration_short_day: &'static str,
    pub duration_short_week: &'static str,
    pub duration_short_sep: &'static str,
    pub doc_filter_all: &'static str,
    pub flow_material: &'static str,
    pub flow_information: &'static str,
    pub flow_electronic: &'static str,
}

const STRINGS_EN: ReportStrings = ReportStrings {
    html_lang: "en",
    page_title: "MAD — Mobile Assessment & Defense",
    subtitle: "Mobile Assessment & Defense",
    meta_policy: "Policy",
    meta_requirements: "Requirements",
    meta_vendors: "Vendors",
    meta_generated: "Generated",
    meta_scope: "iOS & Android MDM only",
    meta_tags_filter: "Tags filter",
    section_methodology: "1. Evaluation Methodology",
    col_status: "Status",
    status_na: "N/A",
    col_weight: "Weight",
    col_meaning: "Meaning",
    meaning_compliant: "Native capability, no workarounds",
    meaning_partial: "Limited, platform-specific, or manual",
    meaning_non_compliant: "Cannot be met",
    meaning_untested: "No assessment data",
    section_requirements: "2. Requirements and Technical Criteria",
    requirements_count: "requirements",
    platforms: "Platforms",
    evaluation_method: "Evaluation method",
    technical_criteria: "Technical criteria",
    section_results: "3. Vendor Assessment Results",
    col_id: "ID",
    col_requirement: "Requirement",
    col_notes: "Notes",
    critical_gaps: "Critical gaps",
    collapse: "Collapse",
    expand: "Expand",
    section_vsm: "Value Stream Maps",
    vsm_intro: "Process flows documented per vendor during evaluation — nodes, flow types, durations, and responsible authors.",
    section_docs: "Vendor Documentation",
    docs_intro: "User-defined per-vendor documentation (e.g. privacy, support). Informational only — not included in capability scores.",
    footer: "Generated by MAD — Mobile Assessment & Defense. Sample assessments for demonstration; production evaluations require lab validation.",
    nav_method: "Method",
    nav_requirements: "Requirements",
    nav_results: "Results",
    nav_vsm: "VSM",
    nav_docs: "Docs",
    toc_heading: "Contents",
    toc_methodology: "Methodology",
    toc_requirements: "Requirements",
    toc_results: "Vendor results",
    toc_vsm: "Value streams",
    toc_docs: "Documentation",
    filter_all_vendors: "All vendors",
    filter_vendor_aria: "Filter by vendor",
    critical_gaps_count: "critical gap(s)",
    vsm_nodes: "nodes",
    vsm_flows: "flows",
    vsm_total_lead: "total lead time",
    vsm_timed_pct: "timed",
    vsm_tab_diagram: "Diagram",
    vsm_tab_timeline: "Timeline",
    vsm_tab_steps: "Steps",
    vsm_zoom_out: "Zoom out",
    vsm_zoom_reset: "Reset view",
    vsm_zoom_in: "Zoom in",
    vsm_flow_types: "Flow types",
    vsm_process_timeline: "Process timeline",
    vsm_flow_details: "Flow details",
    vsm_col_from: "From",
    vsm_col_to: "To",
    vsm_col_type: "Type",
    vsm_col_duration: "Duration",
    vsm_col_author: "Author",
    vsm_timeline_total: "Total lead time",
    vsm_timeline_timed: "Timed flows",
    vsm_timeline_coverage: "Coverage",
    vsm_timeline_aria: "Process timeline",
    vsm_milestones: "Milestones",
    vsm_process_steps: "Process steps",
    vsm_col_step: "Step",
    vsm_col_lead: "Lead",
    vsm_col_cycle: "Cycle",
    vsm_messages: "Messages & notes",
    js_close: "Close",
    js_type: "Type",
    js_author: "Author",
    js_role: "Role",
    js_lead_time: "Lead time",
    js_cycle_time: "Cycle time",
    js_flow_type: "Flow type",
    js_duration: "Duration",
    js_label: "Label",
    duration_short_minute: "m",
    duration_short_hour: "h",
    duration_short_day: "d",
    duration_short_week: "w",
    duration_short_sep: "",
    doc_filter_all: "All",
    flow_material: "Material flow",
    flow_information: "Information flow",
    flow_electronic: "Electronic flow",
};

const STRINGS_FR: ReportStrings = ReportStrings {
    html_lang: "fr",
    page_title: "MAD — Évaluation et défense mobile",
    subtitle: "Évaluation et défense mobile",
    meta_policy: "Politique",
    meta_requirements: "Exigences",
    meta_vendors: "Éditeurs",
    meta_generated: "Généré",
    meta_scope: "MDM iOS et Android uniquement",
    meta_tags_filter: "Filtre étiquettes",
    section_methodology: "1. Méthodologie d'évaluation",
    col_status: "Statut",
    status_na: "N/A",
    col_weight: "Poids",
    col_meaning: "Signification",
    meaning_compliant: "Capacité native, sans contournement",
    meaning_partial: "Limitée, spécifique à la plateforme ou manuelle",
    meaning_non_compliant: "Non réalisable",
    meaning_untested: "Aucune donnée d'évaluation",
    section_requirements: "2. Exigences et critères techniques",
    requirements_count: "exigences",
    platforms: "Plateformes",
    evaluation_method: "Méthode d'évaluation",
    technical_criteria: "Critères techniques",
    section_results: "3. Résultats d'évaluation des éditeurs",
    col_id: "ID",
    col_requirement: "Exigence",
    col_notes: "Notes",
    critical_gaps: "Écarts critiques",
    collapse: "Réduire",
    expand: "Développer",
    section_vsm: "Cartes de flux de valeur",
    vsm_intro: "Flux de processus documentés par éditeur lors de l'évaluation — étapes, types de flux, durées et auteurs responsables.",
    section_docs: "Documentation éditeur",
    docs_intro: "Documentation définie par l'utilisateur par éditeur (ex. confidentialité, support). Informative uniquement — non incluse dans les scores.",
    footer: "Généré par MAD — Évaluation et défense mobile. Évaluations d'exemple à titre de démonstration ; les évaluations de production nécessitent une validation en laboratoire.",
    nav_method: "Méthode",
    nav_requirements: "Exigences",
    nav_results: "Résultats",
    nav_vsm: "VSM",
    nav_docs: "Docs",
    toc_heading: "Sommaire",
    toc_methodology: "Méthodologie",
    toc_requirements: "Exigences",
    toc_results: "Résultats éditeurs",
    toc_vsm: "Flux de valeur",
    toc_docs: "Documentation",
    filter_all_vendors: "Tous les éditeurs",
    filter_vendor_aria: "Filtrer par éditeur",
    critical_gaps_count: "écart(s) critique(s)",
    vsm_nodes: "nœuds",
    vsm_flows: "flux",
    vsm_total_lead: "délai total",
    vsm_timed_pct: "horodatés",
    vsm_tab_diagram: "Schéma",
    vsm_tab_timeline: "Chronologie",
    vsm_tab_steps: "Étapes",
    vsm_zoom_out: "Zoom arrière",
    vsm_zoom_reset: "Réinitialiser",
    vsm_zoom_in: "Zoom avant",
    vsm_flow_types: "Types de flux",
    vsm_process_timeline: "Chronologie du processus",
    vsm_flow_details: "Détail des flux",
    vsm_col_from: "De",
    vsm_col_to: "Vers",
    vsm_col_type: "Type",
    vsm_col_duration: "Durée",
    vsm_col_author: "Auteur",
    vsm_timeline_total: "Délai total",
    vsm_timeline_timed: "Flux horodatés",
    vsm_timeline_coverage: "Couverture",
    vsm_timeline_aria: "Chronologie du processus",
    vsm_milestones: "Jalons",
    vsm_process_steps: "Étapes du processus",
    vsm_col_step: "Étape",
    vsm_col_lead: "Délai",
    vsm_col_cycle: "Cycle",
    vsm_messages: "Messages et notes",
    js_close: "Fermer",
    js_type: "Type",
    js_author: "Auteur",
    js_role: "Rôle",
    js_lead_time: "Délai",
    js_cycle_time: "Temps de cycle",
    js_flow_type: "Type de flux",
    js_duration: "Durée",
    js_label: "Libellé",
    duration_short_minute: "min",
    duration_short_hour: "h",
    duration_short_day: "j",
    duration_short_week: "sem",
    duration_short_sep: " ",
    doc_filter_all: "Tout",
    flow_material: "Flux matériel",
    flow_information: "Flux d'information",
    flow_electronic: "Flux électronique",
};

pub fn strings(locale: ReportLocale) -> &'static ReportStrings {
    match locale {
        ReportLocale::En => &STRINGS_EN,
        ReportLocale::Fr => &STRINGS_FR,
    }
}

pub fn severity_label(severity: RequirementSeverity, locale: ReportLocale) -> &'static str {
    match (locale, severity) {
        (ReportLocale::Fr, RequirementSeverity::Critical) => "CRITIQUE",
        (ReportLocale::Fr, RequirementSeverity::High) => "ÉLEVÉ",
        (ReportLocale::Fr, RequirementSeverity::Medium) => "MOYEN",
        (_, RequirementSeverity::Critical) => "CRITICAL",
        (_, RequirementSeverity::High) => "HIGH",
        (_, RequirementSeverity::Medium) => "MEDIUM",
    }
}

pub fn flow_type_label(id: &str, fallback: &str, locale: ReportLocale) -> Cow<'static, str> {
    let s = strings(locale);
    let label = match id {
        "material" => s.flow_material,
        "information" => s.flow_information,
        "electronic" => s.flow_electronic,
        _ => return Cow::Owned(fallback.to_string()),
    };
    Cow::Borrowed(label)
}

pub fn pillar_name<'a>(id: &str, english: &'a str, locale: ReportLocale) -> Cow<'a, str> {
    if matches!(locale, ReportLocale::Fr) {
        if let Some(name) = fr_pillar_name(id) {
            return Cow::Borrowed(name);
        }
    }
    Cow::Borrowed(english)
}

pub fn pillar_description<'a>(id: &str, english: &'a str, locale: ReportLocale) -> Cow<'a, str> {
    if matches!(locale, ReportLocale::Fr) {
        if let Some(desc) = fr_pillar_description(id) {
            return Cow::Borrowed(desc);
        }
    }
    Cow::Borrowed(english)
}

pub struct LocalizedRequirement<'a> {
    pub title: Cow<'a, str>,
    pub description: Cow<'a, str>,
    pub evaluation_method: Option<Cow<'a, str>>,
    pub technical_criteria: Option<Cow<'a, str>>,
}

pub fn localize_requirement<'a>(req: &'a Requirement, _locale: ReportLocale) -> LocalizedRequirement<'a> {
    LocalizedRequirement {
        title: Cow::Borrowed(&req.title),
        description: Cow::Borrowed(req.description.trim()),
        evaluation_method: req.evaluation_method.as_deref().map(Cow::Borrowed),
        technical_criteria: req.technical_criteria.as_deref().map(Cow::Borrowed),
    }
}

pub fn requirement_title<'a>(id: &str, english: &'a str, _locale: ReportLocale) -> Cow<'a, str> {
    let _ = id;
    Cow::Borrowed(english)
}

fn fr_pillar_name(id: &str) -> Option<&'static str> {
    match id {
        "cybersecurity_dlp" => Some("Cybersécurité et prévention des pertes de données"),
        "dfir" => Some("Forensique numérique et réponse aux incidents"),
        "platform_os" => Some("Plateforme et prise en charge native des OS"),
        _ => None,
    }
}

fn fr_pillar_description(id: &str) -> Option<&'static str> {
    match id {
        "cybersecurity_dlp" => Some(
            "Application de la conteneurisation reposant sur le matériel, de l'accès conditionnel Zero Trust via les fournisseurs d'identité, et de la détection et remédiation automatisées des compromissions OS (jailbreak/root).",
        ),
        "dfir" => Some(
            "Isolation réseau à distance préservant la mémoire volatile, triage non destructif sans alerte utilisateur, et streaming API direct des journaux d'audit vers le SIEM d'entreprise.",
        ),
        "platform_os" => Some(
            "Prise en charge complète d'Apple Business Manager avec mode supervisé iOS, et d'Android Enterprise (profil professionnel, COBO, kiosque dédié) avec OEMConfig.",
        ),
        _ => None,
    }
}

pub fn localized_pillar_fields(pillar: &Pillar, locale: ReportLocale) -> (String, String) {
    (
        pillar_name(&pillar.id, &pillar.name, locale).into_owned(),
        pillar_description(&pillar.id, pillar.description.trim(), locale).into_owned(),
    )
}
