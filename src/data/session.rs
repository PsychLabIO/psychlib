use crate::clock::ClockInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHeader {
    pub participant: String,
    pub script_name: String,
    pub script_path: String,
    pub psychlib_version: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub seed: Option<u64>,
    pub clock: ClockInfo,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

impl SessionHeader {
    pub fn new(
        participant: impl Into<String>,
        script_name: impl Into<String>,
        script_path: impl Into<String>,
        seed: Option<u64>,
        clock_info: ClockInfo,
    ) -> Self {
        Self {
            participant: participant.into(),
            script_name: script_name.into(),
            script_path: script_path.into(),
            psychlib_version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: Utc::now(),
            ended_at: None,
            seed,
            clock: clock_info,
            extra: BTreeMap::new(),
        }
    }

    pub fn close(&mut self) {
        self.ended_at = Some(Utc::now());
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.ended_at.map(|end| end - self.started_at)
    }

    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }

    pub fn to_csv_comments(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("# psychlib_version: {}", self.psychlib_version));
        lines.push(format!("# participant: {}", self.participant));
        lines.push(format!("# script_name: {}", self.script_name));
        lines.push(format!("# script_path: {}", self.script_path));
        lines.push(format!(
            "# started_at: {}",
            self.started_at
                .to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        ));

        if let Some(ended) = self.ended_at {
            lines.push(format!(
                "# ended_at: {}",
                ended.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
            ));
        }

        match self.seed {
            Some(s) => lines.push(format!("# seed: {}", s)),
            None => lines.push("# seed: entropy".to_string()),
        }

        lines.push(format!("# platform: {}", self.clock.platform));
        lines.push(format!(
            "# high_precision_sleep: {}",
            self.clock.high_precision_sleep
        ));

        for (k, v) in &self.extra {
            lines.push(format!("# {}: {}", k, v));
        }

        lines.join("\n") + "\n"
    }
}