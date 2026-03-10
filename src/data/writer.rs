use crate::data::record::{TrialRecord, value_to_csv_field};
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

/// Output format selection, set via `experiment:set_format()` in the script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Csv,
    Json,
    Both,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// Common interface for all data writers.
pub trait DataStore {
    fn write_trial(&mut self, record: &TrialRecord) -> Result<()>;
    fn close(self: Box<Self>) -> Result<()>;
}

pub struct MultiWriter {
    writers: Vec<Box<dyn DataStore + Send>>,
    output_dir: std::path::PathBuf,
    header: SessionHeader,
    format: OutputFormat,
    initialised: bool,
}

impl MultiWriter {
    /// Create a new `MultiWriter`. No files are created until the first
    /// `write_trial` or `close` call.
    pub fn new(output_dir: &Path, header: SessionHeader) -> Result<Self> {
        Ok(Self {
            writers: Vec::new(),
            output_dir: output_dir.to_path_buf(),
            header,
            format: OutputFormat::Csv,
            initialised: false,
        })
    }

    /// Set the output format. Must be called before the first trial is written.
    pub fn set_format(&mut self, format: &OutputFormat) -> Result<()> {
        if self.initialised {
            tracing::warn!("set_format called after the first trial was written - ignored");
            return Ok(());
        }
        self.format = format.clone();
        Ok(())
    }

    /// Initialise writers on demand. Called by `write_trial` and `close`.
    fn ensure_initialised(&mut self) -> Result<()> {
        if self.initialised {
            return Ok(());
        }
        match self.format {
            OutputFormat::Csv => {
                self.writers.push(Box::new(CsvWriter::create(
                    &self.output_dir,
                    self.header.clone(),
                )?));
            }
            OutputFormat::Json => {
                self.writers.push(Box::new(JsonWriter::create(
                    &self.output_dir,
                    self.header.clone(),
                )?));
            }
            OutputFormat::Both => {
                self.writers.push(Box::new(CsvWriter::create(
                    &self.output_dir,
                    self.header.clone(),
                )?));
                self.writers.push(Box::new(JsonWriter::create(
                    &self.output_dir,
                    self.header.clone(),
                )?));
            }
        }
        self.initialised = true;
        Ok(())
    }

    pub fn write_trial(&mut self, record: &TrialRecord) -> Result<()> {
        self.ensure_initialised()?;
        for w in &mut self.writers {
            w.write_trial(record)?;
        }
        Ok(())
    }

    pub fn close(mut self) -> Result<()> {
        // Initialise even if no trials were written so the session header and
        // an empty file are always produced on Save().
        self.ensure_initialised()?;
        for w in self.writers {
            w.close()?;
        }
        Ok(())
    }
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

    /// Write column headers from the first record's field keys and lock them in.
    fn ensure_columns(&mut self, record: &TrialRecord) -> Result<()> {
        if self.columns.is_some() {
            return Ok(());
        }

        let cols: Vec<String> = record.fields.keys().cloned().collect();
        writeln!(self.writer, "{}", cols.join(",")).context("Failed to write CSV column header")?;

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
                    "Trial {} has field '{}' not present in first trial - \
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

/// Quote any CSV field that contains a comma, double-quote, or newline.
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
        let val =
            serde_json::to_value(&record.fields).context("Failed to serialize trial record")?;

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
