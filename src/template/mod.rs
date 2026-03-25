//! Template generation for --from-json body scaffolds.
//!
//! Generates JSON files matching the exact structure each handler's
//! build_create_body / build_edit_body produces.

mod confluence;
mod jira;
mod jsw;

use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::cli::TemplateCommands;
use crate::core::error::ShrugError;

/// A registered template with its metadata and generator function.
struct TemplateEntry {
    product: &'static str,
    entity: &'static str,
    verb: &'static str,
    generate: fn() -> Value,
}

/// All available templates.
fn all_templates() -> Vec<TemplateEntry> {
    vec![
        TemplateEntry { product: "jira", entity: "issue", verb: "create", generate: jira::issue_create },
        TemplateEntry { product: "jira", entity: "issue", verb: "edit", generate: jira::issue_edit },
        TemplateEntry { product: "jira-software", entity: "board", verb: "create", generate: jsw::board_create },
        TemplateEntry { product: "jira-software", entity: "sprint", verb: "create", generate: jsw::sprint_create },
        TemplateEntry { product: "jira-software", entity: "sprint", verb: "edit", generate: jsw::sprint_edit },
        TemplateEntry { product: "confluence", entity: "space", verb: "create", generate: confluence::space_create },
        TemplateEntry { product: "confluence", entity: "space", verb: "edit", generate: confluence::space_edit },
        TemplateEntry { product: "confluence", entity: "page", verb: "create", generate: confluence::page_create },
        TemplateEntry { product: "confluence", entity: "page", verb: "edit", generate: confluence::page_edit },
        TemplateEntry { product: "confluence", entity: "blogpost", verb: "create", generate: confluence::blogpost_create },
        TemplateEntry { product: "confluence", entity: "blogpost", verb: "edit", generate: confluence::blogpost_edit },
        TemplateEntry { product: "confluence", entity: "custom-content", verb: "create", generate: confluence::custom_content_create },
        TemplateEntry { product: "confluence", entity: "custom-content", verb: "edit", generate: confluence::custom_content_edit },
    ]
}

/// Execute a template command.
pub fn execute(cmd: &TemplateCommands) -> Result<(), ShrugError> {
    match cmd {
        TemplateCommands::All { output_dir } => {
            generate_all(output_dir)
        }
        TemplateCommands::Jira { entity, verb, output_dir } => {
            generate_one("jira", entity, verb, output_dir)
        }
        TemplateCommands::JiraSoftware { entity, verb, output_dir } => {
            generate_one("jira-software", entity, verb, output_dir)
        }
        TemplateCommands::Confluence { entity, verb, output_dir } => {
            generate_one("confluence", entity, verb, output_dir)
        }
    }
}

fn generate_one(product: &str, entity: &str, verb: &str, output_dir: &str) -> Result<(), ShrugError> {
    let templates = all_templates();
    let entry = templates.iter().find(|t| {
        t.product == product && t.entity == entity && t.verb == verb
    }).ok_or_else(|| {
        let available: Vec<String> = templates.iter()
            .filter(|t| t.product == product)
            .map(|t| format!("{} {}", t.entity, t.verb))
            .collect();
        ShrugError::UsageError(format!(
            "No template for '{} {} {}'. Available for {}:\n  {}",
            product, entity, verb, product,
            if available.is_empty() { "none".to_string() } else { available.join("\n  ") }
        ))
    })?;

    let json = (entry.generate)();
    let filename = format!("{}-{}-{}.json", product, entity, verb);
    write_template(output_dir, &filename, &json)?;
    eprintln!("Generated {}/{}", output_dir, filename);
    Ok(())
}

fn generate_all(output_dir: &str) -> Result<(), ShrugError> {
    let templates = all_templates();
    fs::create_dir_all(output_dir).map_err(|e| {
        ShrugError::UsageError(format!("Failed to create output directory '{}': {}", output_dir, e))
    })?;

    for entry in &templates {
        let json = (entry.generate)();
        let filename = format!("{}-{}-{}.json", entry.product, entry.entity, entry.verb);
        write_template(output_dir, &filename, &json)?;
    }

    eprintln!("Generated {} templates in {}", templates.len(), output_dir);
    Ok(())
}

fn write_template(dir: &str, filename: &str, json: &Value) -> Result<(), ShrugError> {
    fs::create_dir_all(dir).map_err(|e| {
        ShrugError::UsageError(format!("Failed to create directory '{}': {}", dir, e))
    })?;

    let path = Path::new(dir).join(filename);
    let content = serde_json::to_string_pretty(json).map_err(|e| {
        ShrugError::UsageError(format!("Failed to serialise template: {}", e))
    })?;

    fs::write(&path, content).map_err(|e| {
        ShrugError::UsageError(format!("Failed to write '{}': {}", path.display(), e))
    })?;

    Ok(())
}
