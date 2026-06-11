use serde::{Deserialize, Serialize};

/// Per-vendor value stream map (VSM): process flow with nodes, edges, and messages.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValueStreamMap {
    #[serde(default)]
    pub nodes: Vec<VsmNode>,
    #[serde(default)]
    pub edges: Vec<VsmEdge>,
    #[serde(default)]
    pub messages: Vec<VsmMessage>,
    #[serde(default)]
    pub flow_types: Vec<VsmFlowTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VsmNodeType {
    Process,
    Decision,
    Info,
    Delay,
    External,
    Customer,
    Supplier,
    Inventory,
    Kaizen,
}

impl Default for VsmNodeType {
    fn default() -> Self {
        Self::Process
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmFlowTypeDef {
    pub id: String,
    pub label: String,
    pub color: String,
    #[serde(default)]
    pub dash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub node_type: VsmNodeType,
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_node_width")]
    pub width: f64,
    #[serde(default = "default_node_height")]
    pub height: f64,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub lead_time_minutes: Option<f64>,
    #[serde(default)]
    pub cycle_time_minutes: Option<f64>,
    #[serde(default)]
    pub author: Option<String>,
}

fn default_node_width() -> f64 {
    180.0
}

fn default_node_height() -> f64 {
    72.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_edge_type")]
    pub edge_type: String,
    #[serde(default)]
    pub duration_minutes: Option<f64>,
}

fn default_edge_type() -> String {
    "material".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmMessage {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub edge_id: Option<String>,
}
