mod workspace_store;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use mad_core::{
    default_html_options, render_html, Evaluator, EvaluationWorkspace, PillarId, PolicyBundle,
    Requirement, ScoringConfig,
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
}

#[derive(Deserialize)]
struct AddRequirementBody {
    pillar_id: PillarId,
    requirement: Requirement,
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
        .route("/api/workspace/requirements", post(add_requirement))
        .route(
            "/api/workspace/requirements/{id}",
            axum::routing::delete(delete_requirement),
        )
        .route("/api/workspace/assessments", put(set_assessment))
        .route("/api/workspace/scoring", put(update_scoring))
        .route("/api/policy", get(policy))
        .route("/api/evaluation", get(evaluation))
        .route("/api/report.html", get(report_html))
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
        name: "Operation M.A.D. (Mobile MDM Evaluation)",
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

async fn add_requirement(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddRequirementBody>,
) -> Result<Json<EvaluationWorkspace>, AppError> {
    let ws = state
        .workspace
        .update(|w| {
            w.add_requirement(body.pillar_id, body.requirement);
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
    }))
}

async fn evaluation(
    State(state): State<Arc<AppState>>,
) -> Result<Json<mad_core::EvaluationReport>, AppError> {
    Ok(Json(evaluate_from_state(&state).await?))
}

async fn report_html(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let workspace = state.workspace.get().await;
    let evaluation = evaluate_from_state(&state).await?;
    let bundle = workspace.to_policy_bundle();

    let logo_path = if state.logo_path.exists() {
        Some(state.logo_path.as_path())
    } else {
        None
    };
    let options = default_html_options(logo_path);
    let html = render_html(&bundle, &evaluation, &options);

    Ok((
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
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
