use crate::data::record::{value_to_csv_field, TrialRecord};
use crate::data::session::SessionHeader;
use anyhow::{Context, Result};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

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
    columns: Option<Vec<String>>,
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
            columns: None,
            trial_count: 0,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn ensure_columns(&mut self, record: &TrialRecord) -> Result<()> {
        if self.columns.is_some() {
            return Ok(());
        }

        let cols: Vec<String> = record.fields.keys().cloned().collect();
        writeln!(self.writer, "{}", cols.join(","))
            .context("Failed to write CSV column header")?;

        self.columns = Some(cols);
        Ok(())
    }
}

impl DataStore for CsvWriter {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()> {
        self.ensure_columns(record)?;

        let columns = self.columns.as_ref().unwrap();

        for key in record.fields.keys() {
            if !columns.contains(key) {
                warn!(
                    "Trial {} has field '{}' not present in first trial — \
                     this value cannot be placed in a CSV column and will be dropped",
                    record
                        .fields
                        .get("trial_index")
                        .and_then(|v| v.as_u64())
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "?".to_string()),
                    key
                );
            }
        }

        let row: Vec<String> = columns
            .iter()
            .map(|col| {
                record
                    .fields
                    .get(col)
                    .map(value_to_csv_field)
                    .unwrap_or_default()
            })
            .collect();

        writeln!(self.writer, "{}", escape_csv_row(&row))
            .context("Failed to write CSV trial row")?;

        self.trial_count += 1;
        debug!(
            "CSV: wrote trial {} ({})",
            record
                .fields
                .get("trial_index")
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string()),
            self.path.display()
        );
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
    path: PathBuf,
    trials: Vec<serde_json::Value>,
}

impl JsonWriter {
    pub fn create(output_dir: &Path, header: SessionHeader) -> Result<Self> {
        let filename = data_filename(&header, "json");
        let path = output_dir.join(&filename);

        info!("JSON writer opened: {}", path.display());

        Ok(Self {
            header,
            path,
            trials: Vec::new(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl DataStore for JsonWriter {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()> {
        let val = serde_json::to_value(&record.fields)
            .context("Failed to serialize trial record")?;

        debug!(
            "JSON: queued trial {} ({})",
            record
                .fields
                .get("trial_index")
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".to_string()),
            self.path.display()
        );

        self.trials.push(val);
        Ok(())
    }

    fn close(mut self: Box<Self>) -> Result<()> {
        self.header.close();

        let header_val =
            serde_json::to_value(&self.header).context("Failed to serialize session header")?;

        let output = serde_json::json!({
            "session": header_val,
            "trials": self.trials,
        });

        let file = std::fs::File::create(&self.path)
            .with_context(|| format!("Failed to create JSON file: {}", self.path.display()))?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &output)
            .context("Failed to write JSON output")?;
        writeln!(writer).context("Failed to write trailing newline")?;
        writer.flush().context("Failed to flush JSON writer")?;

        let trial_count = self.trials.len();
        info!(
            "JSON writer closed: {} ({} trials)",
            self.path.display(),
            trial_count
        );
        Ok(())
    }
}