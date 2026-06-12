mod workspace_store;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use mad_core::{
    default_html_options, default_pdf_options, filter_evaluation_by_tags, filter_vendor_map,
    parse_vendor_tags_query, render_html, render_pdf, Evaluator, ReportLocale,
    parse_workspace_import, EvaluationWorkspace, Pillar, PolicyBundle, ProcurementConfig,
    Requirement, ScoringConfig, ValueStreamEntry, ValueStreamMap, Vendor, VendorImportMode,
    VendorDocSection, VendorImportResult,
    VendorSetFile, WorkspaceImportResult,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use workspace_store::{workspace_path, WorkspaceStore};

struct AppState {
    workspace: Arc<WorkspaceStore>,
    logo_path: PathBuf,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    name: &'static str,
}

#[derive(Serialize)]
struct PolicySummary {
    version: String,
    pillar_count: usize,
    total_requirements: usize,
    critical_requirements: usize,
    pillars: Vec<mad_core::Pillar>,
    scoring: ScoringConfig,
    procurement: ProcurementConfig,
    value_streams: HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: HashMap<String, Vec<VendorDocSection>>,
}

#[derive(Deserialize)]
struct AddPillarBody {
    id: String,
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct UpdatePillarBody {
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct AddRequirementBody {
    pillar_id: String,
    requirement: Requirement,
}

#[derive(Deserialize)]
struct UpdateRequirementBody {
    pillar_id: String,
    requirement: Requirement,
}

#[derive(Deserialize)]
struct AddVendorBody {
    vendor: Vendor,
}

#[derive(Deserialize)]
struct UpdateVendorBody {
    vendor: Vendor,
}

#[derive(Deserialize)]
struct SetAssessmentBody {
    vendor_id: String,
    requirement_id: String,
    status: mad_core::ComplianceStatus,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateScoringBody {
    scoring: ScoringConfig,
}

#[derive(Deserialize)]
struct UpdateProcurementBody {
    procurement: ProcurementConfig,
}

#[derive(Deserialize)]
struct UpdateValueStreamBody {
    value_stream: ValueStreamMap,
}

#[derive(Deserialize)]
struct CreateValueStreamBody {
    name: String,
}

#[derive(Deserialize)]
struct UpsertValueStreamBody {
    name: String,
    #[serde(flatten)]
    map: ValueStreamMap,
}

#[derive(Deserialize)]
struct CreateVendorDocBody {
    name: String,
}

#[derive(Deserialize)]
struct UpsertVendorDocBody {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    overview: Option<String>,
    #[serde(default)]
    items: Vec<mad_core::VendorDocItem>,
}

#[derive(Deserialize)]
struct ImportVendorsQuery {
    #[serde(default)]
    mode: VendorImportMode,
}

#[derive(Serialize)]
struct ImportVendorsResponse {
    result: VendorImportResult,
    workspace: EvaluationWorkspace,
}

#[derive(Serialize)]
struct WorkspaceImportResponse {
    result: WorkspaceImportResult,
    workspace: EvaluationWorkspace,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let policies_dir = std::env::var("MAD_POLICIES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("policies"));

    let data_dir = std::env::var("MAD_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"));

    let logo_path = std::env::var("MAD_LOGO_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets/logo.png"));

    let bundle = PolicyBundle::load_dir(&policies_dir).expect("failed to load policies");
    let workspace = Arc::new(
        WorkspaceStore::load(&bundle, workspace_path(&data_dir)).await,
    );

    let state = Arc::new(AppState {
        workspace,
        logo_path,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/workspace", get(get_workspace).put(put_workspace))
        .route("/api/workspace/export", get(export_workspace))
        .route("/api/workspace/import", post(import_workspace))
        .route("/api/workspace/pillars", post(add_pillar))
        .route(
            "/api/workspace/pillars/{id}",
            put(update_pillar).delete(delete_pillar),
        )
        .route("/api/workspace/requirements", post(add_requirement))
        .route(
            "/api/workspace/requirements/{id}",
            put(update_requirement).delete(delete_requirement),
        )
        .route("/api/workspace/vendors", post(add_vendor))
        .route("/api/workspace/vendors/export", get(export_vendors))
        .route("/api/workspace/vendors/import", post(import_vendors))
        .route(
            "/api/workspace/vendors/load-example",
            post(load_example_vendors),
        )
        .route(
            "/api/workspace/vendors/{id}",
            put(update_vendor).delete(delete_vendor),
        )
        .route(
            "/api/workspace/vendors/{id}/value-stream",
            put(update_value_stream),
        )
        .route(
            "/api/workspace/vendors/{id}/value-streams",
            post(create_value_stream),
        )
        .route(
            "/api/workspace/vendors/{id}/value-streams/{stream_id}",
            put(update_value_stream_entry).delete(delete_value_stream_entry),
        )
        .route(
            "/api/workspace/vendors/{id}/docs",
            post(create_vendor_doc),
        )
        .route(
            "/api/workspace/vendors/{id}/docs/{doc_id}",
            put(update_vendor_doc).delete(delete_vendor_doc),
        )
        .route("/api/workspace/assessments", put(set_assessment))
        .route("/api/workspace/scoring", put(update_scoring))
        .route("/api/workspace/procurement", put(update_procurement))
        .route("/api/policy", get(policy))
        .route("/api/evaluation", get(evaluation))
        .route("/api/report.html", get(report_html))
        .route("/api/report.pdf", get(report_pdf))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::info!("mad-server listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

async fn evaluate_from_state(state: &AppState) -> Result<mad_core::EvaluationReport, AppError> {
    let workspace = state.workspace.get().await;
    let evaluator = Evaluator::from_workspace(&workspace);
    Ok(evaluator.evaluate()?)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        name: "MAD (Mobile Assessment & Defense)",
    })
}

async fn get_workspace(State(state): State<Arc<AppState>>) -> Json<EvaluationWorkspace> {
    Json(state.workspace.get().await)
}

async fn put_workspace(
    State(state): State<Arc<AppState>>,
    Json(workspace): Json<EvaluationWorkspace>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    state.workspace.replace(workspace).await?;
    Ok(Json(state.workspace.get().await))
}

async fn add_pillar(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddPillarBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let pillar = Pillar {
        id: body.id,
        name: body.name,
        description: body.description,
        requirements: Vec::new(),
    };
    let ws = state
        .workspace
        .update(|w| {
            w.add_pillar(pillar);
        })
        .await?;
    Ok(Json(ws))
}

async fn update_pillar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdatePillarBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.pillars.iter().any(|p| p.id == id) {
        return Ok(Json(workspace));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.update_pillar(&id, body.name, body.description);
        })
        .await?;
    Ok(Json(ws))
}

async fn delete_pillar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.pillars.iter().any(|p| p.id == id) {
        return Ok(StatusCode::NOT_FOUND);
    }
    if mad_core::pillar::builtin::is_builtin(&id) {
        return Ok(StatusCode::FORBIDDEN);
    }
    state
        .workspace
        .update(|w| {
            w.remove_pillar(&id);
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_requirement(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddRequirementBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.add_requirement(&body.pillar_id, body.requirement);
        })
        .await?;
    Ok(Json(ws))
}

async fn update_requirement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateRequirementBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    let exists = workspace
        .pillars
        .iter()
        .flat_map(|p| &p.requirements)
        .any(|r| r.id == id);
    if !exists {
        return Ok(Json(workspace));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.update_requirement(&id, &body.pillar_id, body.requirement);
        })
        .await?;
    Ok(Json(ws))
}

async fn delete_requirement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let workspace = state.workspace.get().await;
    let exists = workspace
        .pillars
        .iter()
        .flat_map(|p| &p.requirements)
        .any(|r| r.id == id);
    if !exists {
        return Ok(StatusCode::NOT_FOUND);
    }
    state
        .workspace
        .update(|w| {
            w.remove_requirement(&id);
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn export_workspace(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace.get().await;
    let export = workspace.export_bundle(export_timestamp());
    let json = serde_json::to_string_pretty(&export).map_err(json_err)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"mad-workspace.json\"",
            ),
        ],
        json,
    ))
}

async fn import_workspace(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ImportVendorsQuery>,
    body: String,
) -> Result<Json<WorkspaceImportResponse>, AppError> {
    let parsed = parse_workspace_import(&body).map_err(|e| {
        mad_core::MadError::Io {
            path: PathBuf::from("workspace-import"),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        }
    })?;
    let mut import_result = WorkspaceImportResult {
        kind: "unknown",
        pillars: 0,
        requirements: 0,
        vendors: 0,
        assessments: 0,
        value_stream_maps: 0,
        vendor_doc_sections: 0,
        vendor_result: None,
    };
    let workspace = state
        .workspace
        .update(|w| {
            import_result = w.import_parsed(parsed, query.mode);
        })
        .await?;
    Ok(Json(WorkspaceImportResponse {
        result: import_result,
        workspace,
    }))
}

fn json_err(e: serde_json::Error) -> mad_core::MadError {
    mad_core::MadError::Io {
        path: PathBuf::from("json"),
        source: std::io::Error::new(std::io::ErrorKind::Other, e),
    }
}

async fn export_vendors(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace.get().await;
    let exported_at = export_timestamp();
    let export = workspace.export_vendor_set(exported_at);
    let json = serde_json::to_string_pretty(&export).map_err(json_err)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"mad-vendors.json\"",
            ),
        ],
        json,
    ))
}

async fn load_example_vendors(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ImportVendorsResponse>, AppError> {
    let mut import_result = VendorImportResult {
        added: 0,
        updated: 0,
        skipped: 0,
        removed: 0,
        value_streams_imported: 0,
        vendor_docs_imported: 0,
    };
    let workspace = state
        .workspace
        .update(|w| {
            import_result = w.import_sample_vendors();
        })
        .await?;
    Ok(Json(ImportVendorsResponse {
        result: import_result,
        workspace,
    }))
}

async fn import_vendors(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ImportVendorsQuery>,
    Json(file): Json<VendorSetFile>,
) -> Result<Json<ImportVendorsResponse>, AppError> {
    let mode = query.mode;
    let mut import_result = VendorImportResult {
        added: 0,
        updated: 0,
        skipped: 0,
        removed: 0,
        value_streams_imported: 0,
        vendor_docs_imported: 0,
    };
    let workspace = state
        .workspace
        .update(|w| {
            import_result = w.import_vendor_set(file, mode);
        })
        .await?;
    Ok(Json(ImportVendorsResponse {
        result: import_result,
        workspace,
    }))
}

fn export_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

async fn add_vendor(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddVendorBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            if !w.add_vendor(body.vendor) {
                tracing::warn!("vendor already exists or could not be added");
            }
        })
        .await?;
    Ok(Json(ws))
}

async fn update_vendor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateVendorBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Err(AppError(mad_core::MadError::VendorNotFound(id)));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.update_vendor(&id, body.vendor);
        })
        .await?;
    Ok(Json(ws))
}

async fn delete_vendor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(StatusCode::NOT_FOUND);
    }
    state
        .workspace
        .update(|w| {
            w.remove_vendor(&id);
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_assessment(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetAssessmentBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.set_assessment(
                &body.vendor_id,
                &body.requirement_id,
                body.status,
                body.notes,
            );
        })
        .await?;
    Ok(Json(ws))
}

async fn update_scoring(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateScoringBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.scoring = body.scoring;
        })
        .await?;
    Ok(Json(ws))
}

async fn policy(State(state): State<Arc<AppState>>) -> Result<Json<PolicySummary>, AppError> {
    let workspace = state.workspace.get().await;
    Ok(Json(PolicySummary {
        version: workspace.policy_version.clone(),
        pillar_count: workspace.pillars.len(),
        total_requirements: workspace.total_requirements(),
        critical_requirements: workspace.critical_requirements(),
        pillars: workspace.pillars.clone(),
        scoring: workspace.scoring.clone(),
        procurement: workspace.procurement.clone(),
        value_streams: workspace.value_streams.clone(),
        vendor_docs: workspace.vendor_docs.clone(),
    }))
}

async fn update_value_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateValueStreamBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(Json(workspace));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.set_value_stream(&id, body.value_stream);
        })
        .await?;
    Ok(Json(ws))
}

async fn create_value_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<CreateValueStreamBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(Json(workspace));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.create_value_stream(&id, body.name);
        })
        .await?;
    Ok(Json(ws))
}

async fn update_value_stream_entry(
    State(state): State<Arc<AppState>>,
    Path((id, stream_id)): Path<(String, String)>,
    Json(body): Json<UpsertValueStreamBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(Json(workspace));
    }
    let entry = ValueStreamEntry {
        id: stream_id,
        name: body.name,
        map: body.map,
    };
    let ws = state
        .workspace
        .update(|w| {
            w.upsert_value_stream_entry(&id, entry);
        })
        .await?;
    Ok(Json(ws))
}

async fn delete_value_stream_entry(
    State(state): State<Arc<AppState>>,
    Path((id, stream_id)): Path<(String, String)>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.remove_value_stream(&id, &stream_id);
        })
        .await?;
    Ok(Json(ws))
}

async fn create_vendor_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<CreateVendorDocBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(Json(workspace));
    }
    let ws = state
        .workspace
        .update(|w| {
            w.create_vendor_doc(&id, body.name);
        })
        .await?;
    Ok(Json(ws))
}

async fn update_vendor_doc(
    State(state): State<Arc<AppState>>,
    Path((id, doc_id)): Path<(String, String)>,
    Json(body): Json<UpsertVendorDocBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let workspace = state.workspace.get().await;
    if !workspace.vendors.iter().any(|v| v.id.0 == id) {
        return Ok(Json(workspace));
    }
    let section = VendorDocSection {
        id: doc_id,
        name: body.name,
        color: body.color,
        overview: body.overview,
        items: body.items,
    };
    let ws = state
        .workspace
        .update(|w| {
            w.upsert_vendor_doc(&id, section);
        })
        .await?;
    Ok(Json(ws))
}

async fn delete_vendor_doc(
    State(state): State<Arc<AppState>>,
    Path((id, doc_id)): Path<(String, String)>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.remove_vendor_doc(&id, &doc_id);
        })
        .await?;
    Ok(Json(ws))
}

async fn update_procurement(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProcurementBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.procurement = body.procurement;
        })
        .await?;
    Ok(Json(ws))
}

async fn evaluation(
    State(state): State<Arc<AppState>>,
) -> Result<Json<mad_core::EvaluationReport>, AppError> {
    Ok(Json(evaluate_from_state(&state).await?))
}

#[derive(Debug, Deserialize, Default)]
struct ReportHtmlQuery {
    /// When `1`, serve inline for iframe embed (hides chrome via `?embed=1` in the report).
    embed: Option<String>,
    /// Report UI language (`en` or `fr`).
    lang: Option<String>,
    /// Comma-separated vendor tags (`shortlist,ios`).
    tags: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ReportPdfQuery {
    tags: Option<String>,
}

fn report_tag_filter(
    evaluation: mad_core::EvaluationReport,
    value_streams: std::collections::HashMap<String, Vec<ValueStreamEntry>>,
    vendor_docs: std::collections::HashMap<String, Vec<VendorDocSection>>,
    tags: Option<&str>,
) -> (
    mad_core::EvaluationReport,
    std::collections::HashMap<String, Vec<ValueStreamEntry>>,
    std::collections::HashMap<String, Vec<VendorDocSection>>,
    Vec<String>,
) {
    let active_tags = tags
        .map(parse_vendor_tags_query)
        .unwrap_or_default();
    let evaluation = filter_evaluation_by_tags(evaluation, &active_tags);
    let value_streams = filter_vendor_map(&value_streams, &evaluation.vendors);
    let vendor_docs = filter_vendor_map(&vendor_docs, &evaluation.vendors);
    (evaluation, value_streams, vendor_docs, active_tags)
}

async fn report_html(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReportHtmlQuery>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace.get().await;
    let evaluation = evaluate_from_state(&state).await?;
    let bundle = workspace.to_policy_bundle();
    let (evaluation, value_streams, vendor_docs, active_tags) = report_tag_filter(
        evaluation,
        workspace.value_streams.clone(),
        workspace.vendor_docs.clone(),
        query.tags.as_deref(),
    );

    let logo_path = if state.logo_path.exists() {
        Some(state.logo_path.as_path())
    } else {
        None
    };
    let mut options = default_html_options(logo_path);
    if let Some(lang) = query.lang.as_deref() {
        options.locale = ReportLocale::parse(lang);
    }
    options.filter_tags = active_tags;
    let html = render_html(
        &bundle,
        &evaluation,
        &value_streams,
        &vendor_docs,
        &options,
    );

    let inline = query.embed.as_deref() == Some("1");
    let disposition = if inline {
        "inline"
    } else {
        "attachment; filename=\"mad-evaluation-report.html\""
    };

    Ok((
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        html,
    ))
}

async fn report_pdf(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReportPdfQuery>,
) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace.get().await;
    let evaluation = evaluate_from_state(&state).await?;
    let bundle = workspace.to_policy_bundle();
    let (evaluation, value_streams, vendor_docs, _active_tags) = report_tag_filter(
        evaluation,
        workspace.value_streams.clone(),
        workspace.vendor_docs.clone(),
        query.tags.as_deref(),
    );
    let logo_path = if state.logo_path.exists() {
        Some(state.logo_path.as_path())
    } else {
        None
    };
    let options = default_pdf_options(logo_path);
    let pdf = render_pdf(
        &bundle,
        &evaluation,
        &value_streams,
        &vendor_docs,
        &options,
    )
    .map_err(|e| {
        mad_core::MadError::Io {
            path: PathBuf::from("report.pdf"),
            source: std::io::Error::new(std::io::ErrorKind::Other, e),
        }
    })?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"mad-evaluation-report.pdf\"",
            ),
        ],
        pdf,
    ))
}

struct AppError(mad_core::MadError);

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self(mad_core::MadError::Io {
            path: PathBuf::from("workspace"),
            source: value,
        })
    }
}

impl From<mad_core::MadError> for AppError {
    fn from(value: mad_core::MadError) -> Self {
        Self(value)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}
