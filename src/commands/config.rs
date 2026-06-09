use crate::policy::load as policy_load;
use crate::state::layout::Layout;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

pub fn execute_config_verify(json: bool, section: Option<&str>, verbose: bool) -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());

    let mut success = true;
    let mut errors = Vec::new();

    if !json {
        println!("Verifying ChangeGuard configuration...");
    }

    // Verify config.toml
    let config = match crate::config::load_config(&layout) {
        Ok(cfg) => {
            if !json {
                println!("  ✅ config.toml is valid");
            }
            Some(cfg)
        }
        Err(e) => {
            if !json {
                println!("  ❌ config.toml is invalid:\n    {e}");
            }
            errors.push(format!("config.toml is invalid: {e}"));
            success = false;
            None
        }
    };

    // Verify rules.toml
    match policy_load::load_rules(&layout) {
        Ok(_) => {
            if !json {
                println!("  ✅ rules.toml is valid");
            }
        }
        Err(e) => {
            if !json {
                println!("  ❌ rules.toml is invalid:\n    {e}");
            }
            errors.push(format!("rules.toml is invalid: {e}"));
            success = false;
        }
    }

    // Report config sections
    if let (true, Some(cfg)) = (success, &config) {
        match crate::commands::config_verify::render_verify_report(cfg, json, section, verbose) {
            Ok(report) => {
                if json {
                    println!("{report}");
                } else {
                    println!("\nResolved Settings:");
                    println!("{report}");
                }
            }
            Err(e) => {
                errors.push(e.to_string());
                success = false;
            }
        }
    }

    if success {
        if !json {
            println!("\nAll configurations are valid.");
        }
        Ok(())
    } else {
        if json {
            let err_json = serde_json::json!({
                "success": false,
                "errors": errors
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&err_json).unwrap_or_default()
            );
        }
        Err(miette::miette!("Configuration verification failed."))
    }
}

pub fn execute_config_view(json: bool, section: Option<String>, key: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let config = crate::config::load_config(&layout)?;

    let mut val = serde_json::to_value(&config)
        .map_err(|e| miette::miette!("Failed to serialize config: {e}"))?;

    // Redact secret fields (api_key, token, etc.) before any output
    crate::config::redact::redact_config_value(&mut val);

    let filtered = if let Some(sec) = &section {
        let sec_key = val
            .as_object()
            .and_then(|obj| obj.keys().find(|k| k.eq_ignore_ascii_case(sec)).cloned());
        if let Some(sk) = sec_key {
            let sec_val = &val[&sk];
            if let Some(k) = &key {
                let k_key = sec_val.as_object().and_then(|obj| {
                    obj.keys()
                        .find(|inner_k| inner_k.eq_ignore_ascii_case(k))
                        .cloned()
                });
                if let Some(kk) = k_key {
                    sec_val[&kk].clone()
                } else {
                    return Err(miette::miette!("Key '{}' not found in section '{}'", k, sk));
                }
            } else {
                sec_val.clone()
            }
        } else {
            return Err(miette::miette!("Section '{}' not found in config", sec));
        }
    } else if let Some(k) = &key {
        let top_key = val.as_object().and_then(|obj| {
            obj.keys()
                .find(|inner_k| inner_k.eq_ignore_ascii_case(k))
                .cloned()
        });
        if let Some(tk) = top_key {
            val[&tk].clone()
        } else {
            return Err(miette::miette!("Key '{}' not found in top-level config", k));
        }
    } else {
        val
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&filtered)
                .map_err(|e| miette::miette!("Failed to serialize filtered config to JSON: {e}"))?
        );
    } else {
        if filtered.is_string() {
            println!("{}", filtered.as_str().unwrap());
        } else if filtered.is_number() || filtered.is_boolean() || filtered.is_null() {
            println!("{}", filtered);
        } else {
            println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
        }
    }
    Ok(())
}

pub fn execute_config_schema(json: bool) -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let storage = crate::state::storage::StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    let mut stmt = conn.prepare(
        "SELECT var_name, source_kind, required, is_secret, default_value_redacted, description, owner, environment 
         FROM env_declarations ORDER BY var_name ASC"
    ).into_diagnostic()?;

    let rows = stmt
        .query_map([], |row| {
            Ok(crate::index::env_schema::EnvDeclaration {
                var_name: row.get(0)?,
                source_kind: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(1)?))
                    .unwrap_or(crate::index::env_schema::EnvSourceKind::Config),
                required: row.get::<_, i32>(2)? != 0,
                is_secret: row.get::<_, i32>(3)? != 0,
                default_value_redacted: row.get(4)?,
                description: row.get(5)?,
                owner: row.get(6)?,
                environment: row.get(7)?,
                confidence: 1.0,
            })
        })
        .into_diagnostic()?;

    if json {
        let mut results = Vec::new();
        for row in rows {
            results.push(row.into_diagnostic()?);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&results).into_diagnostic()?
        );
    } else {
        use crate::output::table::Table;
        let mut table = Table::new();
        table.set_header(vec!["Variable", "Source", "Req", "Sec", "Default", "Owner"]);

        for row in rows {
            let d = row.into_diagnostic()?;
            table.add_row(vec![
                d.var_name,
                d.source_kind.to_string(),
                if d.required { "YES" } else { "no" }.to_string(),
                if d.is_secret { "🔒" } else { "-" }.to_string(),
                d.default_value_redacted.unwrap_or_else(|| "-".to_string()),
                d.owner.unwrap_or_else(|| "-".to_string()),
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}

pub fn execute_config_diff(json: bool) -> Result<()> {
    let current_dir = std::env::current_dir()
        .map_err(|e| miette::miette!("Failed to get current directory: {e}"))?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let storage = crate::state::storage::StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    let mut decl_stmt = conn
        .prepare("SELECT DISTINCT var_name FROM env_declarations")
        .into_diagnostic()?;
    let declared_vars: std::collections::HashSet<String> = decl_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .into_diagnostic()?
        .collect::<rusqlite::Result<std::collections::HashSet<_>>>()
        .into_diagnostic()?;

    let mut ref_stmt = conn
        .prepare("SELECT DISTINCT var_name FROM env_references")
        .into_diagnostic()?;
    let referenced_vars: std::collections::HashSet<String> = ref_stmt
        .query_map([], |row| row.get::<_, String>(0))
        .into_diagnostic()?
        .collect::<rusqlite::Result<std::collections::HashSet<_>>>()
        .into_diagnostic()?;

    let mut missing_declarations = Vec::new();
    for r_var in &referenced_vars {
        if r_var != "*" && !declared_vars.contains(r_var) {
            missing_declarations.push(r_var.clone());
        }
    }
    missing_declarations.sort();

    let mut unused_declarations = Vec::new();
    for d_var in &declared_vars {
        if !referenced_vars.contains(d_var) {
            unused_declarations.push(d_var.clone());
        }
    }
    unused_declarations.sort();

    if json {
        let res = serde_json::json!({
            "missing_declarations": missing_declarations,
            "unused_declarations": unused_declarations,
        });
        println!("{}", serde_json::to_string_pretty(&res).into_diagnostic()?);
    } else {
        println!(
            "{}",
            "Configuration Diff (Declarations vs References)"
                .bold()
                .cyan()
        );

        println!(
            "\n{}",
            "⚠️  Referenced in code but missing from declarations:"
                .yellow()
                .bold()
        );
        if missing_declarations.is_empty() {
            println!("  None");
        } else {
            for var in &missing_declarations {
                println!("  - {}", var.red());
            }
        }

        println!(
            "\n{}",
            "ℹ️  Declared but not referenced in code:".blue().bold()
        );
        if unused_declarations.is_empty() {
            println!("  None");
        } else {
            for var in &unused_declarations {
                println!("  - {}", var.dimmed());
            }
        }
    }

    Ok(())
}
