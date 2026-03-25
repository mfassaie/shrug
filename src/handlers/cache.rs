use shrug::cli::CacheCommands;
use shrug::core::config::{ShrugConfig, ShrugPaths};
use shrug::core::error::ShrugError;
use shrug::spec::registry::Product;
use shrug::spec::SpecCache;
use shrug::spec::SpecLoader;

pub fn handle_cache(command: &CacheCommands, config: &ShrugConfig) -> Result<(), ShrugError> {
    let paths = ShrugPaths::new()
        .ok_or_else(|| ShrugError::SpecError("Could not determine cache directory".into()))?;
    let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;

    match command {
        CacheCommands::List => {
            let keys = cache.list_cached();
            if keys.is_empty() {
                println!("No cached specs. Run `shrug cache refresh` to download.");
                return Ok(());
            }
            let header = format!(
                "{:<20} {:<12} {:<20} {}",
                "PRODUCT", "VERSION", "CACHED", "STATUS"
            );
            println!("{header}");
            println!("{}", "-".repeat(60));
            for key in &keys {
                match cache.load_metadata(key) {
                    Ok(Some(meta)) => {
                        let age = chrono::Utc::now() - meta.cached_at;
                        let age_str = if age.num_hours() < 1 {
                            format!("{}m ago", age.num_minutes())
                        } else if age.num_hours() < 48 {
                            format!("{}h ago", age.num_hours())
                        } else {
                            format!("{}d ago", age.num_days())
                        };
                        let status = if age.num_hours() < config.cache_ttl_hours as i64 {
                            "fresh"
                        } else {
                            "stale"
                        };
                        println!(
                            "{:<20} {:<12} {:<20} {}",
                            key, meta.spec_version, age_str, status
                        );
                    }
                    _ => {
                        let line = format!("{:<20} {:<12} {:<20} {}", key, "?", "?", "unknown");
                        println!("{line}");
                    }
                }
            }
            return Ok(());
        }
        CacheCommands::Clear { product: None } => {
            let keys = cache.list_cached();
            let mut count = 0;
            for key in &keys {
                let _ = cache.invalidate(key);
                let _ = cache.invalidate_binary(key);
                count += 1;
            }
            println!("Cleared {} cached spec(s).", count);
            return Ok(());
        }
        CacheCommands::Clear {
            product: Some(name),
        } => {
            let product = Product::from_cli_prefix(name).ok_or_else(|| {
                ShrugError::UsageError(format!(
                    "Unknown product '{}'. Valid products: jira, jira-software, confluence",
                    name
                ))
            })?;
            let cache_key = product.info().cache_key;
            cache.invalidate(cache_key)?;
            let _ = cache.invalidate_binary(cache_key);
            println!("Cleared cache for {}.", product.info().display_name);
            return Ok(());
        }
        CacheCommands::Refresh { product: None } => {
            let loader = SpecLoader::new(cache, config.cache_ttl_hours);
            let results = loader.refresh_all();
            let mut ok_count = 0;
            let mut err_count = 0;
            for (product, result) in &results {
                match result {
                    Ok(spec) => {
                        println!(
                            "  {} — {} operations",
                            product.info().display_name,
                            spec.operations.len()
                        );
                        ok_count += 1;
                    }
                    Err(e) => {
                        eprintln!("  {} — failed: {}", product.info().display_name, e);
                        err_count += 1;
                    }
                }
            }
            println!("\nRefreshed {} specs ({} failed).", ok_count, err_count);
            if err_count > 0 {
                return Err(ShrugError::SpecError(format!(
                    "{} spec(s) failed to refresh",
                    err_count
                )));
            }
        }
        CacheCommands::Refresh {
            product: Some(name),
        } => {
            let loader = SpecLoader::new(cache, config.cache_ttl_hours);
            let product = Product::from_cli_prefix(name).ok_or_else(|| {
                ShrugError::UsageError(format!(
                    "Unknown product '{}'. Valid products: jira, jira-software, confluence",
                    name
                ))
            })?;
            let spec = loader.refresh(&product)?;
            println!(
                "{} — {} operations",
                product.info().display_name,
                spec.operations.len()
            );
        }
    }

    Ok(())
}
