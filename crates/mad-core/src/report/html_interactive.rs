use std::collections::HashMap;

use serde_json::json;

use crate::evaluation::{EvaluationReport, EvaluationResult};
use crate::policy::PolicyBundle;
use crate::report::locale::ReportStrings;
use crate::vendor_doc::VendorDocSection;
use crate::value_stream::ValueStreamEntry;

pub const INTERACTIVE_SCRIPT: &str = include_str!("assets/mad-report-interactive.js");

pub const INTERACTIVE_STYLES: &str = r#"
body.mad-interactive { padding-top: 0; }
.mad-hidden { display: none !important; }
.mad-topbar {
  position: sticky; top: 0; z-index: 100;
  background: rgba(10, 22, 40, 0.97); backdrop-filter: blur(8px);
  border-bottom: 2px solid var(--cyan); padding: 0.5rem 1rem;
}
.mad-topbar-inner {
  max-width: 1200px; margin: 0 auto;
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem 1rem;
}
#mad-report-nav { display: flex; flex-wrap: wrap; gap: 0.35rem; }
#mad-report-nav a {
  color: var(--silver); text-decoration: none; font-size: 0.78rem; font-weight: 600;
  padding: 0.3rem 0.65rem; border-radius: 4px; transition: background 0.15s, color 0.15s;
}
#mad-report-nav a:hover, #mad-report-nav a.active {
  background: var(--cyan); color: var(--navy);
}
#mad-vendor-filter { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-left: auto; }
#mad-vendor-filter button, .mad-doc-filter button, .mad-btn-sm {
  font-family: inherit; font-size: 0.75rem; font-weight: 600; cursor: pointer;
  border: 1px solid #c8d0da; background: white; color: var(--navy);
  padding: 0.25rem 0.55rem; border-radius: 4px;
}
#mad-vendor-filter button.active, .mad-doc-filter button.active {
  background: var(--cyan); border-color: var(--cyan); color: var(--navy);
}
.mad-layout {
  max-width: 1200px; margin: 0 auto; padding: 1.25rem;
  display: grid; grid-template-columns: 200px 1fr; gap: 1.25rem; align-items: start;
}
@media (max-width: 900px) { .mad-layout { grid-template-columns: 1fr; } .mad-sidebar { display: none; } }
.mad-sidebar {
  position: sticky; top: 4.5rem; background: white; border-radius: 8px;
  padding: 1rem; box-shadow: 0 2px 8px rgba(10,22,40,0.08); font-size: 0.82rem;
}
.mad-sidebar h4 { color: var(--navy); font-size: 0.75rem; text-transform: uppercase;
  letter-spacing: 0.04em; margin-bottom: 0.5rem; }
.mad-sidebar ul { list-style: none; padding: 0; }
.mad-sidebar a { color: var(--muted); text-decoration: none; display: block; padding: 0.25rem 0; }
.mad-sidebar a:hover { color: var(--cyan); }
.mad-dashboard {
  display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.75rem; margin-bottom: 1.25rem;
}
.mad-score-card {
  background: white; border-radius: 8px; padding: 1rem;
  box-shadow: 0 2px 8px rgba(10,22,40,0.08); border-left: 4px solid var(--cyan);
  cursor: pointer; transition: transform 0.15s;
}
.mad-score-card:hover { transform: translateY(-2px); }
.mad-score-card h4 { font-size: 0.85rem; color: var(--navy); margin-bottom: 0.35rem; }
.mad-score-pct { font-size: 1.5rem; font-weight: 800; font-family: var(--mono); }
.mad-score-bar { height: 6px; background: #eceff1; border-radius: 3px; margin-top: 0.5rem; overflow: hidden; }
.mad-score-bar-fill { height: 100%; border-radius: 3px; }
.mad-pillar-details { margin-bottom: 0.75rem; border: 1px solid #e2e6ea; border-radius: 8px; }
.mad-pillar-details summary {
  padding: 0.75rem 1rem; cursor: pointer; font-weight: 600; color: var(--navy);
  list-style: none; display: flex; align-items: center; gap: 0.5rem;
}
.mad-pillar-details summary::-webkit-details-marker { display: none; }
.mad-pillar-details[open] summary { border-bottom: 1px solid #e2e6ea; }
.mad-pillar-details .mad-pillar-body { padding: 0.5rem 1rem 1rem; }
.mad-vendor-toggle {
  margin-left: auto; font-size: 0.75rem; padding: 0.2rem 0.5rem;
  border: 1px solid #c8d0da; background: white; border-radius: 4px; cursor: pointer;
}
.vendor-card.mad-collapsed .vendor-pillar-tables { display: none; }
.mad-vsm-viewer { position: relative; }
.mad-vsm-toolbar {
  display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-bottom: 0.75rem;
}
.mad-vsm-tabs { display: flex; gap: 0.25rem; }
.mad-vsm-tabs button {
  font-family: inherit; font-size: 0.78rem; font-weight: 600; padding: 0.35rem 0.75rem;
  border: 1px solid #c8d0da; background: #f4f6f8; color: var(--navy); border-radius: 4px; cursor: pointer;
}
.mad-vsm-tabs button.active { background: var(--navy); color: white; border-color: var(--navy); }
.mad-vsm-zoom { display: flex; gap: 0.25rem; margin-left: auto; }
.mad-vsm-zoom button {
  width: 28px; height: 28px; font-size: 1rem; line-height: 1;
  border: 1px solid #c8d0da; background: white; border-radius: 4px; cursor: pointer;
}
.mad-vsm-stage {
  position: relative; overflow: hidden; background: #f8fafc; border-radius: 8px;
  min-height: 280px; cursor: grab; touch-action: none;
}
.mad-vsm-stage:active { cursor: grabbing; }
.mad-vsm-stage .vsm-diagram-wrap { margin: 0 auto; }
.mad-vsm-node { cursor: pointer; }
.mad-vsm-node .vsm-node { transition: box-shadow 0.12s, border-color 0.12s; }
.mad-vsm-node:hover .vsm-node { box-shadow: 0 4px 14px rgba(10, 22, 40, 0.14); }
.mad-vsm-node.selected .vsm-node {
  border-color: var(--vsm-accent, var(--cyan)) !important;
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--vsm-accent, var(--cyan)) 25%, transparent), 0 4px 14px rgba(10, 22, 40, 0.14) !important;
}
.mad-vsm-node.selected .vsm-node-decision-wrap .vsm-node-decision {
  border-color: var(--vsm-accent, #e6a800);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--vsm-accent, #e6a800) 25%, transparent);
}
.mad-vsm-inspector {
  position: absolute; top: 0.75rem; right: 0.75rem; width: min(260px, 90%);
  background: white; border-radius: 8px; padding: 1rem;
  box-shadow: 0 4px 16px rgba(10,22,40,0.15); font-size: 0.85rem; z-index: 10;
}
.mad-vsm-inspector h5 { color: var(--navy); margin-bottom: 0.5rem; }
.mad-vsm-inspector dl { display: grid; grid-template-columns: auto 1fr; gap: 0.2rem 0.75rem; margin: 0; }
.mad-vsm-inspector dt { color: var(--muted); font-size: 0.75rem; }
.mad-vsm-inspector dd { margin: 0; font-weight: 600; }
.mad-vsm-notes { margin-top: 0.5rem; font-size: 0.8rem; color: var(--muted); }
.mad-vsm-payload { display: none; }
.mad-vsm-timeline-bar { cursor: pointer; transition: transform 0.12s, box-shadow 0.12s; }
.mad-vsm-timeline-bar:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(10,22,40,0.12); }
.mad-vsm-timeline-bar.selected { outline: 2px solid var(--cyan); outline-offset: 2px; box-shadow: 0 0 0 3px rgba(0,180,216,0.25); }
.mad-vsm-timeline-bar.longest { box-shadow: inset 0 0 0 1px rgba(255,255,255,0.35); }
.mad-vsm-timeline-milestone { cursor: pointer; }
.mad-vsm-timeline-milestone:hover { border-color: var(--cyan); }
.mad-vsm-timeline-milestone.selected { border-color: var(--cyan); box-shadow: 0 0 0 2px rgba(0,180,216,0.2); }
.mad-vsm-timeline-row { cursor: pointer; }
.mad-vsm-timeline-row:hover { background: #f0fbff; }
.mad-vsm-timeline-row.selected { background: #e3f8fd; }
.mad-vsm-edge path { transition: stroke-width 0.12s; pointer-events: stroke; stroke-width: 2; }
.mad-vsm-edge.selected path { stroke-width: 3.5; stroke-opacity: 1; }
.mad-vsm-node.edge-source .vsm-node,
.mad-vsm-node.edge-target .vsm-node {
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--cyan) 25%, transparent), 0 4px 14px rgba(10, 22, 40, 0.14);
}
.mad-vsm-node.edge-source { filter: drop-shadow(0 0 3px rgba(0,180,216,0.35)); }
.mad-vsm-node.edge-target { filter: drop-shadow(0 0 3px rgba(30,136,229,0.35)); }
.mad-doc-filter { display: flex; flex-wrap: wrap; gap: 0.35rem; margin: 0.75rem 0; }
.mad-doc-filter .mad-color-swatch {
  display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 0.25rem;
}
body.mad-embed .header, body.mad-embed .mad-topbar, body.mad-embed .mad-sidebar, body.mad-embed .footer { display: none; }
body.mad-embed .mad-layout { grid-template-columns: 1fr; padding: 0.5rem; max-width: 100%; }
body.mad-embed .container { max-width: 100%; padding: 0.5rem; }
@media print {
  .mad-topbar, .mad-sidebar, .mad-vsm-toolbar, .mad-vsm-inspector, .mad-vendor-toggle, .mad-doc-filter { display: none !important; }
  .mad-vsm-stage { overflow: visible; }
}
"#;

pub fn render_topbar(
    vendors: &[&EvaluationResult],
    has_vsm: bool,
    has_docs: bool,
    s: &ReportStrings,
) -> String {
    let mut out = String::from(
        "<nav class=\"mad-topbar\" aria-label=\"Report navigation\">\n  <div class=\"mad-topbar-inner\">\n",
    );
    out.push_str("    <div id=\"mad-report-nav\">\n");
    for (id, label) in [
        ("section-methodology", s.nav_method),
        ("section-requirements", s.nav_requirements),
        ("section-results", s.nav_results),
    ] {
        out.push_str(&format!("      <a href=\"#{id}\">{label}</a>\n"));
    }
    if has_vsm {
        out.push_str(&format!("      <a href=\"#section-vsm\">{}</a>\n", s.nav_vsm));
    }
    if has_docs {
        out.push_str(&format!("      <a href=\"#section-docs\">{}</a>\n", s.nav_docs));
    }
    out.push_str("    </div>\n");

    if !vendors.is_empty() {
        out.push_str(&format!(
            "    <div id=\"mad-vendor-filter\" role=\"group\" aria-label=\"{}\">\n",
            escape_attr(s.filter_vendor_aria)
        ));
        out.push_str(&format!(
            "      <button type=\"button\" data-vendor=\"all\" class=\"active\">{}</button>\n",
            escape_html(s.filter_all_vendors)
        ));
        for result in vendors {
            out.push_str(&format!(
                "      <button type=\"button\" data-vendor=\"{}\">{}</button>\n",
                escape_attr(&result.vendor.id.0),
                escape_html(&result.vendor.name),
            ));
        }
        out.push_str("    </div>\n");
    }

    out.push_str("  </div>\n</nav>\n");
    out
}

pub fn render_sidebar(s: &ReportStrings) -> String {
    format!(
        r##"<aside class="mad-sidebar" aria-label="Table of contents">
  <h4>{heading}</h4>
  <ul>
    <li><a href="#section-methodology">{method}</a></li>
    <li><a href="#section-requirements">{req}</a></li>
    <li><a href="#section-results">{res}</a></li>
    <li><a href="#section-vsm">{vsm}</a></li>
    <li><a href="#section-docs">{docs}</a></li>
  </ul>
</aside>
"##,
        heading = s.toc_heading,
        method = s.toc_methodology,
        req = s.toc_requirements,
        res = s.toc_results,
        vsm = s.toc_vsm,
        docs = s.toc_docs,
    )
}

pub fn render_dashboard(
    ranked: &[&EvaluationResult],
    score_color: fn(f64) -> &'static str,
    s: &ReportStrings,
) -> String {
    let mut out = String::from("<div class=\"mad-dashboard\" id=\"section-dashboard\">\n");
    for (rank, result) in ranked.iter().enumerate() {
        let score = result.overall_score.overall_score_percent;
        let color = score_color(score);
        out.push_str(&format!(
            "  <div class=\"mad-score-card\" data-vendor-id=\"{}\">\n",
            escape_attr(&result.vendor.id.0),
        ));
        out.push_str(&format!(
            "    <h4>#{} {}</h4>\n    <div class=\"mad-score-pct\" style=\"color:{color}\">{score:.1}%</div>\n",
            rank + 1,
            escape_html(&result.vendor.name),
        ));
        out.push_str(&format!(
            "    <div class=\"mad-score-bar\"><div class=\"mad-score-bar-fill\" style=\"width:{score:.0}%;background:{color}\"></div></div>\n",
        ));
        if !result.overall_score.critical_gaps.is_empty() {
            out.push_str(&format!(
                "    <p class=\"muted\" style=\"margin-top:0.4rem;font-size:0.75rem\">{} {}</p>\n",
                result.overall_score.critical_gaps.len(),
                s.critical_gaps_count
            ));
        }
        out.push_str("  </div>\n");
    }
    out.push_str("</div>\n");
    out
}

pub fn render_payload_script(
    bundle: &PolicyBundle,
    evaluation: &EvaluationReport,
    value_streams: &HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: &HashMap<String, Vec<VendorDocSection>>,
    s: &ReportStrings,
) -> String {
    let vendors: Vec<_> = evaluation
        .vendors
        .iter()
        .map(|r| {
            json!({
                "id": r.vendor.id.0,
                "name": r.vendor.name,
                "score": r.overall_score.overall_score_percent,
                "critical_gaps": r.overall_score.critical_gaps,
            })
        })
        .collect();

    let pillars: Vec<_> = bundle
        .pillars
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "requirements": p.requirements.iter().map(|r| json!({
                    "id": r.id,
                    "title": r.title,
                    "severity": format!("{:?}", r.severity).to_lowercase(),
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    let payload = json!({
        "version": evaluation.policy_version,
        "vendors": vendors,
        "pillars": pillars,
        "value_stream_vendors": value_streams.keys().collect::<Vec<_>>(),
        "doc_vendors": vendor_docs.keys().collect::<Vec<_>>(),
        "ui": {
            "close": s.js_close,
            "expand": s.expand,
            "collapse": s.collapse,
            "type": s.js_type,
            "author": s.js_author,
            "role": s.js_role,
            "leadTime": s.js_lead_time,
            "cycleTime": s.js_cycle_time,
            "flowType": s.js_flow_type,
            "duration": s.js_duration,
            "label": s.js_label,
        },
    });

    let json_str = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into());
    format!(
        "<script type=\"application/json\" id=\"mad-report-data\">{json_str}</script>\n\
         <script>{INTERACTIVE_SCRIPT}</script>\n"
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_attr(s: &str) -> String {
    escape_html(s)
}
