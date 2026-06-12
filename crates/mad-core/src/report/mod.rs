mod html;
mod html_interactive;
mod locale;
mod pdf;
pub mod vendor_doc;
pub mod vsm;

use std::collections::HashMap;

use crate::evaluation::EvaluationReport;
use crate::policy::PolicyBundle;
use crate::vendor_doc::VendorDocSection;
use crate::value_stream::ValueStreamEntry;

pub use html::{load_logo_data_uri, render_html, HtmlReportOptions};
pub use locale::ReportLocale;
pub use pdf::{render_pdf, PdfReportOptions};

/// Renders a detailed technical evaluation report in Markdown.
pub fn render_markdown(
    bundle: &PolicyBundle,
    evaluation: &EvaluationReport,
    value_streams: &HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: &HashMap<String, Vec<VendorDocSection>>,
) -> String {
    let mut out = String::new();

    out.push_str("# MAD — Mobile Assessment & Defense — Technical Evaluation Report\n\n");
    out.push_str("**Scope:** Mobile Device Management (MDM) solutions for iOS and Android only.\n\n");
    out.push_str(&format!(
        "**Policy version:** {}  \n**Requirements evaluated:** {} ({} critical)\n\n",
        evaluation.policy_version,
        evaluation.total_requirements,
        evaluation.critical_requirements
    ));

    out.push_str("---\n\n");
    out.push_str("## 1. Evaluation Methodology\n\n");
    out.push_str(
        "Vendors are assessed using **Policy-as-Code (PaC)**. Requirements are declared in \
         version-controlled YAML (`policies/mad-standard.yaml`). Each requirement maps to \
         a discrete compliance status:\n\n\
         | Status | Meaning |\n\
         |--------|----------|\n\
         | `compliant` | Vendor natively satisfies the requirement without third-party workarounds |\n\
         | `partial` | Capability exists but is limited, platform-specific, or requires manual steps |\n\
         | `non_compliant` | Requirement cannot be met with current vendor capabilities |\n\
         | `untested` | No assessment data recorded |\n\n",
    );

    out.push_str("### Scoring algorithm\n\n");
    out.push_str(
        "Per-pillar score:\n\n\
         ```\n\
         score = ((compliant × 1.0) + (partial × 0.5)) / total_requirements × 100\n\
         ```\n\n\
         Overall vendor score is the **arithmetic mean** of the three pillar scores. \
         Critical requirements that are `non_compliant` or `untested` are flagged as \
         **critical gaps** regardless of percentage score.\n\n",
    );

    out.push_str("### Data flow\n\n");
    out.push_str(
        "```\n\
policies/*.yaml  →  PolicyBundle (mad-core)  →  Evaluator\n\
                                                      ↓\n\
VendorAssessment (per requirement status)  →  EvaluationReport\n\
                                                      ↓\n\
                              CLI / REST API / Web dashboard\n\
```\n\n",
    );

    out.push_str("---\n\n");
    out.push_str("## 2. Evaluation Pillars and Technical Criteria\n\n");

    for pillar in &bundle.pillars {
        out.push_str(&format!("### {}\n\n", pillar.name));
        out.push_str(&format!("{}\n\n", pillar.description.trim()));

        for req in &pillar.requirements {
            let severity = match req.severity {
                crate::pillar::RequirementSeverity::Critical => "CRITICAL",
                crate::pillar::RequirementSeverity::High => "HIGH",
                crate::pillar::RequirementSeverity::Medium => "MEDIUM",
            };
            out.push_str(&format!(
                "#### [{severity}] `{}` — {}\n\n",
                req.id, req.title
            ));
            out.push_str(&format!("{}\n\n", req.description.trim()));
            out.push_str(&format!(
                "- **Platforms:** {}\n",
                req.platforms.join(", ")
            ));
            if let Some(method) = &req.evaluation_method {
                out.push_str(&format!("- **Evaluation method:** {method}\n"));
            }
            if let Some(criteria) = &req.technical_criteria {
                out.push_str(&format!("- **Technical criteria:** {criteria}\n"));
            }
            out.push('\n');
        }
    }

    out.push_str("---\n\n");
    out.push_str("## 3. Vendor Assessment Results\n\n");

    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        b.overall_score
            .overall_score_percent
            .partial_cmp(&a.overall_score.overall_score_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (rank, result) in ranked.iter().enumerate() {
        out.push_str(&format!(
            "### #{} {} — {:.1}% overall\n\n",
            rank + 1,
            result.vendor.name,
            result.overall_score.overall_score_percent
        ));
        out.push_str(&format!("{}\n\n", result.vendor.description));

        for pillar in &result.pillars {
            out.push_str(&format!(
                "#### {} — {:.1}%\n\n",
                pillar.pillar_name, pillar.score.score_percent
            ));
            out.push_str("| Requirement | Status | Notes |\n");
            out.push_str("|-------------|--------|-------|\n");
            for req in &pillar.requirements {
                let status = format!("{:?}", req.status).to_lowercase();
                let notes = req.notes.as_deref().unwrap_or("—");
                out.push_str(&format!(
                    "| `{}` {} | {} | {} |\n",
                    req.requirement_id, req.title, status, notes
                ));
            }
            out.push('\n');
        }

        if !result.overall_score.critical_gaps.is_empty() {
            out.push_str("**Critical gaps:**\n\n");
            for gap in &result.overall_score.critical_gaps {
                out.push_str(&format!("- {gap}\n"));
            }
            out.push('\n');
        }
    }

    let mut section = 4u8;
    if vsm::any_value_streams(value_streams) {
        out.push_str("---\n\n");
        out.push_str(&format!("## {section}. Value Stream Maps\n\n"));
        out.push_str(
            "Process flow diagrams captured during vendor evaluation. Durations are cumulative \
             lead times along each flow.\n\n",
        );
        for result in &evaluation.vendors {
            if let Some(entries) = value_streams.get(&result.vendor.id.0) {
                for entry in entries {
                    if vsm::map_has_content(&entry.map) {
                        let title = format!("{} — {}", result.vendor.name, entry.name);
                        out.push_str(&vsm::render_vsm_markdown_section(&title, &entry.map));
                    }
                }
            }
        }
        section += 1;
    }

    if vendor_doc::any_vendor_docs(vendor_docs) {
        out.push_str("---\n\n");
        out.push_str(&format!("## {section}. Vendor Documentation\n\n"));
        out.push_str(
            "User-defined per-vendor documentation (e.g. privacy, support, compliance). \
             Informational only — not included in capability scores.\n\n",
        );
        for result in &evaluation.vendors {
            if let Some(sections) = vendor_docs.get(&result.vendor.id.0) {
                out.push_str(&vendor_doc::render_all_vendor_docs_markdown(
                    &result.vendor.name,
                    sections,
                ));
            }
        }
    }

    out.push_str("---\n\n");
    out.push_str(
        "*Generated by MAD (mad-core). This report reflects sample assessments \
         for demonstration; production evaluations require live API probes and lab validation.*\n",
    );

    out
}

/// Builds evaluation report options with an optional embedded logo and current timestamp.
pub fn default_html_options(logo_path: Option<&std::path::Path>) -> HtmlReportOptions {
    HtmlReportOptions {
        logo_data_uri: logo_path.and_then(load_logo_data_uri),
        generated_at: Some(now_rfc3339()),
        interactive: true,
        locale: ReportLocale::En,
        filter_tags: Vec::new(),
    }
}

pub fn default_pdf_options(logo_path: Option<&std::path::Path>) -> PdfReportOptions {
    PdfReportOptions {
        generated_at: Some(now_rfc3339()),
        logo_png: logo_path.and_then(|p| std::fs::read(p).ok()),
    }
}

fn now_rfc3339() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}
