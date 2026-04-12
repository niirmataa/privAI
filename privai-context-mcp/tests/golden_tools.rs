use std::path::PathBuf;

use privai_context_mcp::{
    models::{KnowledgeLayer, LookupRequest, RouteQuestionRequest},
    store::{LocalManifestStore, RetrievalRequest, Retriever},
    tools,
};

fn fixture_store() -> LocalManifestStore {
    LocalManifestStore::new(vec![PathBuf::from("tests/golden/v0_fixture.json")])
}

#[test]
fn registers_exactly_eight_tools() {
    assert_eq!(tools::TOOL_NAMES.len(), 8);
    assert!(tools::TOOL_NAMES.contains(&"privai_v0_lookup_direction"));
    assert!(!tools::TOOL_NAMES.contains(&"lookup_legacy"));
    assert!(!tools::TOOL_NAMES.contains(&"search_everything"));
}

#[test]
fn lookup_direction_returns_v0_only_items() {
    let store = fixture_store();
    let response = tools::lookup_direction::handle(
        &store,
        LookupRequest {
            query: "marketplace".into(),
            topic: None,
        },
    )
    .expect("lookup should succeed");

    assert!(!response.matches.is_empty());
    assert!(response
        .matches
        .iter()
        .all(|item| item.source_scope == "v0_only"));
    assert!(response.matches.iter().all(|item| !item.legacy_allowed));
}

#[test]
fn route_code_question_requires_audit() {
    let response = tools::route_question::handle(RouteQuestionRequest {
        question: "is operatorless escrow implemented in current Rust code?".into(),
    });

    assert_eq!(response.layer, KnowledgeLayer::RepoVerified);
    assert!(response.needs_code_audit);
    assert!(response.must_not_use_legacy);
}

#[test]
fn local_store_filters_by_allowed_layer() {
    let store = fixture_store();
    let items = store
        .retrieve(RetrievalRequest {
            query: "settlement".into(),
            allowed_layers: vec![KnowledgeLayer::V0Direction],
            limit: 4,
        })
        .expect("retrieve should succeed");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "v0-settlement-receipts");
}
