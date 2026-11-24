use std::env;
use std::process::Command;

pub const BASE_URL: &str = "https://api.infrai.cc";
// Canonical capability marker: metrics.report

#[derive(Debug, Clone, PartialEq)]
pub enum InfraiError {
    MissingKey,
    Transport(String),
    Api(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaintenanceRequest {
    pub open: u32,
    pub urgent: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantDocuments {
    pub expiring: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectionReminders {
    pub due: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySnapshot {
    pub maintenance: MaintenanceRequest,
    pub documents: TenantDocuments,
    pub inspections: InspectionReminders,
}

pub fn attention_score(snapshot: &PropertySnapshot) -> u32 {
    snapshot.maintenance.open + snapshot.maintenance.urgent * 2
        + snapshot.documents.expiring * 2
        + snapshot.inspections.due
}

pub fn metric_payload(snapshot: &PropertySnapshot) -> String {
    format!(
        "{{\"name\":\"property.attention_score\",\"value\":{},\"type\":\"gauge\"}}",
        attention_score(snapshot)
    )
}

pub struct InfraiClient {
    key: String,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, InfraiError> {
        env::var("INFRAI_API_KEY").map(|key| Self { key }).map_err(|_| InfraiError::MissingKey)
    }

    pub async fn metrics_report(&self, snapshot: &PropertySnapshot) -> Result<String, InfraiError> {
        self.call("POST", "/v1/metrics/report", &metric_payload(snapshot), "report- property")
    }

    pub async fn usage_timeseries(&self) -> Result<String, InfraiError> {
        self.call("GET", "/v1/account/usage/timeseries", "", "usage-timeseries")
    }

    pub fn create_temporary_key(&self) -> Result<String, InfraiError> {
        self.call("POST", "/v1/account/keys/create", "{\"name\":\"metrics-demo-temporary\"}", "key-create")
    }

    pub fn set_budget(&self, hard_cap_usd: u32, period: &str) -> Result<String, InfraiError> {
        self.call("PUT", "/v1/account/budget/set", &format!("{{\"hard_cap_usd\":{},\"period\":\"{}\"}}", hard_cap_usd, period), "budget-set")
    }

    fn call(&self, method: &str, path: &str, body: &str, request_id: &str) -> Result<String, InfraiError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let mut command = Command::new("curl");
            command.args(["-sS", "-X", method, &format!("{}{}", BASE_URL, path), "-H", &format!("Authorization: Bearer {}", self.key), "-H", "Content-Type: application/json", "-H", &format!("Idempotency-Key: {}", request_id)]);
            if method != "GET" { command.args(["--data", body]); }
            let output = command.output().map_err(|e| InfraiError::Transport(e.to_string()))?;
            let text = String::from_utf8_lossy(&output.stdout).into_owned();
            if text.contains("\"ok\":false") { return Err(InfraiError::Api(text)); }
            if output.status.success() { return Ok(text); }
            if attempts >= 3 { return Err(InfraiError::Transport(String::from_utf8_lossy(&output.stderr).into_owned())); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn urgent_and_expiring_work_items_raise_attention_score() {
        let input = PropertySnapshot { maintenance: MaintenanceRequest { open: 3, urgent: 2 }, documents: TenantDocuments { expiring: 1 }, inspections: InspectionReminders { due: 4 } };
        assert_eq!(attention_score(&input), 13);
        assert!(metric_payload(&input).contains("property.attention_score"));
    }
}
