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

pub fn localize_requirement<'a>(req: &'a Requirement, locale: ReportLocale) -> LocalizedRequirement<'a> {
    if matches!(locale, ReportLocale::Fr) {
        if let Some(fr) = fr_requirement(&req.id) {
            return LocalizedRequirement {
                title: Cow::Borrowed(fr.title),
                description: Cow::Borrowed(fr.description),
                evaluation_method: Some(Cow::Borrowed(fr.evaluation_method)),
                technical_criteria: Some(Cow::Borrowed(fr.technical_criteria)),
            };
        }
    }
    LocalizedRequirement {
        title: Cow::Borrowed(&req.title),
        description: Cow::Borrowed(req.description.trim()),
        evaluation_method: req.evaluation_method.as_deref().map(Cow::Borrowed),
        technical_criteria: req.technical_criteria.as_deref().map(Cow::Borrowed),
    }
}

pub fn requirement_title<'a>(id: &str, english: &'a str, locale: ReportLocale) -> Cow<'a, str> {
    if matches!(locale, ReportLocale::Fr) {
        if let Some(fr) = fr_requirement(id) {
            return Cow::Borrowed(fr.title);
        }
    }
    Cow::Borrowed(english)
}

struct FrRequirement {
    title: &'static str,
    description: &'static str,
    evaluation_method: &'static str,
    technical_criteria: &'static str,
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

fn fr_requirement(id: &str) -> Option<&'static FrRequirement> {
    Some(match id {
        "dlp-001" => &FrRequirement {
            title: "Conteneurisation professionnel/personnel reposant sur le matériel",
            description: "La solution MDM doit imposer une séparation professionnel/personnel reposant sur le matériel via des conteneurs natifs (applications gérées iOS / profil professionnel Android).",
            evaluation_method: "Déployer des appareils de test avec profil professionnel (Android) et inscription utilisateur / applications gérées (iOS). Tenter une exfiltration inter-conteneurs (presse-papiers, partage de fichiers, capture d'écran). Vérifier que les clés de chiffrement sont liées au matériel (TEE/Secure Enclave).",
            technical_criteria: "Android : Android Enterprise Work Profile, Device Policy Controller (DPC), android.app.admin.DevicePolicyManager. iOS : applications gérées, VPN par application, NSFileProtectionComplete, restrictions MDM ManagedOpenIn.",
        },
        "dlp-002" => &FrRequirement {
            title: "Intégration IdP pour l'accès conditionnel Zero Trust",
            description: "Intégration aux fournisseurs d'identité d'entreprise (Entra ID, Okta, etc.) pour des politiques d'accès conditionnel basées sur la conformité des appareils.",
            evaluation_method: "Configurer une règle d'accès conditionnel basée sur la conformité IdP. Inscrire des appareils conformes et non conformes. Vérifier l'octroi/refus d'accès aux ressources d'entreprise (M365, applications internes) selon l'état de conformité remonté par la MDM.",
            technical_criteria: "API Microsoft Graph deviceManagement, Okta Device Access, revendications SAML/OIDC appareil, charges de politique de conformité (détection jailbreak, version OS, état du chiffrement).",
        },
        "dlp-003" => &FrRequirement {
            title: "Détection de compromission OS et remédiation automatisée",
            description: "Détection immédiate du jailbreak/root avec actions de remédiation automatisées (quarantaine, effacement des données professionnelles, blocage d'accès).",
            evaluation_method: "Soumettre un appareil rooté/jailbreaké à la politique de conformité. Mesurer le délai de détection et vérifier le déclenchement automatique (effacement sélectif, effacement complet ou blocage d'accès) sans intervention manuelle.",
            technical_criteria: "iOS : détection jailbreak via conformité MDM (supervisé), SafetyNet/DCIM sur Android, signaux API RootBeer, remédiation automatisée via WipeDevice ou action retire via Graph API / REST éditeur.",
        },
        "dfir-001" => &FrRequirement {
            title: "Isolation réseau à distance avec préservation de la mémoire volatile",
            description: "Capacité d'isoler le réseau à distance tout en préservant la mémoire volatile pour l'analyse forensique lors d'une réponse à incident.",
            evaluation_method: "Déclencher l'isolation réseau sur un appareil de test actif exécutant des applications au premier plan. Vérifier le blocage TCP/UDP tout en maintenant l'appareil sous tension sans effacement de la RAM (pas de redémarrage, pas d'effacement). Confirmer la réversibilité pour l'imagerie forensique.",
            technical_criteria: "VPN par application en mode blocage, règles pare-feu via charge réseau MDM (iOS), VPN always-on Android en mode blocage, ou API « lockdown » propriétaire. Ne doit pas invoquer RebootDevice ou EraseDevice.",
        },
        "dfir-002" => &FrRequirement {
            title: "Triage silencieux non destructif",
            description: "Collecte des journaux système, inventaires d'applications et télémétrie appareil sans alerter ni perturber l'utilisateur final.",
            evaluation_method: "Lancer une collecte de journaux à distance et un inventaire d'applications sur un appareil de test. Confirmer l'absence de notification visible, d'interruption UI et de redémarrage. Valider l'exhaustivité des journaux (syslog, crash logs, liste des applications installées).",
            technical_criteria: "iOS : flux de journaux MDM (supervisé), récupération de journaux de diagnostic. Android : bugreport via Device Owner, inventaire via ApplicationReport API. Les opérations doivent utiliser le canal MDM en arrière-plan, sans invite utilisateur.",
        },
        "dfir-003" => &FrRequirement {
            title: "API de streaming des journaux d'audit vers SIEM",
            description: "Streaming API direct des journaux d'audit administratifs et de la télémétrie appareil vers les plateformes SIEM d'entreprise (Splunk, Sentinel, etc.).",
            evaluation_method: "Activer l'export des journaux d'audit vers un point de terminaison SIEM de test. Effectuer des actions admin (changement de politique, effacement, inscription) et vérifier l'arrivée des événements via API/webhook dans le SLA défini (< 5 min). Confirmer le mapping des champs vers CEF/ECS.",
            technical_criteria: "API REST ou flux syslog pour les événements d'audit admin, changements de conformité, inscriptions. Doit prendre en charge Splunk HEC, connecteur Azure Sentinel ou webhook HTTPS générique avec JSON structuré.",
        },
        "plat-001" => &FrRequirement {
            title: "Apple Business Manager et mode supervisé iOS",
            description: "Inscription et gestion complètes via ABM/ADE avec toutes les capacités de politique en mode supervisé.",
            evaluation_method: "Inscrire un appareil via l'inscription automatisée ABM (ADE). Vérifier le drapeau de supervision, l'attribution du profil DEP et la capacité à pousser toutes les restrictions réservées au mode supervisé (verrouillage d'app, installation silencieuse, kiosque, flux de journaux).",
            technical_criteria: "API Apple Business Manager, inscription MDM DEP, propriété IsSupervised, profils de configuration avec restrictions PayloadType disponibles uniquement en mode supervisé (ex. com.apple.applicationaccess).",
        },
        "plat-002" => &FrRequirement {
            title: "Modèles de déploiement Android Enterprise",
            description: "Prise en charge des modes profil professionnel, COBO (Corporate-Owned Business Only) et kiosque dédié.",
            evaluation_method: "Inscrire trois appareils de test : BYOD profil professionnel, COBO Device Owner et kiosque dédié (COSU). Vérifier le ciblage des politiques, le déploiement d'applications et les capacités de gestion selon le mode.",
            technical_criteria: "Inscription Android Enterprise zero-touch / QR / NFC, provisionnement Device Owner, configurations gérées, mode kiosque COSU via lock task packages, API android.management (Google AMAPI) ou API EMM éditeur.",
        },
        "plat-003" => &FrRequirement {
            title: "Prise en charge OEMConfig",
            description: "Intégration OEMConfig native pour l'application de politiques spécifiques aux constructeurs Android majeurs (Samsung Knox, Zebra, etc.).",
            evaluation_method: "Déployer l'application de configuration gérée OEMConfig sur des appareils Samsung et Zebra de test. Pousser des restrictions OEM (conteneur Knox, Zebra MX) via MDM et vérifier l'application au niveau matériel/firmware.",
            technical_criteria: "Configurations gérées Android Enterprise (RESTRICTED_KEYSET), politiques Samsung Knox MDM SDK, schéma XML Zebra OEMConfig, paires clé-valeur de configuration d'application gérée via MDM ApplicationPolicy.",
        },
        _ => return None,
    })
}

pub fn localized_pillar_fields(pillar: &Pillar, locale: ReportLocale) -> (String, String) {
    (
        pillar_name(&pillar.id, &pillar.name, locale).into_owned(),
        pillar_description(&pillar.id, pillar.description.trim(), locale).into_owned(),
    )
}
