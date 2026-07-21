use lingonberry_core::StorageBackend;
use lingonberry_protocol::{
    build_multi_node_policy_manifest, finalize_knowledge_object, read_json_file, to_canonical_json,
    validate_knowledge_object, validate_publish_request, JsonValue,
};
use lingonberry_storage::{
    build_storage_backend_at, run_storage_doctor, runtime_storage_config_with_overrides,
    runtime_storage_layout, DoctorCheck, DoctorReport, StorageRuntimeConfig,
    StorageRuntimeConfigOverrides, STORAGE_CONFIG_PRECEDENCE,
};
use std::collections::BTreeMap;
use std::env;
use std::process;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{}", error);
        process::exit(exit_code_for_error(&error));
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let invocation = parse_invocation(args)?;
    let config = runtime_storage_config_with_overrides(&invocation.overrides)?;
    let backend = build_storage_backend_at(&config.data_dir);

    match invocation.command.as_str() {
        "capabilities" => {
            println!(
                "{}",
                to_canonical_json(&json_object(vec![
                    ("status", JsonValue::String("ok".to_string())),
                    ("service", JsonValue::String("storage".to_string())),
                    (
                        "operations",
                        JsonValue::Array(vec![
                            JsonValue::String("append".to_string()),
                            JsonValue::String("retrieve".to_string()),
                            JsonValue::String("replay".to_string()),
                            JsonValue::String("list".to_string()),
                            JsonValue::String("config".to_string()),
                            JsonValue::String("status".to_string()),
                            JsonValue::String("doctor".to_string()),
                            JsonValue::String("verify".to_string()),
                            JsonValue::String("health".to_string()),
                            JsonValue::String("ready".to_string()),
                            JsonValue::String("metrics".to_string()),
                            JsonValue::String("run".to_string()),
                        ]),
                    ),
                    ("multiNode", build_multi_node_policy_manifest()),
                ]))
            );
            Ok(())
        }
        "config" => {
            print_config(&config);
            Ok(())
        }
        "status" => {
            print_operator_status(&config);
            Ok(())
        }
        "doctor" => handle_doctor(&config, false),
        "verify" => handle_doctor(&config, true),
        "health" => {
            print_health();
            Ok(())
        }
        "ready" => handle_readiness(&config),
        "metrics" => {
            print_metrics(&config);
            Ok(())
        }
        "run" => {
            print_runtime_status(&config);
            Ok(())
        }
        "append" => {
            let pathname = invocation.command_args.first().ok_or_else(|| {
                "usage: lingonberry-storage [options] append <json-file>".to_string()
            })?;
            handle_append(pathname, &backend)
        }
        "retrieve" => {
            let canonical_id = invocation.command_args.first().ok_or_else(|| {
                "usage: lingonberry-storage [options] retrieve <canonical-id>".to_string()
            })?;
            handle_retrieve(canonical_id, &backend)
        }
        "replay" => handle_replay(&backend),
        "list" => handle_list(&backend),
        command => Err(format!("unknown command: {command}")),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedInvocation {
    command: String,
    command_args: Vec<String>,
    overrides: StorageRuntimeConfigOverrides,
}

fn parse_invocation(args: Vec<String>) -> Result<ParsedInvocation, String> {
    let mut overrides = StorageRuntimeConfigOverrides::default();
    let mut index = 0;
    while let Some(argument) = args.get(index) {
        if argument == "--" {
            index += 1;
            break;
        }
        if !argument.starts_with("--") {
            break;
        }
        let target = match argument.as_str() {
            "--config" => &mut overrides.config_path,
            "--state-dir" => &mut overrides.state_dir,
            "--data-dir" => &mut overrides.data_dir,
            "--backup-dir" => &mut overrides.backup_dir,
            "--temp-dir" => &mut overrides.temp_dir,
            _ => return Err(format!("usage: unknown global option: {argument}")),
        };
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("usage: global option {argument} requires a value"))?;
        if value.is_empty() || value.starts_with("--") {
            return Err(format!("usage: global option {argument} requires a value"));
        }
        *target = Some(value.into());
        index += 2;
    }
    let command = args.get(index).cloned().ok_or_else(|| {
        "usage: lingonberry-storage [--config PATH] [--state-dir PATH] [--data-dir PATH] [--backup-dir PATH] [--temp-dir PATH] <command>".to_string()
    })?;
    Ok(ParsedInvocation {
        command,
        command_args: args[index + 1..].to_vec(),
        overrides,
    })
}

fn print_health() {
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("service", JsonValue::String("storage".to_string())),
            ("status", JsonValue::String("ok".to_string())),
            ("scope", JsonValue::String("process".to_string())),
        ]))
    );
}

fn handle_readiness(config: &StorageRuntimeConfig) -> Result<(), String> {
    let report = run_storage_doctor(config);
    let ready = !report.has_failures();
    let status = if ready { "ready" } else { "not_ready" };
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("service", JsonValue::String("storage".to_string())),
            ("status", JsonValue::String(status.to_string())),
            ("ready", JsonValue::Bool(ready)),
            (
                "diagnosticStatus",
                JsonValue::String(report.severity.as_str().to_string()),
            ),
        ]))
    );
    if ready {
        Ok(())
    } else {
        Err("readiness detected failed checks".to_string())
    }
}

fn print_metrics(config: &StorageRuntimeConfig) {
    let report = run_storage_doctor(config);
    let ok = report
        .checks
        .iter()
        .filter(|check| check.severity.as_str() == "ok")
        .count();
    let warning = report
        .checks
        .iter()
        .filter(|check| check.severity.as_str() == "warning")
        .count();
    let failed = report
        .checks
        .iter()
        .filter(|check| check.severity.as_str() == "failed")
        .count();
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("service", JsonValue::String("storage".to_string())),
            ("metricsVersion", JsonValue::String("1".to_string())),
            ("boundedCardinality", JsonValue::Bool(true)),
            (
                "ready",
                JsonValue::Number((!report.has_failures() as usize).to_string())
            ),
            ("doctorChecksOk", JsonValue::Number(ok.to_string())),
            (
                "doctorChecksWarning",
                JsonValue::Number(warning.to_string())
            ),
            ("doctorChecksFailed", JsonValue::Number(failed.to_string())),
        ]))
    );
}

fn handle_doctor(config: &StorageRuntimeConfig, strict: bool) -> Result<(), String> {
    let report = run_storage_doctor(config);
    println!("{}", to_canonical_json(&doctor_report_json(&report)));
    if report.has_failures() {
        return Err("doctor detected failed checks".to_string());
    }
    if strict && report.severity.as_str() != "ok" {
        return Err("verify detected warning checks".to_string());
    }
    Ok(())
}

fn doctor_report_json(report: &DoctorReport) -> JsonValue {
    json_object(vec![
        (
            "status",
            JsonValue::String(report.severity.as_str().to_string()),
        ),
        ("readOnly", JsonValue::Bool(true)),
        (
            "checkCount",
            JsonValue::Number(report.checks.len().to_string()),
        ),
        (
            "checks",
            JsonValue::Array(report.checks.iter().map(doctor_check_json).collect()),
        ),
    ])
}

fn doctor_check_json(check: &DoctorCheck) -> JsonValue {
    json_object(vec![
        ("name", JsonValue::String(check.name.to_string())),
        (
            "status",
            JsonValue::String(check.severity.as_str().to_string()),
        ),
        ("code", JsonValue::String(check.code.to_string())),
        ("message", JsonValue::String(check.message.clone())),
    ])
}

fn handle_append(pathname: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let loaded = read_json_file(pathname)?;
    let errors = validate_publish_request(&loaded.value);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }
    let Some(request_map) = as_object(&loaded.value) else {
        return Err("publish request must be an object".to_string());
    };
    let Some(object) = request_map.get("object") else {
        return Err("publish request missing object".to_string());
    };
    let object_errors = validate_knowledge_object(object);
    if !object_errors.is_empty() {
        return Err(object_errors.join("; "));
    }
    let finalized = finalize_knowledge_object(object).map_err(|errors| errors.join("; "))?;
    let outcome = backend
        .append_publish_request(&loaded.raw, &finalized)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("canonicalId", JsonValue::String(outcome.canonical_id)),
            (
                "carrierIdentity",
                JsonValue::String(outcome.carrier_identity)
            ),
            ("duplicate", JsonValue::Bool(outcome.duplicate)),
            (
                "storedAt",
                match outcome.stored_at {
                    Some(value) => JsonValue::String(value),
                    None => JsonValue::Null,
                },
            ),
            ("object", outcome.object),
        ]))
    );
    Ok(())
}

fn handle_retrieve(canonical_id: &str, backend: &impl StorageBackend) -> Result<(), String> {
    let record = backend
        .get(canonical_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("object not found: {}", canonical_id))?;
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("canonicalId", JsonValue::String(record.canonical_id)),
            (
                "carrierIdentity",
                JsonValue::String(record.carrier_identity)
            ),
            ("storedAt", JsonValue::String(record.stored_at)),
            ("object", record.object),
        ]))
    );
    Ok(())
}

fn handle_replay(backend: &impl StorageBackend) -> Result<(), String> {
    let records = backend.replay().map_err(|error| error.to_string())?;
    let objects: Vec<JsonValue> = records
        .into_iter()
        .map(|record| {
            json_object(vec![
                ("canonicalId", JsonValue::String(record.canonical_id)),
                (
                    "carrierIdentity",
                    JsonValue::String(record.carrier_identity),
                ),
                ("storedAt", JsonValue::String(record.stored_at)),
                ("object", record.object),
            ])
        })
        .collect();
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("count", JsonValue::Number(objects.len().to_string())),
            ("objects", JsonValue::Array(objects)),
        ]))
    );
    Ok(())
}

fn handle_list(backend: &impl StorageBackend) -> Result<(), String> {
    let ids = backend.list_ids().map_err(|error| error.to_string())?;
    println!(
        "{}",
        to_canonical_json(&json_object(vec![(
            "ids",
            JsonValue::Array(ids.into_iter().map(JsonValue::String).collect())
        ),]))
    );
    Ok(())
}

fn as_object(value: &JsonValue) -> Option<&BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(map) => Some(map),
        _ => None,
    }
}

fn json_object(entries: Vec<(&str, JsonValue)>) -> JsonValue {
    let mut map = BTreeMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    JsonValue::Object(map)
}

fn print_config(config: &StorageRuntimeConfig) {
    let layout = runtime_storage_layout(config);
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("containsSecrets", JsonValue::Bool(false)),
            (
                "precedence",
                JsonValue::Array(
                    STORAGE_CONFIG_PRECEDENCE
                        .iter()
                        .map(|value| JsonValue::String((*value).to_string()))
                        .collect(),
                ),
            ),
            ("configPath", path_value(config.config_path.as_ref())),
            (
                "stateDir",
                JsonValue::String(config.state_dir.to_string_lossy().to_string())
            ),
            (
                "dataDir",
                JsonValue::String(config.data_dir.to_string_lossy().to_string())
            ),
            (
                "backupDir",
                JsonValue::String(config.backup_dir.to_string_lossy().to_string())
            ),
            (
                "tempDir",
                JsonValue::String(config.temp_dir.to_string_lossy().to_string())
            ),
            (
                "rawLogPath",
                JsonValue::String(layout.raw_log_path.to_string_lossy().to_string())
            ),
            (
                "catalogPath",
                JsonValue::String(layout.catalog_path.to_string_lossy().to_string())
            ),
        ]))
    );
}

fn print_operator_status(config: &StorageRuntimeConfig) {
    let report = run_storage_doctor(config);
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("service", JsonValue::String("storage".to_string())),
            (
                "status",
                JsonValue::String(report.severity.as_str().to_string())
            ),
            ("readOnly", JsonValue::Bool(true)),
            (
                "checkCount",
                JsonValue::Number(report.checks.len().to_string())
            ),
        ]))
    );
}

fn print_runtime_status(config: &StorageRuntimeConfig) {
    let layout = runtime_storage_layout(config);
    println!(
        "{}",
        to_canonical_json(&json_object(vec![
            ("status", JsonValue::String("ok".to_string())),
            ("service", JsonValue::String("storage".to_string())),
            ("configPath", path_value(config.config_path.as_ref())),
            (
                "stateDir",
                JsonValue::String(config.state_dir.to_string_lossy().to_string())
            ),
            (
                "dataDir",
                JsonValue::String(config.data_dir.to_string_lossy().to_string())
            ),
            (
                "backupDir",
                JsonValue::String(config.backup_dir.to_string_lossy().to_string())
            ),
            (
                "tempDir",
                JsonValue::String(config.temp_dir.to_string_lossy().to_string())
            ),
            (
                "rawLogPath",
                JsonValue::String(layout.raw_log_path.to_string_lossy().to_string())
            ),
            (
                "catalogPath",
                JsonValue::String(layout.catalog_path.to_string_lossy().to_string())
            ),
        ]))
    );
}

fn path_value(path: Option<&std::path::PathBuf>) -> JsonValue {
    match path {
        Some(path) => JsonValue::String(path.to_string_lossy().to_string()),
        None => JsonValue::Null,
    }
}

fn exit_code_for_error(error: &str) -> i32 {
    if error.starts_with("usage:") {
        64
    } else if error.contains("not found") {
        66
    } else if error.contains("doctor detected")
        || error.contains("verify detected")
        || error.contains("readiness detected")
    {
        69
    } else if error.contains("config") || error.contains("failed to bind") {
        78
    } else if error.contains("validation failed") {
        65
    } else if error.contains("LB_") {
        70
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_options_are_parsed_before_the_command() {
        let parsed = parse_invocation(vec![
            "--state-dir".to_string(),
            "/state".to_string(),
            "--data-dir".to_string(),
            "/data".to_string(),
            "doctor".to_string(),
        ])
        .expect("parse invocation");
        assert_eq!(parsed.command, "doctor");
        assert_eq!(parsed.overrides.state_dir, Some("/state".into()));
        assert_eq!(parsed.overrides.data_dir, Some("/data".into()));
    }

    #[test]
    fn unknown_global_option_is_rejected() {
        let error = parse_invocation(vec![
            "--repair".to_string(),
            "now".to_string(),
            "doctor".to_string(),
        ])
        .expect_err("unknown option must fail");
        assert!(error.contains("unknown global option"));
    }
}
