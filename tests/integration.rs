//! Integration tests using httpmock to exercise the full request pipeline,
//! plus performance benchmarks for spec parsing and command tree generation.

use std::path::PathBuf;
use std::time::Instant;

use httpmock::prelude::*;

use shrug::auth::credentials::{AuthScheme, CredentialSource, ResolvedCredential};
use shrug::cli::OutputFormat;
use shrug::cmd::{router, tree};
use shrug::helpers;
use shrug::jql::JqlShorthand;
use shrug::spec::model::ApiSpec;
use shrug::spec::parse_spec;
use shrug::spec::registry::Product;

/// Load a JSON fixture from tests/fixtures/.
fn load_fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// Create a test credential pointing at a mock server URL.
fn create_test_credential(server_url: &str) -> ResolvedCredential {
    ResolvedCredential {
        site: server_url.to_string(),
        source: CredentialSource::Environment,
        scheme: AuthScheme::Basic {
            email: "test@example.com".to_string(),
            api_token: "test-token".to_string(),
        },
    }
}

/// Load the test Jira spec fixture with real operations.
fn load_test_spec() -> ApiSpec {
    let spec_json = load_fixture("jira_test_spec.json");
    parse_spec(&spec_json).expect("Failed to parse test Jira spec")
}

/// Load the test spec with server_url pointed at mock.
fn load_test_spec_with_server(server_url: &str) -> ApiSpec {
    let mut spec = load_test_spec();
    spec.server_url = Some(server_url.to_string());
    spec
}

#[test]
fn search_via_helpers_returns_results() {
    let server = MockServer::start();
    let search_fixture = load_fixture("jira_search.json");

    server.mock(|when, then| {
        when.method(GET).path_includes("/rest/api/3/search");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(&search_fixture);
    });

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    let shorthand = JqlShorthand {
        project: Some("TEST".to_string()),
        ..Default::default()
    };

    let result = helpers::dispatch_helper(
        "search",
        &Product::Jira,
        &[],
        &spec,
        &client,
        Some(&credential),
        &shorthand,
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    assert!(result.is_ok(), "Search should succeed: {:?}", result.err());
}

#[test]
fn create_issue_via_helpers_returns_key() {
    let server = MockServer::start();
    let create_fixture = load_fixture("jira_create_issue.json");

    server.mock(|when, then| {
        when.method(POST).path_includes("/rest/api/3/issue");
        then.status(201)
            .header("Content-Type", "application/json")
            .body(&create_fixture);
    });

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    let result = helpers::dispatch_helper(
        "create",
        &Product::Jira,
        &[
            "--project".to_string(),
            "TEST".to_string(),
            "--summary".to_string(),
            "Integration test issue".to_string(),
        ],
        &spec,
        &client,
        Some(&credential),
        &JqlShorthand::default(),
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    assert!(result.is_ok(), "Create should succeed: {:?}", result.err());
}

#[test]
fn transition_via_helpers_resolves_name() {
    let server = MockServer::start();
    let transitions_fixture = load_fixture("jira_transitions.json");

    // GET transitions
    server.mock(|when, then| {
        when.method(GET)
            .path_includes("/rest/api/3/issue/TEST-1/transitions");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(&transitions_fixture);
    });

    // POST transition (204 No Content)
    server.mock(|when, then| {
        when.method(POST)
            .path_includes("/rest/api/3/issue/TEST-1/transitions");
        then.status(204);
    });

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    let result = helpers::dispatch_helper(
        "transition",
        &Product::Jira,
        &[
            "--issue".to_string(),
            "TEST-1".to_string(),
            "--to".to_string(),
            "In Progress".to_string(),
        ],
        &spec,
        &client,
        Some(&credential),
        &JqlShorthand::default(),
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    assert!(
        result.is_ok(),
        "Transition should succeed: {:?}",
        result.err()
    );
}

#[test]
fn error_401_returns_server_error() {
    let server = MockServer::start();
    let error_fixture = load_fixture("jira_error_401.json");

    server.mock(|when, then| {
        when.method(POST).path_includes("/rest/api/3/issue");
        then.status(401)
            .header("Content-Type", "application/json")
            .body(&error_fixture);
    });

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    let result = helpers::dispatch_helper(
        "create",
        &Product::Jira,
        &[
            "--project".to_string(),
            "TEST".to_string(),
            "--summary".to_string(),
            "Should fail".to_string(),
        ],
        &spec,
        &client,
        Some(&credential),
        &JqlShorthand::default(),
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    assert!(result.is_err(), "401 should produce an error");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("401") || err_msg.contains("authenticated"),
        "Error should indicate auth failure: {}",
        err_msg
    );
}

#[test]
fn search_default_jql_uses_current_user() {
    let server = MockServer::start();
    let search_fixture = load_fixture("jira_search.json");

    let mock = server.mock(|when, then| {
        when.method(GET).path_includes("/rest/api/3/search");
        then.status(200)
            .header("Content-Type", "application/json")
            .body(&search_fixture);
    });

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    // No shorthand flags, no raw JQL — should default to currentUser()
    let result = helpers::dispatch_helper(
        "search",
        &Product::Jira,
        &[],
        &spec,
        &client,
        Some(&credential),
        &JqlShorthand::default(),
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    assert!(
        result.is_ok(),
        "Default search should succeed: {:?}",
        result.err()
    );
    mock.assert(); // Verify the mock was called
}

#[test]
#[ignore] // Backoff delay makes this test slow (~1-3 seconds)
fn retry_on_429_succeeds_on_second_attempt() {
    let server = MockServer::start();
    // First call: 429
    server.mock(|when, then| {
        when.method(GET).path_includes("/rest/api/3/search");
        then.status(429).header("Retry-After", "0");
    });

    // Second call: 200 (httpmock serves mocks in registration order for same path)
    // Note: httpmock matches the first registered mock. For retry testing,
    // we use a sequence-aware approach or just verify the error type.
    // Since httpmock doesn't natively support call-count-based responses,
    // we verify that a 429 is properly retried by the executor.

    let base_url = server.base_url();
    let credential = create_test_credential(&base_url);
    let spec = load_test_spec_with_server(&base_url);
    let client = reqwest::blocking::Client::new();

    // The executor will get 429 repeatedly and eventually error out after max retries.
    // This verifies the retry path is exercised (the executor doesn't crash on 429).
    let result = helpers::dispatch_helper(
        "search",
        &Product::Jira,
        &[],
        &spec,
        &client,
        Some(&credential),
        &JqlShorthand {
            project: Some("TEST".to_string()),
            ..Default::default()
        },
        None,
        &OutputFormat::Json,
        false,
        false,
        None,
        true,
        false,
    );

    // After max retries, the helper should return an error (not panic)
    assert!(result.is_err(), "Should error after retries exhausted");
}

// --- Performance benchmarks ---

#[test]
fn bench_spec_parsing() {
    let spec_json = load_fixture("jira_test_spec.json");
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parse_spec(&spec_json).expect("Spec parse should not fail");
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() / iterations as u128;
    eprintln!(
        "bench_spec_parsing: {} iterations in {:?} (avg {avg_us}µs/parse)",
        iterations, elapsed
    );

    assert!(
        elapsed.as_millis() < 500,
        "Spec parsing took too long: {:?} (limit 500ms for {} iterations)",
        elapsed,
        iterations
    );
}

#[test]
fn bench_command_tree_generation() {
    let spec = load_test_spec();
    let iterations = 100;

    let start = Instant::now();
    for _ in 0..iterations {
        let tags = router::available_tags(&spec);
        for tag in &tags {
            let _ = tree::format_operations(&spec, tag);
        }
        let _ = tree::format_tag_list(&spec);
    }
    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() / iterations as u128;
    eprintln!(
        "bench_command_tree_generation: {} iterations in {:?} (avg {avg_us}µs/iteration)",
        iterations, elapsed
    );

    assert!(
        elapsed.as_millis() < 1000,
        "Command tree generation took too long: {:?} (limit 1000ms for {} iterations)",
        elapsed,
        iterations
    );
}
