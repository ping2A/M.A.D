use std::collections::HashMap;

use crate::evaluation::EvaluationReport;
use crate::policy::PolicyBundle;
use crate::report::{html_interactive, locale, vendor_doc, vsm};
use crate::report::locale::ReportLocale;
use crate::vendor_doc::VendorDocSection;
use crate::value_stream::ValueStreamEntry;
use crate::vendor::ComplianceStatus;

/// Options for HTML report generation.
#[derive(Debug, Clone, Default)]
pub struct HtmlReportOptions {
    /// `data:image/png;base64,...` URI embedded in the document.
    pub logo_data_uri: Option<String>,
    /// ISO-8601 timestamp shown in the report footer.
    pub generated_at: Option<String>,
    /// Interactive navigation, VSM pan/zoom, vendor filters (default: true).
    pub interactive: bool,
    /// Report UI language.
    pub locale: ReportLocale,
    /// Active vendor tag filter (shown in meta bar when non-empty).
    pub filter_tags: Vec<String>,
}

pub fn load_logo_data_uri(path: &std::path::Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    Some(format!("data:image/png;base64,{encoded}"))
}

/// Renders a self-contained HTML report (inline CSS, optional embedded logo).
/// Suitable for sharing as a single file via email, SharePoint, or file share.
pub fn render_html(
    bundle: &PolicyBundle,
    evaluation: &EvaluationReport,
    value_streams: &HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: &HashMap<String, Vec<VendorDocSection>>,
    options: &HtmlReportOptions,
) -> String {
    let generated_at = options
        .generated_at
        .as_deref()
        .unwrap_or("—");

    let mut ranked: Vec<_> = evaluation.vendors.iter().collect();
    ranked.sort_by(|a, b| {
        b.overall_score
            .overall_score_percent
            .partial_cmp(&a.overall_score.overall_score_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let interactive = options.interactive;
    let locale = options.locale;
    let s = locale::strings(locale);
    let has_vsm = vsm::any_value_streams(value_streams);
    let has_docs = vendor_doc::any_vendor_docs(vendor_docs);

    let mut out = String::with_capacity(128 * 1024);
    out.push_str("<!DOCTYPE html>\n<html lang=\"");
    out.push_str(s.html_lang);
    out.push_str("\">\n<head>\n");
    out.push_str("<meta charset=\"UTF-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    out.push_str("<title>");
    out.push_str(s.page_title);
    out.push_str("</title>\n");
    out.push_str("<style>\n");
    out.push_str(HTML_STYLES);
    if interactive {
        out.push_str(html_interactive::INTERACTIVE_STYLES);
    }
    out.push_str("</style>\n</head>\n<body");
    if interactive {
        out.push_str(" class=\"mad-interactive\"");
    }
    out.push_str(">\n");

    // Header
    out.push_str("<header class=\"header\">\n  <div class=\"header-inner\">\n");
    if let Some(logo) = &options.logo_data_uri {
        out.push_str("    <img class=\"logo\" src=\"");
        out.push_str(logo);
        out.push_str("\" alt=\"MAD logo\">\n");
    }
    out.push_str("    <div>\n");
    out.push_str("      <h1>MAD</h1>\n");
    out.push_str("      <p class=\"subtitle\">");
    out.push_str(s.subtitle);
    out.push_str("</p>\n");
    out.push_str("    </div>\n  </div>\n</header>\n");

    if interactive {
        out.push_str(&html_interactive::render_topbar(&ranked, has_vsm, has_docs, s));
        out.push_str("<div class=\"mad-layout\">\n");
        out.push_str(&html_interactive::render_sidebar(s));
        out.push_str("<div class=\"mad-content\">\n");
    }

    out.push_str("<main class=\"container\">\n");

    // Meta bar
    out.push_str("<div class=\"meta-bar\">\n");
    out.push_str(&format!(
        "  <span><strong>{}</strong> v{}</span>\n",
        s.meta_policy,
        escape_html(&evaluation.policy_version)
    ));
    out.push_str(&format!(
        "  <span><strong>{}</strong> {} ({} critical)</span>\n",
        s.meta_requirements,
        evaluation.total_requirements,
        evaluation.critical_requirements
    ));
    out.push_str(&format!(
        "  <span><strong>{}</strong> {}</span>\n",
        s.meta_vendors,
        evaluation.vendors.len()
    ));
    out.push_str(&format!(
        "  <span><strong>{}</strong> {}</span>\n",
        s.meta_generated,
        escape_html(generated_at)
    ));
    out.push_str("  <span class=\"scope-tag\">");
    out.push_str(s.meta_scope);
    out.push_str("</span>\n");
    if !options.filter_tags.is_empty() {
        out.push_str(&format!(
            "  <span class=\"scope-tag filter-tag\">{} {}</span>\n",
            s.meta_tags_filter,
            escape_html(&options.filter_tags.join(", "))
        ));
    }
    out.push_str("</div>\n");

    if interactive {
        out.push_str(&html_interactive::render_dashboard(&ranked, score_color, s));
    }

    // Section 1: Methodology
    out.push_str("<section class=\"card\" id=\"section-methodology\">\n");
    out.push_str("  <h2>");
    out.push_str(s.section_methodology);
    out.push_str("</h2>\n");
    out.push_str("  <table class=\"data-table\">\n");
    out.push_str(&format!(
        "    <thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead>\n",
        s.col_status, s.col_weight, s.col_meaning
    ));
    out.push_str("    <tbody>\n");
    for (status, weight, meaning) in [
        ("compliant", "1.0", s.meaning_compliant),
        ("partial", "0.5", s.meaning_partial),
        ("non_compliant", "0.0", s.meaning_non_compliant),
        ("untested", "0.0", s.meaning_untested),
    ] {
        out.push_str(&format!(
            "      <tr><td><span class=\"badge status-{status}\">{status}</span></td>\
             <td>{weight}</td><td>{meaning}</td></tr>\n"
        ));
    }
    out.push_str("    </tbody>\n  </table>\n");
    out.push_str("  <pre class=\"code\">pillar_score = ((compliant × 1.0) + (partial × 0.5)) / total × 100\n\
overall_score  = mean(cybersecurity, dfir, platform_os)</pre>\n");
    out.push_str("</section>\n");

    // Section 2: Requirements
    out.push_str("<section class=\"card\" id=\"section-requirements\">\n");
    out.push_str("  <h2>");
    out.push_str(s.section_requirements);
    out.push_str("</h2>\n");
    for pillar in &bundle.pillars {
        let (pillar_name, pillar_desc) = locale::localized_pillar_fields(pillar, locale);
        if interactive {
            out.push_str(&format!(
                "  <details class=\"mad-pillar-details\" open>\n\
                   <summary>{} <span class=\"muted\">({} {})</span></summary>\n\
                   <div class=\"mad-pillar-body\">\n\
                   <p class=\"muted\">{}</p>\n",
                escape_html(&pillar_name),
                pillar.requirements.len(),
                s.requirements_count,
                escape_html(&pillar_desc)
            ));
        } else {
            out.push_str(&format!(
                "  <h3 class=\"pillar-title\">{}</h3>\n  <p class=\"muted\">{}</p>\n",
                escape_html(&pillar_name),
                escape_html(&pillar_desc)
            ));
        }
        for req in &pillar.requirements {
            let severity = locale::severity_label(req.severity, locale);
            let loc = locale::localize_requirement(req, locale);
            out.push_str("  <div class=\"requirement\">\n");
            out.push_str(&format!(
                "    <div class=\"req-header\">\
                 <code class=\"req-id\">{}</code>\
                 <span class=\"badge severity-{}\">{}</span>\
                 <strong>{}</strong></div>\n",
                escape_html(&req.id),
                severity.to_lowercase(),
                severity,
                escape_html(&loc.title)
            ));
            out.push_str(&format!(
                "    <p>{}</p>\n",
                escape_html(&loc.description)
            ));
            out.push_str(&format!(
                "    <p class=\"muted\"><strong>{}:</strong> {}</p>\n",
                s.platforms,
                escape_html(&req.platforms.join(", "))
            ));
            if let Some(m) = &loc.evaluation_method {
                out.push_str(&format!(
                    "    <div class=\"tech-box\"><strong>{}</strong><br>{}</div>\n",
                    s.evaluation_method,
                    escape_html(m.trim())
                ));
            }
            if let Some(c) = &loc.technical_criteria {
                out.push_str(&format!(
                    "    <div class=\"tech-box\"><strong>{}</strong><br>{}</div>\n",
                    s.technical_criteria,
                    escape_html(c.trim())
                ));
            }
            out.push_str("  </div>\n");
        }
        if interactive {
            out.push_str("  </div></details>\n");
        }
    }
    out.push_str("</section>\n");

    // Section 3: Vendor results
    out.push_str("<section class=\"card\" id=\"section-results\">\n");
    out.push_str("  <h2>");
    out.push_str(s.section_results);
    out.push_str("</h2>\n");
    for (rank, result) in ranked.iter().enumerate() {
        let score = result.overall_score.overall_score_percent;
        let collapse_btn = if interactive {
            format!(
                "    <button type=\"button\" class=\"mad-vendor-toggle\" data-expand=\"{}\" data-collapse=\"{}\">{}</button>\n",
                escape_html(s.expand),
                escape_html(s.collapse),
                escape_html(s.collapse),
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  <article class=\"vendor-card\" data-vendor-id=\"{}\">\n\
               <div class=\"vendor-header\">\n\
                 <span class=\"rank\">#{}</span>\n\
                 <h3>{}</h3>\n\
                 <span class=\"score\" style=\"color:{}\">{:.1}%</span>\n\
                 {collapse_btn}\
               </div>\n\
               <p class=\"muted\">{}</p>\n\
               <div class=\"vendor-pillar-tables\">\n",
            escape_html(&result.vendor.id.0),
            rank + 1,
            escape_html(&result.vendor.name),
            score_color(score),
            score,
            escape_html(&result.vendor.description)
        ));

        for pillar in &result.pillars {
            let ps = pillar.score.score_percent;
            let pillar_title = locale::pillar_name(&pillar.pillar_id, &pillar.pillar_name, locale);
            out.push_str(&format!(
                "    <h4>{} — <span style=\"color:{}\">{:.1}%</span></h4>\n",
                escape_html(&pillar_title),
                score_color(ps),
                ps
            ));
            out.push_str("    <table class=\"data-table compact\">\n");
            out.push_str(&format!(
                "      <thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead>\n",
                s.col_id, s.col_requirement, s.col_status, s.col_notes
            ));
            out.push_str("      <tbody>\n");
            for req in &pillar.requirements {
                let title = locale::requirement_title(&req.requirement_id, &req.title, locale);
                let (status_cell, notes_cell) = if req.applicable {
                    let status = status_str(req.status);
                    (
                        format!("<span class=\"badge status-{status}\">{status}</span>"),
                        escape_html(req.notes.as_deref().unwrap_or("—")),
                    )
                } else {
                    (
                        format!("<span class=\"badge status-na\">{}</span>", s.status_na),
                        "—".to_string(),
                    )
                };
                out.push_str(&format!(
                    "        <tr><td><code>{}</code></td><td>{}</td>\
                     <td>{}</td>\
                     <td>{}</td></tr>\n",
                    escape_html(&req.requirement_id),
                    escape_html(&title),
                    status_cell,
                    notes_cell,
                ));
            }
            out.push_str("      </tbody>\n    </table>\n");
        }

        if !result.overall_score.critical_gaps.is_empty() {
            out.push_str(&format!("    <div class=\"gaps\"><strong>{}</strong><ul>\n", s.critical_gaps));
            for gap in &result.overall_score.critical_gaps {
                out.push_str(&format!("      <li>{}</li>\n", escape_html(gap)));
            }
            out.push_str("    </ul></div>\n");
        }
        out.push_str("  </div></article>\n");
    }
    out.push_str("</section>\n");

    let mut section = 4u8;
    if has_vsm {
        out.push_str("<section class=\"card vsm-report\" id=\"section-vsm\">\n");
        out.push_str(&format!("  <h2>{section}. {}</h2>\n", s.section_vsm));
        out.push_str("  <p class=\"muted\">");
        out.push_str(s.vsm_intro);
        out.push_str("</p>\n");
        for result in &evaluation.vendors {
            if let Some(entries) = value_streams.get(&result.vendor.id.0) {
                for entry in entries {
                    if vsm::map_has_content(&entry.map) {
                        let title = format!("{} — {}", result.vendor.name, entry.name);
                        let opts = vsm::VsmHtmlOptions {
                            interactive,
                            vendor_id: Some(result.vendor.id.0.clone()),
                            locale,
                        };
                        out.push_str(&vsm::render_vsm_html_section(
                            &title,
                            &entry.map,
                            escape_html,
                            &opts,
                        ));
                    }
                }
            }
        }
        out.push_str("</section>\n");
        section += 1;
    }

    if has_docs {
        out.push_str("<section class=\"card vendor-doc-report\" id=\"section-docs\">\n");
        out.push_str(&format!("  <h2>{section}. {}</h2>\n", s.section_docs));
        out.push_str("  <p class=\"muted\">");
        out.push_str(s.docs_intro);
        out.push_str("</p>\n");
        for result in &evaluation.vendors {
            if let Some(sections) = vendor_docs.get(&result.vendor.id.0) {
                out.push_str(&vendor_doc::render_all_vendor_docs_html(
                    &result.vendor.name,
                    &result.vendor.id.0,
                    sections,
                    escape_html,
                    interactive,
                    locale,
                ));
            }
        }
        out.push_str("</section>\n");
    }

    out.push_str("</main>\n");

    if interactive {
        out.push_str("</div></div>\n");
        out.push_str(&html_interactive::render_payload_script(
            bundle,
            evaluation,
            value_streams,
            vendor_docs,
            s,
        ));
    }

    out.push_str("<footer class=\"footer\">");
    out.push_str(s.footer);
    out.push_str("</footer>\n");
    out.push_str("</body>\n</html>\n");

    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn status_str(status: ComplianceStatus) -> &'static str {
    match status {
        ComplianceStatus::Compliant => "compliant",
        ComplianceStatus::Partial => "partial",
        ComplianceStatus::NonCompliant => "non_compliant",
        ComplianceStatus::Untested => "untested",
    }
}

fn score_color(pct: f64) -> &'static str {
    if pct >= 90.0 {
        "#28a745"
    } else if pct >= 70.0 {
        "#e6a800"
    } else {
        "#dc3545"
    }
}

const HTML_STYLES: &str = r#"
:root {
  --navy: #0a1628;
  --navy-light: #132238;
  --cyan: #00b4d8;
  --silver: #c0c8d4;
  --text: #1a2332;
  --muted: #5a6578;
  --bg: #e8eaed;
  --compliant: #28a745;
  --partial: #e6a800;
  --gap: #dc3545;
  --font: "Segoe UI", system-ui, -apple-system, sans-serif;
  --mono: "Cascadia Code", "Fira Code", ui-monospace, monospace;
}
* { box-sizing: border-box; margin: 0; }
body { font-family: var(--font); color: var(--text); background: var(--bg); line-height: 1.5; }
.header {
  background: linear-gradient(135deg, var(--navy) 0%, #1e3a5f 100%);
  color: white; padding: 1.5rem 2rem; border-bottom: 3px solid var(--cyan);
}
.header-inner { max-width: 960px; margin: 0 auto; display: flex; align-items: center; gap: 1.25rem; }
.logo { width: 64px; height: 64px; border-radius: 8px; border: 2px solid var(--cyan); }
.header h1 { font-size: 1.5rem; }
.subtitle { color: var(--silver); font-size: 0.9rem; margin-top: 0.2rem; }
.container { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
.meta-bar {
  display: flex; flex-wrap: wrap; gap: 1rem; background: var(--navy-light); color: white;
  padding: 1rem 1.25rem; border-radius: 8px; margin-bottom: 1.25rem; font-size: 0.85rem;
}
.meta-bar strong { color: var(--cyan); }
.scope-tag {
  background: var(--cyan); color: var(--navy); padding: 0.15rem 0.6rem;
  border-radius: 4px; font-weight: 600; font-size: 0.75rem;
}
.card {
  background: white; border-radius: 10px; padding: 1.5rem; margin-bottom: 1.25rem;
  box-shadow: 0 2px 8px rgba(10,22,40,0.08);
}
.card h2 { color: var(--navy); font-size: 1.15rem; border-bottom: 2px solid var(--cyan);
  padding-bottom: 0.5rem; margin-bottom: 1rem; }
.card h3, .pillar-title { color: var(--navy); font-size: 1rem; margin: 1.25rem 0 0.5rem; }
.card h4 { color: var(--navy); font-size: 0.9rem; margin: 1rem 0 0.5rem; }
.muted { color: var(--muted); font-size: 0.9rem; }
.scope-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-top: 1rem; }
@media (max-width: 600px) { .scope-grid { grid-template-columns: 1fr; } }
.scope-in, .scope-out { padding: 1rem; border-radius: 8px; font-size: 0.9rem; }
.scope-in { background: #e8f5e9; border-left: 4px solid var(--compliant); }
.scope-out { background: #fde8ea; border-left: 4px solid var(--gap); }
.scope-in h3, .scope-out h3 { font-size: 0.8rem; text-transform: uppercase; margin-bottom: 0.5rem; }
.scope-in ul, .scope-out ul { padding-left: 1.25rem; }
.data-table { width: 100%; border-collapse: collapse; font-size: 0.85rem; margin: 1rem 0; }
.data-table th, .data-table td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #e2e6ea; }
.data-table th { background: var(--navy-light); color: white; }
.data-table.compact { font-size: 0.8rem; }
.code {
  background: var(--navy); color: var(--cyan); padding: 1rem; border-radius: 8px;
  font-family: var(--mono); font-size: 0.8rem; overflow-x: auto; margin-top: 1rem;
}
.badge {
  display: inline-block; font-size: 0.7rem; font-weight: 700; text-transform: uppercase;
  padding: 0.1rem 0.45rem; border-radius: 3px;
}
.severity-critical { background: #fde8ea; color: var(--gap); }
.severity-high { background: #fff3e0; color: #fd7e14; }
.severity-medium { background: #fff8e1; color: #b8860b; }
.status-compliant { background: #e8f5e9; color: var(--compliant); }
.status-partial { background: #fff8e1; color: var(--partial); }
.status-non_compliant { background: #fde8ea; color: var(--gap); }
.status-untested { background: #eceff1; color: var(--muted); }
.status-na { background: #f5f5f5; color: #9aa0a6; }
.requirement { padding: 1rem 0; border-bottom: 1px solid #e8eaed; }
.req-header { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.4rem; }
.req-id { font-family: var(--mono); font-size: 0.75rem; background: var(--navy-light);
  color: var(--cyan); padding: 0.1rem 0.4rem; border-radius: 3px; }
.tech-box {
  font-size: 0.8rem; color: var(--muted); background: #f4f6f8; padding: 0.5rem 0.75rem;
  border-radius: 4px; margin-top: 0.5rem; line-height: 1.5;
}
.vendor-card {
  border: 1px solid #e2e6ea; border-left: 4px solid var(--cyan); border-radius: 8px;
  padding: 1.25rem; margin-bottom: 1rem;
}
.vendor-header { display: flex; align-items: baseline; gap: 0.75rem; flex-wrap: wrap; }
.rank { font-size: 1.25rem; font-weight: 800; color: var(--navy); opacity: 0.3; }
.vendor-header h3 { flex: 1; font-size: 1.1rem; color: var(--navy); }
.score { font-size: 1.4rem; font-weight: 800; font-family: var(--mono); }
.gaps { background: #fde8ea; padding: 0.75rem 1rem; border-radius: 6px; margin-top: 1rem; font-size: 0.85rem; }
.gaps ul { padding-left: 1.25rem; margin-top: 0.4rem; }
.footer {
  text-align: center; padding: 1.5rem; font-size: 0.8rem; color: var(--muted);
  border-top: 1px solid #dde1e6;
}
.vsm-report-card {
  border: 1px solid #e2e6ea; border-radius: 8px; padding: 1.25rem; margin: 1.25rem 0;
  break-inside: avoid;
}
.vsm-report-card h3 { color: var(--navy); margin-bottom: 0.75rem; }
.vsm-report-card h4 { color: var(--navy); font-size: 0.9rem; margin: 1rem 0 0.5rem; }
.vsm-report-stats { display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.85rem; color: var(--muted); margin-bottom: 1rem; }
.vsm-report-stats strong { color: var(--navy); }
.vsm-report-diagram { overflow: auto; background: #f8fafc; border-radius: 8px; padding: 0.75rem; margin-bottom: 1rem; }
.vsm-diagram-wrap { position: relative; margin: 0 auto; }
.vsm-diagram-canvas { position: relative; width: 100%; height: 100%; transform-origin: 0 0; }
.vsm-edges-svg { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; }
.vsm-edges-layer { pointer-events: stroke; }
.vsm-nodes-layer { position: absolute; inset: 0; pointer-events: none; }
.vsm-html-node { position: absolute; pointer-events: auto; box-sizing: border-box; }
.vsm-edge-path-label {
  font-size: 10px; font-weight: 600; fill: var(--navy); font-family: var(--font);
  paint-order: stroke fill; stroke: white; stroke-width: 3px;
}
.vsm-node {
  position: relative; width: 100%; height: 100%; box-sizing: border-box;
  padding: 0.5rem 0.65rem; border-radius: 8px; border: 2px solid #dde1e6;
  background: white; font-size: 0.78rem; font-weight: 600; color: var(--navy);
  min-width: 0; box-shadow: 0 2px 10px rgba(10, 22, 40, 0.1);
}
.vsm-node-meta { display: flex; flex-wrap: wrap; gap: 0.25rem; margin-bottom: 0.15rem; }
.vsm-node-role {
  font-size: 0.62rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em;
  color: var(--vsm-accent, #00b4d8);
}
.vsm-node-author {
  font-size: 0.62rem; font-weight: 600; padding: 0.08rem 0.35rem; border-radius: 999px;
  background: color-mix(in srgb, var(--vsm-accent, #00b4d8) 14%, white); color: var(--navy);
  max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.vsm-node-metrics { display: flex; gap: 0.35rem; margin-top: 0.3rem; flex-wrap: wrap; }
.vsm-node-metrics span {
  font-size: 0.62rem; font-weight: 700; padding: 0.1rem 0.35rem; border-radius: 4px;
  background: color-mix(in srgb, var(--vsm-accent, #78909c) 12%, white); color: var(--navy);
}
.vsm-node-label { line-height: 1.3; word-break: break-word; }
.vsm-node-notes { font-size: 0.68rem; font-weight: 400; color: var(--muted); margin-top: 0.2rem; }
.vsm-node-process {
  border-color: color-mix(in srgb, var(--vsm-accent) 55%, white);
  background: linear-gradient(180deg, color-mix(in srgb, var(--vsm-accent) 8%, white) 0%, #fff 100%);
}
.vsm-node-info { border-color: #9fd4ad; background: #f4fbf6; border-style: dashed; }
.vsm-info-icon { position: absolute; top: 4px; right: 6px; font-size: 0.7rem; opacity: 0.7; }
.vsm-node-delay { border-color: #f0d080; background: #fffaf0; border-radius: 20px; }
.vsm-node-external {
  border-color: #b0bec5; background: #f5f7f9; border-radius: 4px; transform: skewX(-6deg);
}
.vsm-node-external .vsm-node-label { transform: skewX(6deg); }
.vsm-node-decision-wrap {
  padding: 0; border: none; background: transparent; box-shadow: none; overflow: visible;
}
.vsm-node-decision-wrap .vsm-node-author { position: absolute; top: -2px; left: 50%; transform: translateX(-50%); z-index: 1; }
.vsm-node-decision {
  width: 72px; height: 72px; margin: 14px auto; background: #fff8e8; border: 2px solid #e6a800;
  transform: rotate(45deg); display: flex; align-items: center; justify-content: center;
}
.vsm-node-decision span {
  transform: rotate(-45deg); font-size: 0.72rem; font-weight: 700; text-align: center;
  line-height: 1.2; max-width: 56px;
}
.vsm-node-customer {
  border-color: color-mix(in srgb, var(--vsm-accent) 50%, white);
  background: linear-gradient(135deg, #f5f6ff 0%, #fff 100%); border-radius: 12px 12px 4px 4px;
}
.vsm-node-supplier {
  border-color: color-mix(in srgb, var(--vsm-accent) 50%, white);
  background: #faf6f4; border-radius: 4px 4px 12px 12px;
}
.vsm-shape-icon { position: absolute; top: 4px; right: 6px; font-size: 0.85rem; opacity: 0.85; }
.vsm-node-inventory-wrap { padding: 0; border: none; background: transparent; box-shadow: none; }
.vsm-node-inventory {
  width: 0; height: 0; margin: 0 auto; border-left: 52px solid transparent;
  border-right: 52px solid transparent;
  border-bottom: 72px solid color-mix(in srgb, var(--vsm-accent) 18%, white);
  position: relative; filter: drop-shadow(0 2px 6px rgba(10, 22, 40, 0.12));
}
.vsm-node-inventory span {
  position: absolute; left: 50%; top: 28px; transform: translateX(-50%); width: 80px;
  text-align: center; font-size: 0.68rem; font-weight: 700; line-height: 1.2; color: var(--navy);
}
.vsm-node-kaizen-wrap {
  padding: 0; border: none; background: transparent; box-shadow: none;
  display: flex; align-items: center; justify-content: center;
}
.vsm-node-kaizen-wrap .vsm-node-author { position: absolute; top: -2px; left: 50%; transform: translateX(-50%); z-index: 1; }
.vsm-node-kaizen {
  width: 88px; height: 88px; background: radial-gradient(circle at 30% 30%, #fff5f8 0%, #fce4ec 100%);
  border: 2px dashed color-mix(in srgb, var(--vsm-accent) 70%, white); border-radius: 50%;
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.15rem;
  padding: 0.35rem; box-shadow: 0 2px 10px rgba(233, 30, 99, 0.15);
}
.vsm-kaizen-icon { font-size: 1rem; color: var(--vsm-accent); }
.vsm-kaizen-label { font-size: 0.62rem; font-weight: 700; text-align: center; line-height: 1.15; max-width: 72px; }
.vsm-report-legend ul { list-style: none; display: flex; flex-wrap: wrap; gap: 0.75rem 1.25rem; padding: 0; margin: 0; font-size: 0.8rem; }
.vsm-report-legend li { display: flex; align-items: center; gap: 0.4rem; }
.vsm-legend-line { display: inline-block; width: 28px; border-top: 3px solid; }
.vsm-legend-line.dashed { border-top-style: dashed; }
.vsm-flow-badge { font-size: 0.7rem; font-weight: 600; padding: 0.1rem 0.4rem; border-radius: 3px; }
.vsm-gantt { margin-top: 1rem; font-size: 0.8rem; }
.vsm-gantt-row { display: grid; grid-template-columns: minmax(120px, 1.4fr) 2fr auto; gap: 0.5rem; align-items: center; margin-bottom: 0.35rem; }
.vsm-gantt-label { color: var(--muted); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.vsm-gantt-track { background: #eceff1; border-radius: 4px; height: 14px; overflow: hidden; }
.vsm-gantt-bar { height: 100%; border-radius: 4px; min-width: 2px; }
.vsm-gantt-dur { font-family: var(--mono); font-size: 0.75rem; color: var(--navy); }
.vsm-messages { padding-left: 1.25rem; font-size: 0.85rem; color: var(--muted); }
.vendor-doc-report-card {
  border: 1px solid #e2e8f0; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem;
}
.vendor-doc-report-card h3 { color: var(--navy); margin-bottom: 0.5rem; }
.vendor-doc-report-card h4 { color: var(--navy); font-size: 0.9rem; margin: 1rem 0 0.5rem; }
.vendor-doc-item-list { list-style: none; padding: 0; margin: 0; }
.vendor-doc-item { padding: 0.65rem 0; border-bottom: 1px solid #eef1f5; font-size: 0.9rem; }
.vendor-doc-item:last-child { border-bottom: none; }
.vendor-doc-notes { display: block; margin-top: 0.35rem; font-size: 0.85rem; color: var(--navy); }
.mad-vsm-timeline-chart { margin: 1rem 0; padding: 1rem; background: #f8fafc; border-radius: 8px; border: 1px solid #e2e8f0; }
.mad-vsm-timeline-stats { display: flex; flex-wrap: wrap; gap: 0.75rem; margin-bottom: 1rem; }
.mad-vsm-timeline-stat { flex: 1; min-width: 120px; background: white; border-radius: 6px; padding: 0.65rem 0.75rem; border: 1px solid #e2e8f0; }
.mad-vsm-timeline-stat-primary { border-left: 3px solid var(--cyan); }
.mad-vsm-timeline-stat-label { display: block; font-size: 0.7rem; text-transform: uppercase; color: var(--muted); letter-spacing: 0.03em; }
.mad-vsm-timeline-stat strong { font-size: 1.05rem; color: var(--navy); }
.mad-vsm-timeline-track { display: flex; gap: 3px; min-height: 52px; align-items: stretch; margin-bottom: 0.5rem; }
.mad-vsm-timeline-bar {
  flex: 1 1 0; min-width: 48px; border: 2px solid transparent; border-radius: 6px; color: white;
  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 0.15rem;
  padding: 0.35rem 0.25rem; font-family: inherit; font-size: 0.75rem; position: relative;
}
.mad-vsm-timeline-bar.untimed { background: #eceff1 !important; color: var(--muted); border-style: dashed; }
.mad-vsm-timeline-bar-type { font-weight: 800; font-size: 0.65rem; opacity: 0.9; }
.mad-vsm-timeline-bar-label { font-weight: 700; font-family: var(--mono); font-size: 0.72rem; }
.mad-vsm-timeline-bar-pct { margin-left: 0.25rem; opacity: 0.85; font-size: 0.65rem; }
.mad-vsm-timeline-ruler { position: relative; height: 1.25rem; margin-bottom: 0.75rem; border-top: 1px solid #dde1e6; }
.mad-vsm-timeline-ruler-tick { position: absolute; top: 0.2rem; transform: translateX(-50%); font-size: 0.65rem; color: var(--muted); font-family: var(--mono); }
.mad-vsm-timeline-milestones { margin-top: 0.5rem; }
.mad-vsm-timeline-milestones-title { display: block; font-size: 0.7rem; text-transform: uppercase; color: var(--muted); margin-bottom: 0.35rem; }
.mad-vsm-timeline-milestone-lane { position: relative; min-height: 2.5rem; }
.mad-vsm-timeline-milestone {
  position: absolute; transform: translateX(-50%); background: white; border: 1px solid #dde1e6;
  border-radius: 6px; padding: 0.2rem 0.45rem; font-size: 0.7rem; display: flex; align-items: center; gap: 0.3rem;
  font-family: inherit; color: var(--navy); max-width: 120px;
}
.mad-vsm-timeline-milestone-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--milestone-color, var(--cyan)); flex-shrink: 0; }
.mad-vsm-timeline-milestone-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.mad-vsm-timeline-details { margin-top: 1rem; }
.mad-vsm-timeline-details summary { cursor: pointer; font-weight: 600; color: var(--navy); margin-bottom: 0.5rem; }
@media print {
  body { background: white; }
  .card { box-shadow: none; border: 1px solid #ddd; break-inside: avoid; }
  .header { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  .vsm-gantt-bar, .vsm-flow-badge, .vsm-node, .vsm-node-decision, .vsm-node-inventory, .mad-vsm-edge path { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
}
"#;
