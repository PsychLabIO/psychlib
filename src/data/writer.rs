use crate::data::record::TrialRecord;
use crate::data::session::SessionHeader;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Generate a data filename from session metadata.
/// Format: `<participant>_<script_stem>_<YYYYMMDD_HHMMSS>.<ext>`
pub fn data_filename(header: &SessionHeader, ext: &str) -> String {
    let ts = header.started_at.format("%Y%m%d_%H%M%S");
    let participant = sanitise(&header.participant);
    let script = sanitise(&header.script_name);
    format!("{}_{}_{}.{}", participant, script, ts, ext)
}

fn sanitise(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Common interface for all data writers.
pub trait DataStore {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()>;
    fn close(self: Box<Self>) -> Result<()>;
}

pub struct CsvWriter {
    header: SessionHeader,
    writer: BufWriter<std::fs::File>,
    path: PathBuf,
    custom_keys: Option<Vec<String>>,
    trial_count: usize,
}

impl CsvWriter {
    pub fn create(output_dir: &Path, header: SessionHeader) -> Result<Self> {
        let filename = data_filename(&header, "csv");
        let path = output_dir.join(&filename);

        let file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

        let mut writer = BufWriter::new(file);

        write!(writer, "{}", header.to_csv_comments())
            .context("Failed to write CSV header comments")?;

        info!("CSV writer opened: {}", path.display());

        Ok(Self {
            header,
            writer,
            path,
            custom_keys: None,
            trial_count: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn ensure_header(&mut self, record: &TrialRecord) -> Result<()> {
        if self.custom_keys.is_some() {
            return Ok(());
        }

        let keys: Vec<String> = match &record.custom {
            serde_json::Value::Object(map) => {
                let mut keys: Vec<String> = map.keys().cloned().collect();
                keys.sort();
                keys
            }
            _ => Vec::new(),
        };

        let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
        let cols = TrialRecord::csv_column_names(&key_refs);
        writeln!(self.writer, "{}", cols.join(",")).context("Failed to write CSV column header")?;

        self.custom_keys = Some(keys);
        Ok(())
    }
}

impl DataStore for CsvWriter {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()> {
        self.ensure_header(record)?;

        let keys = self.custom_keys.as_ref().unwrap();

        let record_custom_keys: BTreeSet<String> = match &record.custom {
            serde_json::Value::Object(map) => map.keys().cloned().collect(),
            _ => BTreeSet::new(),
        };
        let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();

        for new_key in &record_custom_keys {
            if !keys.contains(new_key) {
                tracing::warn!(
                    "Trial {} has new custom key '{}' not present in first trial — \
                     this column will be misaligned in CSV",
                    record.trial_index,
                    new_key
                );
            }
        }

        let row = record.to_csv_row(&key_refs);
        writeln!(self.writer, "{}", escape_csv_row(&row))
            .context("Failed to write CSV trial row")?;

        self.trial_count += 1;
        debug!("CSV: wrote trial {}", record.trial_index);
        Ok(())
    }

    fn close(mut self: Box<Self>) -> Result<()> {
        self.header.close();
        self.writer.flush().context("Failed to flush CSV writer")?;
        info!(
            "CSV writer closed: {} ({} trials)",
            self.path.display(),
            self.trial_count
        );
        Ok(())
    }
}

/// Escape a CSV row: quote any field containing a comma, newline, or quote.
fn escape_csv_row(row: &[String]) -> String {
    row.iter()
        .map(|field| {
            if field.contains(',') || field.contains('"') || field.contains('\n') {
                format!("\"{}\"", field.replace('"', "\"\""))
            } else {
                field.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub struct JsonWriter {
    header: SessionHeader,
    writer: BufWriter<std::fs::File>,
    path: PathBuf,
    trial_count: usize,
}

impl JsonWriter {
    pub fn create(output_dir: &Path, header: SessionHeader) -> Result<Self> {
        let filename = data_filename(&header, "ndjson");
        let path = output_dir.join(&filename);

        let file = std::fs::File::create(&path)
            .with_context(|| format!("Failed to create JSON file: {}", path.display()))?;

        let mut writer = BufWriter::new(file);

        let mut header_val =
            serde_json::to_value(&header).context("Failed to serialize session header")?;
        header_val["type"] = serde_json::json!("session");
        writeln!(writer, "{}", serde_json::to_string(&header_val)?)
            .context("Failed to write JSON session header")?;

        info!("JSON writer opened: {}", path.display());

        Ok(Self {
            header,
            writer,
            path,
            trial_count: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl DataStore for JsonWriter {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()> {
        let mut val = serde_json::to_value(record).context("Failed to serialize trial record")?;
        val["type"] = serde_json::json!("trial");

        writeln!(self.writer, "{}", serde_json::to_string(&val)?)
            .context("Failed to write JSON trial row")?;

        self.trial_count += 1;
        debug!("JSON: wrote trial {}", record.trial_index);
        Ok(())
    }

    fn close(mut self: Box<Self>) -> Result<()> {
        self.header.close();
        self.writer.flush().context("Failed to flush JSON writer")?;
        info!(
            "JSON writer closed: {} ({} trials)",
            self.path.display(),
            self.trial_count
        );
        Ok(())
    }
}
