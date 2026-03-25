use shrug::auth::credentials::ResolvedCredential;
use shrug::cli::{ColorChoice, OutputFormat};
use shrug::cmd::router;
use shrug::core::config::{ShrugConfig, ShrugPaths};
use shrug::core::error::ShrugError;
use shrug::executor;
use shrug::core::output;
use shrug::spec::registry::Product;
use shrug::spec::SpecCache;
use shrug::spec::SpecLoader;

#[allow(clippy::too_many_arguments)]
pub fn handle_product(
    product: Product,
    args: &[String],
    config: &ShrugConfig,
    client: &reqwest::blocking::Client,
    credential: Option<&ResolvedCredential>,
    dry_run: bool,
    limit: Option<u32>,
    output_format: &OutputFormat,
    color: &ColorChoice,
) -> Result<(), ShrugError> {
    let paths = ShrugPaths::new()
        .ok_or_else(|| ShrugError::SpecError("Could not determine cache directory".into()))?;
    let cache = SpecCache::new(paths.cache_dir().to_path_buf())?;
    let loader = SpecLoader::new(cache, config.cache_ttl_hours);
    let spec = loader.load(&product)?;

    let resolved = router::route_product(&product, &spec, args)?;

    let (json_body, cleaned_args) = executor::extract_json_body(&resolved.remaining_args);
    let parsed_args = executor::parse_args(&resolved.operation, &cleaned_args, json_body)?;

    let is_tty = is_terminal::is_terminal(std::io::stdout());
    let effective_format = output::resolve_format(output_format, is_tty);
    let color_enabled = output::should_use_color(color, is_tty);

    // --limit implies pagination
    let page_all = limit.is_some();

    executor::execute(
        client,
        &product,
        &resolved,
        &parsed_args,
        credential,
        dry_run,
        page_all,
        limit,
        &effective_format,
        is_tty,
        color_enabled,
        None,
    )
}
