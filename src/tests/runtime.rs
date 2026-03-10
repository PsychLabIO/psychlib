use crate::renderer::RenderConfig;
#[allow(unused_imports)]
use crate::runtime::ExperimentConfig;

#[allow(dead_code)]
fn config(dir: &std::path::Path, script: &str, seed: u64) -> ExperimentConfig {
    let script_path = dir.join("experiment.lua");
    std::fs::write(&script_path, script).unwrap();

    ExperimentConfig {
        participant: "TEST".into(),
        script_path,
        output_dir: dir.join("data"),
        seed: Some(seed),
        render: RenderConfig::default(),
    }
}

#[allow(dead_code)]
fn find_csv(data_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    std::fs::read_dir(data_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |x| x == "csv"))
        .map(|e| e.path())
}