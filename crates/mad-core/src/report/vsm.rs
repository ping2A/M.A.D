use std::collections::HashMap;

use crate::value_stream::{ValueStreamEntry, ValueStreamMap, VsmEdge, VsmNode, VsmNodeType};

pub const MINUTES_PER_HOUR: f64 = 60.0;
pub const MINUTES_PER_DAY: f64 = MINUTES_PER_HOUR * 24.0;
pub const MINUTES_PER_WEEK: f64 = MINUTES_PER_DAY * 7.0;

#[derive(Debug, Clone)]
pub struct ResolvedFlowType {
    pub id: String,
    pub label: String,
    pub color: String,
    pub dashed: bool,
}

#[derive(Debug, Clone)]
pub struct VsmTimelineSegment {
    pub from_label: String,
    pub to_label: String,
    pub edge_label: Option<String>,
    pub edge_type: String,
    pub duration_minutes: f64,
    pub start_offset: f64,
    pub end_offset: f64,
    pub percent_of_total: u32,
    pub source_author: Option<String>,
    pub target_author: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct VsmTimelineStats {
    pub flow_count: usize,
    pub timed_flow_count: usize,
    pub untimed_flow_count: usize,
    pub total_minutes: f64,
    pub coverage_percent: u32,
}

#[derive(Debug, Clone)]
pub struct VsmTimeline {
    pub segments: Vec<VsmTimelineSegment>,
    pub stats: VsmTimelineStats,
}

pub fn map_has_content(map: &ValueStreamMap) -> bool {
    !map.nodes.is_empty() || !map.edges.is_empty()
}

pub fn any_value_streams(value_streams: &HashMap<String, Vec<ValueStreamEntry>>) -> bool {
    value_streams
        .values()
        .any(|entries| entries.iter().any(|entry| map_has_content(&entry.map)))
}

pub fn resolve_flow_types(map: &ValueStreamMap) -> Vec<ResolvedFlowType> {
    let mut merged: HashMap<String, ResolvedFlowType> = HashMap::new();
    for builtin in default_flow_types() {
        merged.insert(builtin.id.clone(), builtin);
    }
    for custom in &map.flow_types {
        merged.insert(
            custom.id.clone(),
            ResolvedFlowType {
                id: custom.id.clone(),
                label: custom.label.clone(),
                color: custom.color.clone(),
                dashed: custom.dash.as_ref().is_some_and(|d| !d.is_empty()),
            },
        );
    }
    let mut types: Vec<_> = merged.into_values().collect();
    types.sort_by(|a, b| a.label.cmp(&b.label));
    types
}

pub fn flow_type_config<'a>(
    id: &str,
    types: &'a [ResolvedFlowType],
) -> &'a ResolvedFlowType {
    types
        .iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| types.first().expect("flow types"))
}

pub fn format_duration(minutes: f64, compact: bool) -> String {
    let total = minutes.round() as i64;
    if total <= 0 {
        return "0".into();
    }
    if total < MINUTES_PER_HOUR as i64 {
        return format!("{total}m");
    }
    if total < MINUTES_PER_DAY as i64 {
        let hours = total / MINUTES_PER_HOUR as i64;
        let mins = total % MINUTES_PER_HOUR as i64;
        if mins == 0 || compact {
            return format!("{hours}h");
        }
        return format!("{hours}h {mins}m");
    }
    if total < MINUTES_PER_WEEK as i64 {
        let days = total / MINUTES_PER_DAY as i64;
        let rem = total % MINUTES_PER_DAY as i64;
        let hours = rem / MINUTES_PER_HOUR as i64;
        let mins = rem % MINUTES_PER_HOUR as i64;
        if hours == 0 && mins == 0 {
            return format!("{days}d");
        }
        if mins == 0 || compact {
            return if hours > 0 {
                format!("{days}d {hours}h")
            } else {
                format!("{days}d")
            };
        }
        if hours == 0 {
            return format!("{days}d {mins}m");
        }
        return format!("{days}d {hours}h");
    }
    let weeks = total / MINUTES_PER_WEEK as i64;
    let rem = total % MINUTES_PER_WEEK as i64;
    let days = rem / MINUTES_PER_DAY as i64;
    let hours = (rem % MINUTES_PER_DAY as i64) / MINUTES_PER_HOUR as i64;
    if days == 0 && hours == 0 {
        return format!("{weeks}w");
    }
    if hours == 0 || compact {
        return if days > 0 {
            format!("{weeks}w {days}d")
        } else {
            format!("{weeks}w")
        };
    }
    if days > 0 {
        format!("{weeks}w {days}d")
    } else {
        format!("{weeks}w {hours}h")
    }
}

pub fn node_accent_color(node_type: &VsmNodeType) -> &'static str {
    match node_type {
        VsmNodeType::Process => "#1e88e5",
        VsmNodeType::Decision => "#f9a825",
        VsmNodeType::Info => "#43a047",
        VsmNodeType::Delay => "#fb8c00",
        VsmNodeType::External => "#78909c",
        VsmNodeType::Customer => "#5c6bc0",
        VsmNodeType::Supplier => "#8d6e63",
        VsmNodeType::Inventory => "#26a69a",
        VsmNodeType::Kaizen => "#e91e63",
    }
}

pub fn node_type_label(node_type: &VsmNodeType) -> &'static str {
    match node_type {
        VsmNodeType::Process => "Process",
        VsmNodeType::Decision => "Decision",
        VsmNodeType::Info => "Info",
        VsmNodeType::Delay => "Delay",
        VsmNodeType::External => "External",
        VsmNodeType::Customer => "Customer",
        VsmNodeType::Supplier => "Supplier",
        VsmNodeType::Inventory => "Inventory",
        VsmNodeType::Kaizen => "Kaizen",
    }
}

pub fn build_timeline(map: &ValueStreamMap) -> VsmTimeline {
    let node_by_id: HashMap<&str, &VsmNode> = map.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut ordered_edges: Vec<&VsmEdge> = map.edges.iter().collect();
    ordered_edges.sort_by(|a, b| {
        let ax = node_by_id.get(a.from.as_str()).map(|n| n.x).unwrap_or(0.0);
        let bx = node_by_id.get(b.from.as_str()).map(|n| n.x).unwrap_or(0.0);
        if ax != bx {
            return ax.partial_cmp(&bx).unwrap_or(std::cmp::Ordering::Equal);
        }
        let atx = node_by_id.get(a.to.as_str()).map(|n| n.x).unwrap_or(0.0);
        let btx = node_by_id.get(b.to.as_str()).map(|n| n.x).unwrap_or(0.0);
        atx.partial_cmp(&btx).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut offset = 0.0;
    let mut segments = Vec::new();
    for edge in ordered_edges {
        let duration = edge.duration_minutes.unwrap_or(0.0);
        let from = node_by_id.get(edge.from.as_str());
        let to = node_by_id.get(edge.to.as_str());
        let segment = VsmTimelineSegment {
            from_label: from.map(|n| n.label.clone()).unwrap_or_else(|| edge.from.clone()),
            to_label: to.map(|n| n.label.clone()).unwrap_or_else(|| edge.to.clone()),
            edge_label: edge.label.clone(),
            edge_type: edge.edge_type.clone(),
            duration_minutes: duration,
            start_offset: offset,
            end_offset: offset + duration,
            percent_of_total: 0,
            source_author: from.and_then(|n| n.author.clone()),
            target_author: to.and_then(|n| n.author.clone()),
        };
        offset += duration;
        segments.push(segment);
    }

    let total_minutes = offset;
    for segment in &mut segments {
        segment.percent_of_total = if total_minutes > 0.0 {
            ((segment.duration_minutes / total_minutes) * 100.0).round() as u32
        } else {
            0
        };
    }

    let timed_flow_count = segments.iter().filter(|s| s.duration_minutes > 0.0).count();
    let flow_count = segments.len();
    let untimed_flow_count = flow_count.saturating_sub(timed_flow_count);
    let coverage_percent = if flow_count > 0 {
        ((timed_flow_count as f64 / flow_count as f64) * 100.0).round() as u32
    } else {
        0
    };

    VsmTimeline {
        segments,
        stats: VsmTimelineStats {
            flow_count,
            timed_flow_count,
            untimed_flow_count,
            total_minutes,
            coverage_percent,
        },
    }
}

pub fn render_vsm_html_section(
    vendor_name: &str,
    map: &ValueStreamMap,
    escape_html: fn(&str) -> String,
) -> String {
    if !map_has_content(map) {
        return String::new();
    }

    let flow_types = resolve_flow_types(map);
    let timeline = build_timeline(map);
    let mut out = String::new();

    out.push_str("  <article class=\"vsm-report-card\">\n");
    out.push_str(&format!(
        "    <h3>{}</h3>\n",
        escape_html(vendor_name)
    ));
    out.push_str("    <div class=\"vsm-report-stats\">\n");
    out.push_str(&format!(
        "      <span><strong>{}</strong> nodes</span>\n",
        map.nodes.len()
    ));
    out.push_str(&format!(
        "      <span><strong>{}</strong> flows</span>\n",
        map.edges.len()
    ));
    if timeline.stats.total_minutes > 0.0 {
        out.push_str(&format!(
            "      <span><strong>{}</strong> total lead time</span>\n",
            escape_html(&format_duration(timeline.stats.total_minutes, false))
        ));
        out.push_str(&format!(
            "      <span><strong>{}%</strong> timed</span>\n",
            timeline.stats.coverage_percent
        ));
    }
    out.push_str("    </div>\n");

    if !map.nodes.is_empty() {
        out.push_str("    <div class=\"vsm-report-diagram\">\n");
        out.push_str(&render_svg_diagram(map, &flow_types, escape_html));
        out.push_str("    </div>\n");
    }

    out.push_str("    <div class=\"vsm-report-legend\">\n");
    out.push_str("      <h4>Flow types</h4>\n      <ul>\n");
    for ft in &flow_types {
        let dash = if ft.dashed { " dashed" } else { "" };
        out.push_str(&format!(
            "        <li><span class=\"vsm-legend-line{dash}\" style=\"border-color:{}\"></span>{}</li>\n",
            escape_html(&ft.color),
            escape_html(&ft.label)
        ));
    }
    out.push_str("      </ul>\n    </div>\n");

    if !timeline.segments.is_empty() {
        out.push_str("    <h4>Process timeline</h4>\n");
        out.push_str("    <table class=\"data-table compact vsm-timeline-table\">\n");
        out.push_str(
            "      <thead><tr><th>From</th><th>To</th><th>Type</th><th>Duration</th><th>Author</th></tr></thead>\n",
        );
        out.push_str("      <tbody>\n");
        for segment in &timeline.segments {
            let ft = flow_type_config(&segment.edge_type, &flow_types);
            let author = segment
                .target_author
                .as_deref()
                .or(segment.source_author.as_deref())
                .unwrap_or("—");
            let duration = if segment.duration_minutes > 0.0 {
                format_duration(segment.duration_minutes, false)
            } else {
                "—".into()
            };
            out.push_str(&format!(
                "        <tr><td>{}</td><td>{}</td><td><span class=\"vsm-flow-badge\" style=\"background:{}20;color:{}\">{}</span></td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&segment.from_label),
                escape_html(&segment.to_label),
                escape_html(&ft.color),
                escape_html(&ft.color),
                escape_html(&ft.label),
                escape_html(&duration),
                escape_html(author),
            ));
        }
        out.push_str("      </tbody>\n    </table>\n");

        if timeline.stats.total_minutes > 0.0 {
            out.push_str("    <div class=\"vsm-gantt\">\n");
            for segment in &timeline.segments {
                if segment.duration_minutes <= 0.0 {
                    continue;
                }
                let width_pct = segment.percent_of_total.max(1);
                let ft = flow_type_config(&segment.edge_type, &flow_types);
                out.push_str(&format!(
                    "      <div class=\"vsm-gantt-row\"><span class=\"vsm-gantt-label\">{} → {}</span><div class=\"vsm-gantt-track\"><div class=\"vsm-gantt-bar\" style=\"width:{width_pct}%;background:{}\" title=\"{}\"></div></div><span class=\"vsm-gantt-dur\">{}</span></div>\n",
                    escape_html(&segment.from_label),
                    escape_html(&segment.to_label),
                    escape_html(&ft.color),
                    escape_html(&format!(
                        "{} — {}",
                        ft.label,
                        format_duration(segment.duration_minutes, false)
                    )),
                    escape_html(&format_duration(segment.duration_minutes, true)),
                ));
            }
            out.push_str("    </div>\n");
        }
    }

    if !map.nodes.is_empty() {
        out.push_str("    <h4>Process steps</h4>\n");
        out.push_str("    <table class=\"data-table compact\">\n");
        out.push_str(
            "      <thead><tr><th>Step</th><th>Type</th><th>Author</th><th>Lead</th><th>Cycle</th></tr></thead>\n",
        );
        out.push_str("      <tbody>\n");
        let mut nodes = map.nodes.clone();
        nodes.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for node in &nodes {
            let lead = node
                .lead_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| format_duration(v, true))
                .unwrap_or_else(|| "—".into());
            let cycle = node
                .cycle_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| format_duration(v, true))
                .unwrap_or_else(|| "—".into());
            out.push_str(&format!(
                "        <tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&node.label),
                node_type_label(&node.node_type),
                escape_html(node.author.as_deref().unwrap_or("—")),
                escape_html(&lead),
                escape_html(&cycle),
            ));
        }
        out.push_str("      </tbody>\n    </table>\n");
    }

    if !map.messages.is_empty() {
        out.push_str("    <h4>Messages &amp; notes</h4>\n    <ul class=\"vsm-messages\">\n");
        for msg in &map.messages {
            out.push_str(&format!(
                "      <li>{}</li>\n",
                escape_html(msg.text.trim())
            ));
        }
        out.push_str("    </ul>\n");
    }

    out.push_str("  </article>\n");
    out
}

pub fn render_vsm_markdown_section(vendor_name: &str, map: &ValueStreamMap) -> String {
    if !map_has_content(map) {
        return String::new();
    }

    let flow_types = resolve_flow_types(map);
    let timeline = build_timeline(map);
    let mut out = String::new();

    out.push_str(&format!("### {vendor_name}\n\n"));
    out.push_str(&format!(
        "- **Nodes:** {} · **Flows:** {}",
        map.nodes.len(),
        map.edges.len()
    ));
    if timeline.stats.total_minutes > 0.0 {
        out.push_str(&format!(
            " · **Total lead time:** {} · **Timed:** {}%",
            format_duration(timeline.stats.total_minutes, false),
            timeline.stats.coverage_percent
        ));
    }
    out.push('\n');

    if !timeline.segments.is_empty() {
        out.push_str("\n| From | To | Flow type | Duration | Author |\n");
        out.push_str("|------|-----|-----------|----------|--------|\n");
        for segment in &timeline.segments {
            let ft = flow_type_config(&segment.edge_type, &flow_types);
            let author = segment
                .target_author
                .as_deref()
                .or(segment.source_author.as_deref())
                .unwrap_or("—");
            let duration = if segment.duration_minutes > 0.0 {
                format_duration(segment.duration_minutes, false)
            } else {
                "—".into()
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                segment.from_label, segment.to_label, ft.label, duration, author
            ));
        }
    }

    if !map.messages.is_empty() {
        out.push_str("\n**Messages:**\n\n");
        for msg in &map.messages {
            out.push_str(&format!("- {}\n", msg.text.trim()));
        }
    }

    out.push('\n');
    out
}

fn render_svg_diagram(
    map: &ValueStreamMap,
    flow_types: &[ResolvedFlowType],
    escape_html: fn(&str) -> String,
) -> String {
    if map.nodes.is_empty() {
        return String::new();
    }

    let padding = 24.0;
    let min_x = map.nodes.iter().map(|n| n.x).fold(f64::INFINITY, f64::min);
    let min_y = map.nodes.iter().map(|n| n.y).fold(f64::INFINITY, f64::min);
    let max_x = map
        .nodes
        .iter()
        .map(|n| n.x + n.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = map
        .nodes
        .iter()
        .map(|n| n.y + n.height)
        .fold(f64::NEG_INFINITY, f64::max);

    let content_w = (max_x - min_x).max(1.0);
    let content_h = (max_y - min_y).max(1.0);
    let view_w = content_w + padding * 2.0;
    let view_h = content_h + padding * 2.0;

    let node_by_id: HashMap<&str, &VsmNode> = map.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg class=\"vsm-svg\" viewBox=\"0 0 {view_w:.1} {view_h:.1}\" xmlns=\"http://www.w3.org/2000/svg\" role=\"img\" aria-label=\"Value stream diagram\">\n"
    ));

    for edge in &map.edges {
        let Some(from) = node_by_id.get(edge.from.as_str()) else {
            continue;
        };
        let Some(to) = node_by_id.get(edge.to.as_str()) else {
            continue;
        };
        let ft = flow_type_config(&edge.edge_type, flow_types);
        let x1 = from.x + from.width - min_x + padding;
        let y1 = from.y + from.height / 2.0 - min_y + padding;
        let x2 = to.x - min_x + padding;
        let y2 = to.y + to.height / 2.0 - min_y + padding;
        let dash = if ft.dashed {
            " stroke-dasharray=\"6 4\""
        } else {
            ""
        };
        svg.push_str(&format!(
            "  <line x1=\"{x1:.1}\" y1=\"{y1:.1}\" x2=\"{x2:.1}\" y2=\"{y2:.1}\" stroke=\"{}\" stroke-width=\"2\"{dash} />\n",
            escape_html(&ft.color)
        ));
        if let Some(label) = &edge.label {
            if !label.trim().is_empty() {
                let mx = (x1 + x2) / 2.0;
                let my = (y1 + y2) / 2.0 - 4.0;
                svg.push_str(&format!(
                    "  <text x=\"{mx:.1}\" y=\"{my:.1}\" class=\"vsm-edge-label\" text-anchor=\"middle\">{}</text>\n",
                    escape_html(label.trim())
                ));
            }
        }
        if let Some(mins) = edge.duration_minutes.filter(|v| *v > 0.0) {
            let mx = (x1 + x2) / 2.0;
            let my = (y1 + y2) / 2.0 + 10.0;
            svg.push_str(&format!(
                "  <text x=\"{mx:.1}\" y=\"{my:.1}\" class=\"vsm-edge-duration\" text-anchor=\"middle\">{}</text>\n",
                escape_html(&format_duration(mins, true))
            ));
        }
    }

    for node in &map.nodes {
        let x = node.x - min_x + padding;
        let y = node.y - min_y + padding;
        let color = node_accent_color(&node.node_type);
        let rx = if matches!(node.node_type, VsmNodeType::Decision | VsmNodeType::Kaizen) {
            10.0
        } else {
            6.0
        };
        svg.push_str(&format!(
            "  <rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{rx}\" fill=\"{color}\" fill-opacity=\"0.12\" stroke=\"{color}\" stroke-width=\"1.5\" />\n",
            node.width, node.height
        ));
        let label = truncate_label(&node.label, 22);
        svg.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"vsm-node-label\" text-anchor=\"middle\">{}</text>\n",
            x + node.width / 2.0,
            y + node.height / 2.0 + 4.0,
            escape_html(&label)
        ));
        if let Some(author) = node.author.as_deref().filter(|a| !a.is_empty()) {
            svg.push_str(&format!(
                "  <text x=\"{:.1}\" y=\"{:.1}\" class=\"vsm-node-author\" text-anchor=\"middle\">{}</text>\n",
                x + node.width / 2.0,
                y + node.height - 6.0,
                escape_html(&truncate_label(author, 18))
            ));
        }
    }

    svg.push_str("</svg>\n");
    svg
}

/// Parse `#rrggbb` hex color to 0–1 RGB components for PDF rendering.
pub fn hex_to_rgb(hex: &str) -> Option<(f32, f32, f32)> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0;
    Some((r, g, b))
}

fn truncate_label(label: &str, max: usize) -> String {
    let trimmed = label.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let cut: String = trimmed.chars().take(max.saturating_sub(1)).collect();
    format!("{cut}…")
}

fn default_flow_types() -> Vec<ResolvedFlowType> {
    vec![
        ResolvedFlowType {
            id: "material".into(),
            label: "Material flow".into(),
            color: "#37474f".into(),
            dashed: false,
        },
        ResolvedFlowType {
            id: "information".into(),
            label: "Information flow".into(),
            color: "#1e88e5".into(),
            dashed: true,
        },
        ResolvedFlowType {
            id: "electronic".into(),
            label: "Electronic flow".into(),
            color: "#8e24aa".into(),
            dashed: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_stream::{VsmEdge, VsmNode};

    #[test]
    fn format_duration_weeks() {
        assert_eq!(format_duration(10_080.0, false), "1w");
        assert_eq!(format_duration(11_520.0, false), "1w 1d");
        assert_eq!(format_duration(90.0, false), "1h 30m");
    }

    #[test]
    fn build_timeline_orders_by_x() {
        let map = ValueStreamMap {
            nodes: vec![
                VsmNode {
                    id: "a".into(),
                    label: "A".into(),
                    node_type: VsmNodeType::Process,
                    x: 0.0,
                    y: 0.0,
                    width: 180.0,
                    height: 72.0,
                    notes: None,
                    role: None,
                    lead_time_minutes: None,
                    cycle_time_minutes: None,
                    author: None,
                },
                VsmNode {
                    id: "b".into(),
                    label: "B".into(),
                    node_type: VsmNodeType::Process,
                    x: 200.0,
                    y: 0.0,
                    width: 180.0,
                    height: 72.0,
                    notes: None,
                    role: None,
                    lead_time_minutes: None,
                    cycle_time_minutes: None,
                    author: None,
                },
            ],
            edges: vec![VsmEdge {
                id: "e1".into(),
                from: "a".into(),
                to: "b".into(),
                label: None,
                edge_type: "material".into(),
                duration_minutes: Some(120.0),
            }],
            messages: vec![],
            flow_types: vec![],
        };
        let timeline = build_timeline(&map);
        assert_eq!(timeline.stats.total_minutes, 120.0);
        assert_eq!(timeline.segments[0].from_label, "A");
    }
}
