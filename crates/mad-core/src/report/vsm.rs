use std::collections::{HashMap, HashSet};

use crate::report::locale::{self, ReportLocale, ReportStrings};
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
    pub edge_id: String,
    pub from_id: String,
    pub to_id: String,
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

#[derive(Debug, Clone)]
pub struct VsmTimelineMilestone {
    pub node_id: String,
    pub label: String,
    pub offset_minutes: f64,
    pub node_type: String,
    pub author: Option<String>,
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
    pub milestones: Vec<VsmTimelineMilestone>,
    pub ruler_ticks: Vec<f64>,
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
    format_duration_labeled(minutes, compact, EN_DURATION_LABELS)
}

#[derive(Clone, Copy)]
pub struct DurationLabels {
    pub minute: &'static str,
    pub hour: &'static str,
    pub day: &'static str,
    pub week: &'static str,
    pub sep: &'static str,
}

const EN_DURATION_LABELS: DurationLabels = DurationLabels {
    minute: "m",
    hour: "h",
    day: "d",
    week: "w",
    sep: "",
};

pub fn duration_labels(s: &ReportStrings) -> DurationLabels {
    DurationLabels {
        minute: s.duration_short_minute,
        hour: s.duration_short_hour,
        day: s.duration_short_day,
        week: s.duration_short_week,
        sep: s.duration_short_sep,
    }
}

fn duration_part(value: i64, unit: &str, sep: &str) -> String {
    if sep.is_empty() {
        format!("{value}{unit}")
    } else {
        format!("{value}{sep}{unit}")
    }
}

pub fn format_duration_labeled(minutes: f64, compact: bool, labels: DurationLabels) -> String {
    let total = minutes.round() as i64;
    if total <= 0 {
        return "0".into();
    }
    let sep = labels.sep;
    if total < MINUTES_PER_HOUR as i64 {
        return duration_part(total, labels.minute, sep);
    }
    if total < MINUTES_PER_DAY as i64 {
        let hours = total / MINUTES_PER_HOUR as i64;
        let mins = total % MINUTES_PER_HOUR as i64;
        if mins == 0 || compact {
            return duration_part(hours, labels.hour, sep);
        }
        return format!(
            "{} {}",
            duration_part(hours, labels.hour, sep),
            duration_part(mins, labels.minute, sep)
        );
    }
    if total < MINUTES_PER_WEEK as i64 {
        let days = total / MINUTES_PER_DAY as i64;
        let rem = total % MINUTES_PER_DAY as i64;
        let hours = rem / MINUTES_PER_HOUR as i64;
        let mins = rem % MINUTES_PER_HOUR as i64;
        if hours == 0 && mins == 0 {
            return duration_part(days, labels.day, sep);
        }
        if mins == 0 || compact {
            return if hours > 0 {
                format!(
                    "{} {}",
                    duration_part(days, labels.day, sep),
                    duration_part(hours, labels.hour, sep)
                )
            } else {
                duration_part(days, labels.day, sep)
            };
        }
        if hours == 0 {
            return format!(
                "{} {}",
                duration_part(days, labels.day, sep),
                duration_part(mins, labels.minute, sep)
            );
        }
        return format!(
            "{} {}",
            duration_part(days, labels.day, sep),
            duration_part(hours, labels.hour, sep)
        );
    }
    let weeks = total / MINUTES_PER_WEEK as i64;
    let rem = total % MINUTES_PER_WEEK as i64;
    let days = rem / MINUTES_PER_DAY as i64;
    let hours = (rem % MINUTES_PER_DAY as i64) / MINUTES_PER_HOUR as i64;
    if days == 0 && hours == 0 {
        return duration_part(weeks, labels.week, sep);
    }
    if hours == 0 || compact {
        return if days > 0 {
            format!(
                "{} {}",
                duration_part(weeks, labels.week, sep),
                duration_part(days, labels.day, sep)
            )
        } else {
            duration_part(weeks, labels.week, sep)
        };
    }
    if days > 0 {
        format!(
            "{} {}",
            duration_part(weeks, labels.week, sep),
            duration_part(days, labels.day, sep)
        )
    } else {
        format!(
            "{} {}",
            duration_part(weeks, labels.week, sep),
            duration_part(hours, labels.hour, sep)
        )
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

fn node_accent_from_type_label(label: &str) -> &'static str {
    match label {
        "Process" => "#1e88e5",
        "Decision" => "#f9a825",
        "Info" => "#43a047",
        "Delay" => "#fb8c00",
        "External" => "#78909c",
        "Customer" => "#5c6bc0",
        "Supplier" => "#8d6e63",
        "Inventory" => "#26a69a",
        "Kaizen" => "#e91e63",
        _ => "#78909c",
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
            edge_id: edge.id.clone(),
            from_id: edge.from.clone(),
            to_id: edge.to.clone(),
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

    let milestones = build_milestones(&segments, &node_by_id);
    let ruler_ticks = build_ruler_ticks(total_minutes);

    VsmTimeline {
        segments,
        milestones,
        ruler_ticks,
        stats: VsmTimelineStats {
            flow_count,
            timed_flow_count,
            untimed_flow_count,
            total_minutes,
            coverage_percent,
        },
    }
}

fn build_milestones(
    segments: &[VsmTimelineSegment],
    node_by_id: &HashMap<&str, &VsmNode>,
) -> Vec<VsmTimelineMilestone> {
    let mut milestones = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let mut push = |node_id: &str, offset_minutes: f64, milestones: &mut Vec<VsmTimelineMilestone>| {
        if !seen.insert(node_id.to_string()) {
            return;
        }
        let node = node_by_id.get(node_id);
        milestones.push(VsmTimelineMilestone {
            node_id: node_id.to_string(),
            label: node
                .map(|n| n.label.clone())
                .unwrap_or_else(|| node_id.to_string()),
            offset_minutes,
            node_type: node
                .map(|n| node_type_label(&n.node_type).to_string())
                .unwrap_or_else(|| "Process".into()),
            author: node.and_then(|n| n.author.clone()),
        });
    };

    if let Some(first) = segments.first() {
        push(&first.from_id, 0.0, &mut milestones);
    }
    for segment in segments {
        push(&segment.to_id, segment.end_offset, &mut milestones);
    }
    milestones
}

fn build_ruler_ticks(total_minutes: f64) -> Vec<f64> {
    if total_minutes <= 0.0 {
        return vec![0.0];
    }
    let divisions = 4.0;
    let raw = total_minutes / divisions;
    let step = pick_ruler_step(raw, total_minutes);
    let mut ticks = vec![0.0];
    let mut value = step;
    while value < total_minutes {
        ticks.push(value);
        value += step;
    }
    if ticks.last().copied().unwrap_or(0.0) < total_minutes {
        ticks.push(total_minutes);
    }
    ticks
}

fn pick_ruler_step(raw: f64, total_minutes: f64) -> f64 {
    let candidates: &[f64] = if total_minutes >= MINUTES_PER_WEEK {
        &[MINUTES_PER_WEEK, MINUTES_PER_DAY, MINUTES_PER_HOUR * 12.0, MINUTES_PER_HOUR]
    } else if total_minutes >= MINUTES_PER_DAY {
        &[
            MINUTES_PER_DAY,
            MINUTES_PER_HOUR * 6.0,
            MINUTES_PER_HOUR * 3.0,
            MINUTES_PER_HOUR,
        ]
    } else if total_minutes >= MINUTES_PER_HOUR {
        &[MINUTES_PER_HOUR, 30.0, 15.0, 5.0]
    } else {
        &[15.0, 10.0, 5.0, 1.0]
    };

    for &candidate in candidates {
        if raw <= candidate * 1.5 {
            return candidate;
        }
    }
    raw.max(1.0).round()
}

fn edge_type_short(edge_type: &str) -> char {
    match edge_type {
        "information" => 'I',
        "electronic" => 'E',
        "material" => 'M',
        _ => edge_type
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or('?'),
    }
}

fn timeline_offset_percent(offset: f64, total_minutes: f64) -> f64 {
    if total_minutes > 0.0 {
        (offset / total_minutes) * 100.0
    } else {
        0.0
    }
}

fn segment_flex_weight(segment: &VsmTimelineSegment, total_minutes: f64, segment_count: usize) -> u32 {
    if total_minutes > 0.0 && segment.duration_minutes > 0.0 {
        ((segment.duration_minutes / total_minutes) * 1000.0).round().max(1.0) as u32
    } else if segment_count > 0 {
        (1000.0 / segment_count as f64).round().max(1.0) as u32
    } else {
        1
    }
}

fn render_timeline_chart_html(
    timeline: &VsmTimeline,
    flow_types: &[ResolvedFlowType],
    escape_html: fn(&str) -> String,
    interactive: bool,
    s: &ReportStrings,
) -> String {
    if timeline.segments.is_empty() {
        return String::new();
    }

    let dl = duration_labels(s);
    let total = timeline.stats.total_minutes;
    let longest_edge = timeline
        .segments
        .iter()
        .max_by(|a, b| {
            a.duration_minutes
                .partial_cmp(&b.duration_minutes)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|s| s.edge_id.as_str());

    let mut out = String::from("    <div class=\"mad-vsm-timeline-chart\">\n");

    out.push_str("      <div class=\"mad-vsm-timeline-stats\">\n");
    out.push_str("        <div class=\"mad-vsm-timeline-stat mad-vsm-timeline-stat-primary\">\n");
    out.push_str("          <span class=\"mad-vsm-timeline-stat-label\">");
    out.push_str(s.vsm_timeline_total);
    out.push_str("</span>\n");
    out.push_str(&format!(
        "          <strong>{}</strong>\n",
        if total > 0.0 {
            escape_html(&format_duration_labeled(total, false, dl))
        } else {
            "—".into()
        }
    ));
    out.push_str("        </div>\n");
    out.push_str("        <div class=\"mad-vsm-timeline-stat\">\n");
    out.push_str("          <span class=\"mad-vsm-timeline-stat-label\">");
    out.push_str(s.vsm_timeline_timed);
    out.push_str("</span>\n");
    out.push_str(&format!(
        "          <strong>{}/{}</strong>\n",
        timeline.stats.timed_flow_count, timeline.stats.flow_count
    ));
    out.push_str("        </div>\n");
    out.push_str("        <div class=\"mad-vsm-timeline-stat\">\n");
    out.push_str("          <span class=\"mad-vsm-timeline-stat-label\">");
    out.push_str(s.vsm_timeline_coverage);
    out.push_str("</span>\n");
    out.push_str(&format!(
        "          <strong>{}%</strong>\n",
        timeline.stats.coverage_percent
    ));
    out.push_str("        </div>\n");
    out.push_str("      </div>\n");

    let bar_tag = if interactive { "button" } else { "div" };
    let milestone_tag = if interactive { "button" } else { "div" };
    out.push_str(&format!(
        "      <div class=\"mad-vsm-timeline-track\" role=\"list\" aria-label=\"{}\">\n",
        escape_html(s.vsm_timeline_aria)
    ));

    let seg_count = timeline.segments.len();
    for segment in &timeline.segments {
        let ft = flow_type_config(&segment.edge_type, flow_types);
        let has_duration = segment.duration_minutes > 0.0;
        let flex = segment_flex_weight(segment, total, seg_count);
        let is_longest = longest_edge == Some(segment.edge_id.as_str()) && has_duration;
        let title = if let Some(label) = &segment.edge_label {
            format!(
                "{} → {} ({})",
                segment.from_label, segment.to_label, label
            )
        } else {
            format!("{} → {}", segment.from_label, segment.to_label)
        };
        let duration_label = if has_duration {
            format_duration_labeled(segment.duration_minutes, true, dl)
        } else {
            "—".into()
        };
        let mut classes = String::from("mad-vsm-timeline-bar");
        if !has_duration {
            classes.push_str(" untimed");
        }
        if is_longest {
            classes.push_str(" longest");
        }

        let type_attr = if interactive {
            " type=\"button\""
        } else {
            ""
        };
        let data_attrs = if interactive {
            format!(
                " data-edge-id=\"{}\" data-from-node=\"{}\" data-to-node=\"{}\"",
                escape_html(&segment.edge_id),
                escape_html(&segment.from_id),
                escape_html(&segment.to_id),
            )
        } else {
            String::new()
        };

        out.push_str(&format!(
            "        <{bar_tag}{type_attr} class=\"{classes}\" style=\"flex-grow:{flex};background:{}\" title=\"{}\"{data_attrs}>\n",
            escape_html(&ft.color),
            escape_html(&title),
        ));
        out.push_str(&format!(
            "          <span class=\"mad-vsm-timeline-bar-type\">{}</span>\n",
            edge_type_short(&segment.edge_type)
        ));
        out.push_str("          <span class=\"mad-vsm-timeline-bar-label\">");
        out.push_str(&escape_html(&duration_label));
        if has_duration && segment.percent_of_total > 0 {
            out.push_str(&format!(
                "<span class=\"mad-vsm-timeline-bar-pct\">{}%</span>",
                segment.percent_of_total
            ));
        }
        out.push_str(&format!("</span>\n        </{bar_tag}>\n"));
    }
    out.push_str("      </div>\n");

    if total > 0.0 && timeline.ruler_ticks.len() > 1 {
        out.push_str("      <div class=\"mad-vsm-timeline-ruler\">\n");
        for tick in &timeline.ruler_ticks {
            let left = timeline_offset_percent(*tick, total);
            out.push_str(&format!(
                "        <span class=\"mad-vsm-timeline-ruler-tick\" style=\"left:{left:.2}%\">{}</span>\n",
                escape_html(&format_duration_labeled(*tick, true, dl))
            ));
        }
        out.push_str("      </div>\n");
    }

    if !timeline.milestones.is_empty() {
        out.push_str("      <div class=\"mad-vsm-timeline-milestones\">\n");
        out.push_str("        <span class=\"mad-vsm-timeline-milestones-title\">");
        out.push_str(s.vsm_milestones);
        out.push_str("</span>\n");
        out.push_str("        <div class=\"mad-vsm-timeline-milestone-lane\">\n");
        for milestone in &timeline.milestones {
            let left = timeline_offset_percent(milestone.offset_minutes, total.max(1.0));
            let color = node_accent_from_type_label(&milestone.node_type);
            let mut m_attrs = format!(
                " class=\"mad-vsm-timeline-milestone\" style=\"left:{left:.2}%;--milestone-color:{color}\" title=\"{}\"",
                escape_html(&milestone.label),
            );
            if interactive {
                m_attrs.push_str(&format!(
                    " data-node-id=\"{}\"",
                    escape_html(&milestone.node_id)
                ));
            }
            let m_type = if interactive {
                " type=\"button\""
            } else {
                ""
            };
            out.push_str(&format!(
                "          <{milestone_tag}{m_type}{m_attrs}><span class=\"mad-vsm-timeline-milestone-dot\"></span><span class=\"mad-vsm-timeline-milestone-label\">{}</span></{milestone_tag}>\n",
                escape_html(&truncate_label(&milestone.label, 16)),
            ));
        }
        out.push_str("        </div>\n");
        out.push_str("      </div>\n");
    }

    out.push_str("    </div>\n");
    out
}

#[derive(Debug, Clone, Default)]
pub struct VsmHtmlOptions {
    pub interactive: bool,
    pub vendor_id: Option<String>,
    pub locale: ReportLocale,
}

pub fn render_vsm_html_section(
    vendor_name: &str,
    map: &ValueStreamMap,
    escape_html: fn(&str) -> String,
    options: &VsmHtmlOptions,
) -> String {
    if !map_has_content(map) {
        return String::new();
    }

    let flow_types = resolve_flow_types(map);
    let timeline = build_timeline(map);
    let s = locale::strings(options.locale);
    let dl = duration_labels(s);
    let mut out = String::new();

    let vendor_attr = options
        .vendor_id
        .as_deref()
        .map(|id| format!(" data-vendor-vsm=\"{}\"", escape_html(id)))
        .unwrap_or_default();
    let viewer_class = if options.interactive {
        "vsm-report-card mad-vsm-viewer"
    } else {
        "vsm-report-card"
    };
    out.push_str(&format!("  <article class=\"{viewer_class}\"{vendor_attr}>\n"));
    out.push_str(&format!(
        "    <h3>{}</h3>\n",
        escape_html(vendor_name)
    ));
    out.push_str("    <div class=\"vsm-report-stats\">\n");
    out.push_str(&format!(
        "      <span><strong>{}</strong> {}</span>\n",
        map.nodes.len(),
        s.vsm_nodes
    ));
    out.push_str(&format!(
        "      <span><strong>{}</strong> {}</span>\n",
        map.edges.len(),
        s.vsm_flows
    ));
    if timeline.stats.total_minutes > 0.0 {
        out.push_str(&format!(
            "      <span><strong>{}</strong> {}</span>\n",
            escape_html(&format_duration_labeled(timeline.stats.total_minutes, false, dl)),
            s.vsm_total_lead
        ));
        out.push_str(&format!(
            "      <span><strong>{}%</strong> {}</span>\n",
            timeline.stats.coverage_percent,
            s.vsm_timed_pct
        ));
    }
    out.push_str("    </div>\n");

    if options.interactive && !map.nodes.is_empty() {
        out.push_str("    <div class=\"mad-vsm-toolbar\">\n");
        out.push_str("      <div class=\"mad-vsm-tabs\">\n");
        out.push_str(&format!("        <button type=\"button\" data-vsm-tab=\"diagram\" class=\"active\">{}</button>\n", s.vsm_tab_diagram));
        out.push_str(&format!("        <button type=\"button\" data-vsm-tab=\"timeline\">{}</button>\n", s.vsm_tab_timeline));
        out.push_str(&format!("        <button type=\"button\" data-vsm-tab=\"steps\">{}</button>\n", s.vsm_tab_steps));
        out.push_str("      </div>\n");
        out.push_str("      <div class=\"mad-vsm-zoom\">\n");
        out.push_str(&format!("        <button type=\"button\" data-vsm-action=\"zoom-out\" title=\"{}\">−</button>\n", escape_html(s.vsm_zoom_out)));
        out.push_str(&format!("        <button type=\"button\" data-vsm-action=\"reset\" title=\"{}\">⟲</button>\n", escape_html(s.vsm_zoom_reset)));
        out.push_str(&format!("        <button type=\"button\" data-vsm-action=\"zoom-in\" title=\"{}\">+</button>\n", escape_html(s.vsm_zoom_in)));
        out.push_str("      </div>\n");
        out.push_str("    </div>\n");
        out.push_str("    <div class=\"mad-vsm-stage\" data-vsm-panel=\"diagram\">\n");
        out.push_str("    <div class=\"vsm-report-diagram\">\n");
        out.push_str(&render_svg_diagram(map, &flow_types, escape_html, true, dl));
        out.push_str("    </div>\n");
        out.push_str("    <aside class=\"mad-vsm-inspector mad-hidden\" aria-live=\"polite\"></aside>\n");
        out.push_str("    </div>\n");
        out.push_str(&vsm_map_payload(map, options.locale));
    } else if !map.nodes.is_empty() {
        out.push_str("    <div class=\"vsm-report-diagram\">\n");
        out.push_str(&render_svg_diagram(map, &flow_types, escape_html, false, dl));
        out.push_str("    </div>\n");
    }

    let timeline_panel_attr = if options.interactive {
        " data-vsm-panel=\"timeline\" class=\"mad-hidden\""
    } else {
        ""
    };
    let steps_panel_attr = if options.interactive {
        " data-vsm-panel=\"steps\" class=\"mad-hidden\""
    } else {
        ""
    };

    out.push_str("    <div class=\"vsm-report-legend\">\n");
    out.push_str(&format!("      <h4>{}</h4>\n      <ul>\n", s.vsm_flow_types));
    for ft in &flow_types {
        let dash = if ft.dashed { " dashed" } else { "" };
        let label = locale::flow_type_label(&ft.id, &ft.label, options.locale);
        out.push_str(&format!(
            "        <li><span class=\"vsm-legend-line{dash}\" style=\"border-color:{}\"></span>{}</li>\n",
            escape_html(&ft.color),
            escape_html(&label)
        ));
    }
    out.push_str("      </ul>\n    </div>\n");

    if !timeline.segments.is_empty() {
        out.push_str(&format!("    <div{timeline_panel_attr}>\n"));
        out.push_str(&format!("    <h4>{}</h4>\n", s.vsm_process_timeline));
        out.push_str(&render_timeline_chart_html(
            &timeline,
            &flow_types,
            escape_html,
            options.interactive,
            s,
        ));
        out.push_str("    <details class=\"mad-vsm-timeline-details\" open>\n");
        out.push_str(&format!("      <summary>{}</summary>\n", s.vsm_flow_details));
        out.push_str("    <table class=\"data-table compact vsm-timeline-table\">\n");
        out.push_str(&format!(
            "      <thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead>\n",
            s.vsm_col_from, s.vsm_col_to, s.vsm_col_type, s.vsm_col_duration, s.vsm_col_author
        ));
        out.push_str("      <tbody>\n");
        for segment in &timeline.segments {
            let ft = flow_type_config(&segment.edge_type, &flow_types);
            let author = segment
                .target_author
                .as_deref()
                .or(segment.source_author.as_deref())
                .unwrap_or("—");
            let duration = if segment.duration_minutes > 0.0 {
                format_duration_labeled(segment.duration_minutes, false, dl)
            } else {
                "—".into()
            };
            let row_attrs = if options.interactive {
                format!(
                    " class=\"mad-vsm-timeline-row\" data-edge-id=\"{}\" data-from-node=\"{}\" data-to-node=\"{}\" tabindex=\"0\" role=\"button\"",
                    escape_html(&segment.edge_id),
                    escape_html(&segment.from_id),
                    escape_html(&segment.to_id),
                )
            } else {
                String::new()
            };
            out.push_str(&format!(
                "        <tr{row_attrs}><td>{}</td><td>{}</td><td><span class=\"vsm-flow-badge\" style=\"background:{}20;color:{}\">{}</span></td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&segment.from_label),
                escape_html(&segment.to_label),
                escape_html(&ft.color),
                escape_html(&ft.color),
                escape_html(&locale::flow_type_label(&segment.edge_type, &ft.label, options.locale)),
                escape_html(&duration),
                escape_html(author),
            ));
        }
        out.push_str("      </tbody>\n    </table>\n");
        out.push_str("    </details>\n");
        out.push_str("    </div>\n");
    }

    if !map.nodes.is_empty() {
        out.push_str(&format!("    <div{steps_panel_attr}>\n"));
        out.push_str(&format!("    <h4>{}</h4>\n", s.vsm_process_steps));
        out.push_str("    <table class=\"data-table compact\">\n");
        out.push_str(&format!(
            "      <thead><tr><th>{}</th><th>{}</th><th>{}</th><th>{}</th><th>{}</th></tr></thead>\n",
            s.vsm_col_step, s.vsm_col_type, s.vsm_col_author, s.vsm_col_lead, s.vsm_col_cycle
        ));
        out.push_str("      <tbody>\n");
        let mut nodes = map.nodes.clone();
        nodes.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for node in &nodes {
            let lead = node
                .lead_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| format_duration_labeled(v, true, dl))
                .unwrap_or_else(|| "—".into());
            let cycle = node
                .cycle_time_minutes
                .filter(|v| *v > 0.0)
                .map(|v| format_duration_labeled(v, true, dl))
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
        out.push_str("    </div>\n");
    }

    if !map.messages.is_empty() {
        out.push_str(&format!("    <h4>{}</h4>\n    <ul class=\"vsm-messages\">\n", s.vsm_messages));
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

fn vsm_map_payload(map: &ValueStreamMap, locale: ReportLocale) -> String {
    let timeline = build_timeline(map);
    let flow_types = resolve_flow_types(map);
    let nodes: Vec<_> = map
        .nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "label": n.label,
                "node_type": node_type_label(&n.node_type).to_lowercase(),
                "author": n.author,
                "role": n.role,
                "lead_time_minutes": n.lead_time_minutes.unwrap_or(0.0),
                "cycle_time_minutes": n.cycle_time_minutes.unwrap_or(0.0),
                "notes": n.notes,
            })
        })
        .collect();
    let segments: Vec<_> = timeline
        .segments
        .iter()
        .map(|s| {
            let ft = flow_type_config(&s.edge_type, &flow_types);
            let ft_label = locale::flow_type_label(&s.edge_type, &ft.label, locale);
            serde_json::json!({
                "edge_id": s.edge_id,
                "from_id": s.from_id,
                "to_id": s.to_id,
                "from_label": s.from_label,
                "to_label": s.to_label,
                "edge_label": s.edge_label,
                "edge_type": s.edge_type,
                "flow_type_label": ft_label,
                "flow_type_color": ft.color,
                "duration_minutes": s.duration_minutes,
            })
        })
        .collect();
    let json_str = serde_json::to_string(&serde_json::json!({
        "nodes": nodes,
        "segments": segments,
    }))
        .unwrap_or_else(|_| "{}".into());
    format!(
        "    <script type=\"application/json\" class=\"mad-vsm-payload\">{json_str}</script>\n"
    )
}

fn render_svg_diagram(
    map: &ValueStreamMap,
    flow_types: &[ResolvedFlowType],
    escape_html: fn(&str) -> String,
    interactive: bool,
    dl: DurationLabels,
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
    if interactive {
        svg.push_str("  <g class=\"mad-vsm-world\">\n");
    }

    let decision_handles = resolve_decision_edge_handles(map);

    for edge in &map.edges {
        let Some(from) = node_by_id.get(edge.from.as_str()) else {
            continue;
        };
        let Some(to) = node_by_id.get(edge.to.as_str()) else {
            continue;
        };
        let ft = flow_type_config(&edge.edge_type, flow_types);
        let handle = decision_handles.get(&edge.id).map(|s| s.as_str());
        let (sx, sy) = edge_source_point(from, handle.or(edge.source_handle.as_deref()));
        let x1 = sx - min_x + padding;
        let y1 = sy - min_y + padding;
        let x2 = to.x - min_x + padding;
        let y2 = to.y + to.height / 2.0 - min_y + padding;
        let dash = if ft.dashed {
            " stroke-dasharray=\"6 4\""
        } else {
            ""
        };
        if interactive {
            svg.push_str(&format!(
                "  <g class=\"mad-vsm-edge\" data-edge-id=\"{}\" data-from-node=\"{}\" data-to-node=\"{}\">\n",
                escape_html(&edge.id),
                escape_html(&edge.from),
                escape_html(&edge.to),
            ));
        }
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
                escape_html(&format_duration_labeled(mins, true, dl))
            ));
        }
        if interactive {
            svg.push_str("  </g>\n");
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
        if interactive {
            svg.push_str(&format!(
                "  <g class=\"mad-vsm-node\" data-node-id=\"{}\" tabindex=\"0\" role=\"button\" aria-label=\"{}\">\n",
                escape_html(&node.id),
                escape_html(&node.label),
            ));
        }
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
        if interactive {
            svg.push_str("  </g>\n");
        }
    }

    if interactive {
        svg.push_str("  </g>\n");
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

fn edge_source_point(from: &VsmNode, source_handle: Option<&str>) -> (f64, f64) {
    match (&from.node_type, source_handle) {
        (VsmNodeType::Decision, Some("yes")) => {
            (from.x + from.width / 2.0, from.y + from.height)
        }
        (VsmNodeType::Decision, Some("no")) => {
            (from.x + from.width, from.y + from.height / 2.0)
        }
        _ => (from.x + from.width, from.y + from.height / 2.0),
    }
}

fn infer_decision_source_handle(
    edge: &VsmEdge,
    decision: &VsmNode,
    target: Option<&VsmNode>,
) -> &'static str {
    if edge.source_handle.as_deref() == Some("yes") {
        return "yes";
    }
    if edge.source_handle.as_deref() == Some("no") {
        return "no";
    }
    if let Some(label) = edge.label.as_deref() {
        let l = label.trim().to_lowercase();
        if matches!(
            l.as_str(),
            "yes" | "y" | "oui" | "true" | "approve" | "approved" | "ok" | "pass" | "accept"
        ) {
            return "yes";
        }
        if matches!(
            l.as_str(),
            "no" | "n" | "non" | "false" | "reject" | "rejected" | "denied" | "fail" | "nok"
        ) {
            return "no";
        }
    }
    if let Some(to) = target {
        let dc_y = decision.y + decision.height / 2.0;
        let tc_y = to.y + to.height / 2.0;
        return if tc_y <= dc_y { "yes" } else { "no" };
    }
    "yes"
}

fn resolve_decision_edge_handles(map: &ValueStreamMap) -> HashMap<String, String> {
    let node_by_id: HashMap<&str, &VsmNode> = map.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let mut outgoing: HashMap<&str, Vec<&VsmEdge>> = HashMap::new();
    for edge in &map.edges {
        let Some(from) = node_by_id.get(edge.from.as_str()) else {
            continue;
        };
        if !matches!(from.node_type, VsmNodeType::Decision) {
            continue;
        }
        outgoing.entry(edge.from.as_str()).or_default().push(edge);
    }

    let mut result = HashMap::new();
    for (_, edges) in outgoing {
        let mut used = HashSet::new();
        let mut sorted: Vec<&VsmEdge> = edges;
        sorted.sort_by(|a, b| {
            let ay = node_by_id
                .get(a.to.as_str())
                .map(|n| n.y + n.height / 2.0)
                .unwrap_or(0.0);
            let by = node_by_id
                .get(b.to.as_str())
                .map(|n| n.y + n.height / 2.0)
                .unwrap_or(0.0);
            ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
        });
        let decision = node_by_id.get(sorted[0].from.as_str()).copied();
        let Some(decision) = decision else {
            continue;
        };
        for edge in sorted {
            if edge.source_handle.as_deref() == Some("yes")
                || edge.source_handle.as_deref() == Some("no")
            {
                let h = edge.source_handle.clone().unwrap();
                result.insert(edge.id.clone(), h.clone());
                used.insert(h);
                continue;
            }
            let target = node_by_id.get(edge.to.as_str()).copied();
            let mut handle = infer_decision_source_handle(edge, decision, target).to_string();
            if used.contains(&handle) {
                handle = if handle == "yes" {
                    "no".into()
                } else {
                    "yes".into()
                };
            }
            used.insert(handle.clone());
            result.insert(edge.id.clone(), handle);
        }
    }
    result
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
    fn format_duration_french() {
        let fr = duration_labels(locale::strings(ReportLocale::Fr));
        assert_eq!(format_duration_labeled(1_440.0, true, fr), "1 j");
        assert_eq!(format_duration_labeled(10_080.0, true, fr), "1 sem");
        assert_eq!(format_duration_labeled(90.0, false, fr), "1 h 30 min");
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
                source_handle: None,
            }],
            messages: vec![],
            flow_types: vec![],
        };
        let timeline = build_timeline(&map);
        assert_eq!(timeline.stats.total_minutes, 120.0);
        assert_eq!(timeline.segments[0].from_label, "A");
    }

    #[test]
    fn resolve_decision_branches_from_labels_and_geometry() {
        let map = ValueStreamMap {
            nodes: vec![
                VsmNode {
                    id: "d".into(),
                    label: "Approved?".into(),
                    node_type: VsmNodeType::Decision,
                    x: 460.0,
                    y: 90.0,
                    width: 110.0,
                    height: 110.0,
                    notes: None,
                    role: None,
                    lead_time_minutes: None,
                    cycle_time_minutes: None,
                    author: None,
                },
                VsmNode {
                    id: "yes".into(),
                    label: "Enroll".into(),
                    node_type: VsmNodeType::Process,
                    x: 660.0,
                    y: 60.0,
                    width: 180.0,
                    height: 72.0,
                    notes: None,
                    role: None,
                    lead_time_minutes: None,
                    cycle_time_minutes: None,
                    author: None,
                },
                VsmNode {
                    id: "no".into(),
                    label: "Reject".into(),
                    node_type: VsmNodeType::Process,
                    x: 660.0,
                    y: 200.0,
                    width: 180.0,
                    height: 72.0,
                    notes: None,
                    role: None,
                    lead_time_minutes: None,
                    cycle_time_minutes: None,
                    author: None,
                },
            ],
            edges: vec![
                VsmEdge {
                    id: "e-yes".into(),
                    from: "d".into(),
                    to: "yes".into(),
                    label: Some("Yes".into()),
                    edge_type: "material".into(),
                    duration_minutes: None,
                    source_handle: None,
                },
                VsmEdge {
                    id: "e-no".into(),
                    from: "d".into(),
                    to: "no".into(),
                    label: Some("No".into()),
                    edge_type: "material".into(),
                    duration_minutes: None,
                    source_handle: None,
                },
            ],
            messages: vec![],
            flow_types: vec![],
        };
        let handles = resolve_decision_edge_handles(&map);
        assert_eq!(handles.get("e-yes").map(|s| s.as_str()), Some("yes"));
        assert_eq!(handles.get("e-no").map(|s| s.as_str()), Some("no"));
    }
}
