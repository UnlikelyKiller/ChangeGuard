use crate::policy::load as policy_load;
use crate::state::layout::Layout;
use miette::Result;

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

    let val = serde_json::to_value(&config)
        .map_err(|e| miette::miette!("Failed to serialize config: {e}"))?;

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
