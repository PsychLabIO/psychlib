use crate::clock::Instant;
use crate::io::response::Response;
use crate::scheduler::{Event, TrialPhase};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub trial_index: usize,
    pub block_index: usize,
    pub phase: TrialPhase,
    pub recorded_at_wall: DateTime<Utc>,
    pub recorded_at: Instant,
    pub response_key: Option<String>,
    pub rt_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rt_raw: Option<RtRaw>,
    pub timed_out: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stimulus_onset_error_us: Option<i64>,
    #[serde(flatten)]
    pub custom: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtRaw {
    pub onset: Instant,
    pub keypress: Instant,
    pub rt_nanos: i64,
}

impl TrialRecord {
    pub fn new(
        trial_index: usize,
        block_index: usize,
        phase: TrialPhase,
        response: Option<Response>,
        events: &[Event],
        custom: serde_json::Value,
        now: Instant,
        now_wall: DateTime<Utc>,
    ) -> Self {
        let (response_key, rt_ms, rt_raw, timed_out) = match response {
            Some(ref r) => {
                let raw = RtRaw {
                    onset: r.onset,
                    keypress: r.timestamp,
                    rt_nanos: r.rt.as_nanos(),
                };
                (Some(r.key.to_string()), Some(r.rt_ms()), Some(raw), false)
            }
            None => (None, None, None, true),
        };

        use crate::scheduler::EventKind;
        let stimulus_onset_error_us = events
            .iter()
            .find(|e| e.kind == EventKind::StimulusOnset)
            .and_then(|e| e.timing_error())
            .map(|d| d.as_micros());

        let custom = match custom {
            serde_json::Value::Object(_) => custom,
            other => {
                serde_json::json!({ "value": other })
            }
        };

        Self {
            trial_index,
            block_index,
            phase,
            recorded_at_wall: now_wall,
            recorded_at: now,
            response_key,
            rt_ms,
            rt_raw,
            timed_out,
            stimulus_onset_error_us,
            custom,
        }
    }

    pub fn csv_column_names(custom_keys: &[&str]) -> Vec<String> {
        let mut cols = vec![
            "trial_index",
            "block_index",
            "phase",
            "recorded_at_wall",
            "response_key",
            "rt_ms",
            "timed_out",
            "stimulus_onset_error_us",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

        let mut custom = custom_keys
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        custom.sort();
        cols.extend(custom);
        cols
    }

    pub fn to_csv_row(&self, custom_keys: &[&str]) -> Vec<String> {
        let mut row = vec![
            self.trial_index.to_string(),
            self.block_index.to_string(),
            format!("{:?}", self.phase).to_lowercase(),
            self.recorded_at_wall
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            self.response_key.clone().unwrap_or_default(),
            self.rt_ms.map_or(String::new(), |v| format!("{:.3}", v)),
            self.timed_out.to_string(),
            self.stimulus_onset_error_us
                .map_or(String::new(), |v| v.to_string()),
        ];

        let mut keys: Vec<_> = custom_keys.to_vec();
        keys.sort();
        for key in keys {
            let val = self
                .custom
                .get(key)
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            row.push(val);
        }

        row
    }
}
