use clap::Args;
use miette::{IntoDiagnostic, Result};
use crate::commands::helpers::get_layout;
use crate::state::storage::StorageManager;
use crate::output::table::Table;

#[derive(Args, Debug)]
pub struct EndpointsArgs {
    /// Filter by method (e.g. GET, POST)
    #[arg(short, long)]
    method: Option<String>,
    /// Filter by path pattern
    #[arg(short, long)]
    path: Option<String>,
    /// Show auth details
    #[arg(long)]
    auth: bool,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

pub fn execute_endpoints(args: EndpointsArgs) -> Result<()> {
    let layout = get_layout()?;
    let storage = StorageManager::open_read_only(&layout.root)?;
    let conn = storage.get_connection();

    let mut query = String::from(
        "SELECT method, path_pattern, handler_symbol_name, framework, auth_requirements, owning_service, consumers 
         FROM api_routes WHERE 1=1"
    );
    
    if let Some(m) = &args.method {
        query.push_str(&format!(" AND method = '{}'", m.to_uppercase()));
    }
    if let Some(p) = &args.path {
        query.push_str(&format!(" AND path_pattern LIKE '%{}%'", p));
    }
    
    query.push_str(" ORDER BY path_pattern ASC");

    let mut stmt = conn.prepare(&query).into_diagnostic()?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
        ))
    }).into_diagnostic()?;

    if args.json {
        let mut results = Vec::new();
        for row in rows {
            let (method, path, handler, framework, auth, service, consumers) = row.into_diagnostic()?;
            results.push(serde_json::json!({
                "method": method,
                "path": path,
                "handler": handler,
                "framework": framework,
                "auth": auth.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
                "service": service,
                "consumers": consumers.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            }));
        }
        println!("{}", serde_json::to_string_pretty(&results).into_diagnostic()?);
    } else {
        let mut table = Table::new();
        table.set_header(vec!["Method", "Path", "Framework", "Service", "Auth"]);
        
        for row in rows {
            let (method, path, _handler, framework, auth_json, service, _consumers) = row.into_diagnostic()?;
            let auth_str = if let Some(aj) = auth_json {
                if aj == "[]" || aj == "null" { "None".to_string() } else { aj }
            } else {
                "Unknown".to_string()
            };
            
            table.add_row(vec![
                method,
                path,
                framework,
                service.unwrap_or_else(|| "-".to_string()),
                auth_str,
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}
