use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub fields: IndexMap<String, Value>,
}

impl TrialRecord {
    pub fn new(fields: IndexMap<String, Value>) -> Self {
        Self { fields }
    }

    pub fn builder() -> TrialRecordBuilder {
        TrialRecordBuilder::default()
    }
}

#[derive(Default)]
pub struct TrialRecordBuilder {
    fields: IndexMap<String, Value>,
}

impl TrialRecordBuilder {
    pub fn field(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn field_opt(mut self, key: impl Into<String>, value: Option<impl Into<Value>>) -> Self {
        if let Some(v) = value {
            self.fields.insert(key.into(), v.into());
        }
        self
    }

    pub fn merge(mut self, extra: impl IntoIterator<Item = (String, Value)>) -> Self {
        self.fields.extend(extra);
        self
    }

    pub fn build(self) -> TrialRecord {
        TrialRecord::new(self.fields)
    }
}

pub fn value_to_csv_field(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}