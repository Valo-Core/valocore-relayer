use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use regex::Regex;
use serde::Deserialize;
use sha2::Sha256;
use sqlx::PgPool;

use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct PullRequestPayload {
    action: String,
    pull_request: PullRequest,
}

#[derive(Deserialize)]
struct PullRequest {
    merged: bool,
    body: Option<String>,
}

pub async fn handle_github_webhook(
    State(AppState {
        db_pool,
        webhook_secret,
    }): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    if let Err(e) = verify_signature(&headers, &webhook_secret, &body) {
        tracing::warn!("Signature verification failed: {}", e);
        return (StatusCode::FORBIDDEN, "Invalid signature");
    }

    let payload = match serde_json::from_slice::<PullRequestPayload>(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to parse payload: {}", e);
            return (StatusCode::BAD_REQUEST, "Invalid JSON payload");
        }
    };

    if payload.action != "closed" || !payload.pull_request.merged {
        tracing::info!("Ignoring non-merged PR");
        return (StatusCode::OK, "Ignored");
    }

    let Some(body) = payload.pull_request.body else {
        return (StatusCode::OK, "No body to parse");
    };

    let milestone_ids = extract_milestone_ids(&body);
    if milestone_ids.is_empty() {
        return (StatusCode::OK, "No milestones found");
    }

    if let Err(e) = update_milestones(&db_pool, &milestone_ids).await {
        tracing::error!("Failed to update milestones: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update milestones",
        );
    }

    (StatusCode::OK, "Milestones updated")
}

fn verify_signature(
    headers: &HeaderMap,
    secret: &str,
    body: &[u8],
) -> Result<(), String> {
    let signature_header = headers
        .get("X-Hub-Signature-256")
        .ok_or("Missing signature header")?;

    let signature = signature_header
        .to_str()
        .map_err(|_| "Invalid signature encoding")?;

    let expected_signature = format!(
        "sha256={}",
        hex::encode(
            HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|_| "Failed to create HMAC")?
                .chain_update(body)
                .finalize()
                .into_bytes()
        )
    );

    if !constant_time_eq::constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()) {
        return Err("Signature mismatch".into());
    }

    Ok(())
}

fn extract_milestone_ids(body: &str) -> Vec<i64> {
    let re = Regex::new(r"(?i)(?:fixes|closes|resolves)\s+valocore#(\d+)").unwrap();
    re.captures_iter(body)
        .filter_map(|cap| cap[1].parse::<i64>().ok())
        .collect()
}

async fn update_milestones(pool: &PgPool, milestone_ids: &[i64]) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    for &milestone_id in milestone_ids {
        sqlx::query(
            "UPDATE tracked_milestones SET status = 'Completed' WHERE id = $1",
        )
        .bind(milestone_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
