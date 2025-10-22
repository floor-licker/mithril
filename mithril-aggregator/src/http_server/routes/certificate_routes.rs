use warp::Filter;

use crate::http_server::routes::middlewares;
use crate::http_server::routes::router::RouterState;

/// Certificate routes configuration
///
/// # Multi-Chain Routing Strategy
///
/// This module implements an **asymmetric routing design** to support multiple blockchains
/// while maintaining full backward compatibility with existing Mithril clients.
///
/// ## Design Rationale
///
/// The aggregator previously served only Cardano certificates. To add support for Ethereum
/// and future chains without breaking existing clients, we've adopted the following strategy:
///
/// ### Legacy Routes (Backward Compatible)
/// ```text
/// GET /certificates           → Returns Cardano certificates (implicit)
/// GET /certificate/genesis    → Returns Cardano genesis certificate (implicit)
/// GET /certificate/{hash}     → Returns Cardano certificate by hash (implicit)
/// ```
///
/// These routes **default to Cardano** for backward compatibility. Existing Mithril clients
/// that call these endpoints will continue to work without any changes.
///
/// ### New Chain-Specific Routes
/// ```text
/// GET /cardano/certificates          → Returns Cardano certificates (explicit)
/// GET /cardano/certificate/genesis   → Returns Cardano genesis certificate (explicit)
/// GET /cardano/certificate/{hash}    → Returns Cardano certificate by hash (explicit)
///
/// GET /ethereum/certificates         → Returns Ethereum certificates (explicit)
/// GET /ethereum/certificate/genesis  → Returns Ethereum genesis certificate (explicit)
/// GET /ethereum/certificate/{hash}   → Returns Ethereum certificate by hash (explicit)
/// ```
///
/// New clients can use these explicit routes to request certificates from specific chains.
///
/// ## Benefits
///
/// 1. **Zero Breaking Changes**: Existing clients continue working without modification
/// 2. **Clear Semantics**: Explicit chain selection via URL path
/// 3. **Future-Proof**: Easy to add more chains (just add new routes)
/// 4. **Gradual Migration**: Clients can migrate at their own pace
/// 5. **Simple to Document**: Clear separation between legacy and multi-chain APIs
///
/// ## Implementation Note
///
/// All routes ultimately call the same handlers with different chain_type parameters.
/// The asymmetry is intentional and serves backward compatibility, not technical limitations.
pub fn routes(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    // Legacy routes (backward compatible, default to Cardano)
    legacy_certificate_routes(router_state)
        // Chain-specific routes (explicit chain selection)
        .or(cardano_certificate_routes(router_state))
        .or(ethereum_certificate_routes(router_state))
}

/// Legacy certificate routes
///
/// These routes maintain backward compatibility by defaulting to Cardano.
/// They are kept for existing Mithril clients that don't specify a chain type.
fn legacy_certificate_routes(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    legacy_certificate_certificates(router_state)
        .or(legacy_certificate_genesis(router_state))
        .or(legacy_certificate_certificate_hash(router_state))
}

/// Cardano-specific certificate routes
///
/// These routes explicitly serve Cardano certificates.
fn cardano_certificate_routes(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    cardano_certificate_certificates(router_state)
        .or(cardano_certificate_genesis(router_state))
        .or(cardano_certificate_certificate_hash(router_state))
}

/// Ethereum-specific certificate routes
///
/// These routes explicitly serve Ethereum certificates.
fn ethereum_certificate_routes(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    ethereum_certificate_certificates(router_state)
        .or(ethereum_certificate_genesis(router_state))
        .or(ethereum_certificate_certificate_hash(router_state))
}

// ============================================================================
// LEGACY ROUTES (Backward Compatible - Default to Cardano)
// ============================================================================

/// GET /certificates
///
/// Legacy endpoint that defaults to Cardano for backward compatibility.
fn legacy_certificate_certificates(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("certificates")
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string())) // Default to Cardano
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and_then(handlers::certificate_certificates)
}

/// GET /certificate/genesis
///
/// Legacy endpoint that defaults to Cardano for backward compatibility.
fn legacy_certificate_genesis(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("certificate" / "genesis")
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string())) // Default to Cardano
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_genesis)
}

/// GET /certificate/{certificate_hash}
///
/// Legacy endpoint that defaults to Cardano for backward compatibility.
fn legacy_certificate_certificate_hash(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("certificate" / String)
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string())) // Default to Cardano
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_certificate_hash)
}

// ============================================================================
// CARDANO-SPECIFIC ROUTES (Explicit Chain Selection)
// ============================================================================

/// GET /cardano/certificates
///
/// Returns Cardano certificates explicitly.
fn cardano_certificate_certificates(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("cardano" / "certificates")
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string()))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and_then(handlers::certificate_certificates)
}

/// GET /cardano/certificate/genesis
///
/// Returns Cardano genesis certificate explicitly.
fn cardano_certificate_genesis(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("cardano" / "certificate" / "genesis")
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string()))
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_genesis)
}

/// GET /cardano/certificate/{certificate_hash}
///
/// Returns Cardano certificate by hash explicitly.
fn cardano_certificate_certificate_hash(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("cardano" / "certificate" / String)
        .and(warp::get())
        .and(warp::any().map(|| "cardano".to_string()))
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_certificate_hash)
}

// ============================================================================
// ETHEREUM-SPECIFIC ROUTES (Explicit Chain Selection)
// ============================================================================

/// GET /ethereum/certificates
///
/// Returns Ethereum certificates explicitly.
fn ethereum_certificate_certificates(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("ethereum" / "certificates")
        .and(warp::get())
        .and(warp::any().map(|| "ethereum".to_string()))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and_then(handlers::certificate_certificates)
}

/// GET /ethereum/certificate/genesis
///
/// Returns Ethereum genesis certificate explicitly.
fn ethereum_certificate_genesis(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("ethereum" / "certificate" / "genesis")
        .and(warp::get())
        .and(warp::any().map(|| "ethereum".to_string()))
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_genesis)
}

/// GET /ethereum/certificate/{certificate_hash}
///
/// Returns Ethereum certificate by hash explicitly.
fn ethereum_certificate_certificate_hash(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("ethereum" / "certificate" / String)
        .and(warp::get())
        .and(warp::any().map(|| "ethereum".to_string()))
        .and(middlewares::with_client_metadata(router_state))
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_http_message_service(router_state))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::certificate_certificate_hash)
}

// ============================================================================
// HANDLERS
// ============================================================================

mod handlers {
    use slog::{Logger, warn};
    use std::convert::Infallible;
    use std::sync::Arc;
    use warp::http::StatusCode;

    use crate::MetricsService;
    use crate::http_server::routes::middlewares::ClientMetadata;
    use crate::{http_server::routes::reply, services::MessageService};

    pub const LIST_MAX_ITEMS: usize = 20;

    /// List Certificates by chain type
    ///
    /// This handler serves both legacy and chain-specific routes.
    /// The chain_type parameter is injected by the route filters.
    pub async fn certificate_certificates(
        chain_type: String,
        logger: Logger,
        http_message_service: Arc<dyn MessageService>,
    ) -> Result<impl warp::Reply, Infallible> {
        match http_message_service
            .get_certificate_list_message_by_chain(&chain_type, LIST_MAX_ITEMS)
            .await
        {
            Ok(certificates) => Ok(reply::json(&certificates, StatusCode::OK)),
            Err(err) => {
                warn!(logger,"certificate_certificates::error"; "error" => ?err, "chain_type" => &chain_type);
                Ok(reply::server_error(err))
            }
        }
    }

    /// Get genesis certificate by chain type
    ///
    /// This handler serves both legacy and chain-specific routes.
    /// The chain_type parameter is injected by the route filters.
    pub async fn certificate_genesis(
        chain_type: String,
        client_metadata: ClientMetadata,
        logger: Logger,
        http_message_service: Arc<dyn MessageService>,
        metrics_service: Arc<MetricsService>,
    ) -> Result<impl warp::Reply, Infallible> {
        metrics_service
            .get_certificate_detail_total_served_since_startup()
            .increment(&[
                client_metadata.origin_tag.as_deref().unwrap_or_default(),
                client_metadata.client_type.as_deref().unwrap_or_default(),
            ]);

        match http_message_service
            .get_latest_genesis_certificate_message_by_chain(&chain_type)
            .await
        {
            Ok(Some(certificate)) => Ok(reply::json(&certificate, StatusCode::OK)),
            Ok(None) => Ok(reply::empty(StatusCode::NOT_FOUND)),
            Err(err) => {
                warn!(logger,"certificate_certificate_genesis::error"; "error" => ?err, "chain_type" => &chain_type);
                Ok(reply::server_error(err))
            }
        }
    }

    /// Get certificate by hash and chain type
    ///
    /// This handler serves both legacy and chain-specific routes.
    /// The chain_type parameter is injected by the route filters.
    pub async fn certificate_certificate_hash(
        certificate_hash: String,
        chain_type: String,
        client_metadata: ClientMetadata,
        logger: Logger,
        http_message_service: Arc<dyn MessageService>,
        metrics_service: Arc<MetricsService>,
    ) -> Result<impl warp::Reply, Infallible> {
        metrics_service
            .get_certificate_detail_total_served_since_startup()
            .increment(&[
                client_metadata.origin_tag.as_deref().unwrap_or_default(),
                client_metadata.client_type.as_deref().unwrap_or_default(),
            ]);

        match http_message_service
            .get_certificate_message_by_chain(&certificate_hash, &chain_type)
            .await
        {
            Ok(Some(certificate)) => Ok(reply::json(&certificate, StatusCode::OK)),
            Ok(None) => Ok(reply::empty(StatusCode::NOT_FOUND)),
            Err(err) => {
                warn!(logger,"certificate_certificate_hash::error"; "error" => ?err, "chain_type" => &chain_type);
                Ok(reply::server_error(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use serde_json::Value::Null;
    use std::sync::Arc;
    use warp::{
        http::{Method, StatusCode},
        test::request,
    };

    use mithril_api_spec::APISpec;
    use mithril_common::{
        MITHRIL_CLIENT_TYPE_HEADER, MITHRIL_ORIGIN_TAG_HEADER, messages::CertificateMessage,
        test::double::fake_data,
    };

    use crate::{initialize_dependencies, services::MockMessageService};

    use super::*;

    fn setup_router(
        state: RouterState,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS]);

        warp::any().and(routes(&state).with(cors))
    }

    // ========================================================================
    // LEGACY ENDPOINT TESTS (Backward Compatibility)
    // ========================================================================

    #[tokio::test]
    async fn test_certificate_certificates_get_ok() {
        let dependency_manager = initialize_dependencies!().await;
        dependency_manager
            .certificate_repository
            .create_certificate(fake_data::genesis_certificate("{certificate_hash}"))
            .await
            .expect("certificate store save should have succeeded");

        let method = Method::GET.as_str();
        let path = "/certificates";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::OK,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_when_error_retrieving_certificates_returns_ko_500() {
        let mut dependency_manager = initialize_dependencies!().await;
        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_certificate_list_message_by_chain()
            .returning(|_, _| Err(anyhow!("an error")));
        dependency_manager.message_service = Arc::new(message_service);

        let method = Method::GET.as_str();
        let path = "/certificates";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::INTERNAL_SERVER_ERROR,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_certificate_hash_increments_certificate_detail_total_served_since_startup_metric()
     {
        let method = Method::GET.as_str();
        let path = "/certificate/{certificate_hash}";

        let dependency_manager = Arc::new(initialize_dependencies!().await);
        let metric = dependency_manager
            .metrics_service
            .get_certificate_detail_total_served_since_startup();
        let initial_counter_value_with_tag = metric.get(&["TEST", "CLI"]);
        let initial_counter_value_with_unknown_tags =
            metric.get(&["UNKNOWN_ORIGIN", "UNKNOWN_CLIENT_TYPE"]);

        request()
            .method(method)
            .path(path)
            .header(MITHRIL_ORIGIN_TAG_HEADER, "TEST")
            .header(MITHRIL_CLIENT_TYPE_HEADER, "CLI")
            .reply(&setup_router(RouterState::new_with_origin_tag_white_list(
                dependency_manager.clone(),
                &["TEST"],
            )))
            .await;

        assert_eq!(
            initial_counter_value_with_tag + 1,
            metric.get(&["TEST", "CLI"])
        );
        assert_eq!(
            initial_counter_value_with_unknown_tags,
            metric.get(&["UNKNOWN_ORIGIN", "UNKNOWN_CLIENT_TYPE"])
        );
    }

    #[tokio::test]
    async fn test_certificate_certificate_hash_get_ok() {
        let dependency_manager = initialize_dependencies!().await;
        dependency_manager
            .certificate_repository
            .create_certificate(fake_data::genesis_certificate("{certificate_hash}"))
            .await
            .expect("certificate store save should have succeeded");

        let method = Method::GET.as_str();
        let path = "/certificate/{certificate_hash}";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::OK,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_certificate_hash_get_ko_404() {
        let dependency_manager = initialize_dependencies!().await;

        let method = Method::GET.as_str();
        let path = "/certificate/{certificate_hash}";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::NOT_FOUND,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_when_error_on_retrieving_certificate_hash_returns_ko_500() {
        let mut dependency_manager = initialize_dependencies!().await;
        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_certificate_message_by_chain()
            .returning(|_, _| Err(anyhow!("an error")));
        dependency_manager.message_service = Arc::new(message_service);

        let method = Method::GET.as_str();
        let path = "/certificate/{certificate_hash}";

        let response = request()
            .method(method)
            .path(&path.replace("{certificate_hash}", "whatever"))
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::INTERNAL_SERVER_ERROR,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_genesis_increments_certificate_detail_total_served_since_startup_metric()
     {
        let method = Method::GET.as_str();
        let path = "/certificate/genesis";

        let dependency_manager = Arc::new(initialize_dependencies!().await);
        let metric = dependency_manager
            .metrics_service
            .get_certificate_detail_total_served_since_startup();
        let initial_counter_value_with_tag = metric.get(&["TEST", "CLI"]);
        let initial_counter_value_with_unknown_tags =
            metric.get(&["UNKNOWN_ORIGIN", "UNKNOWN_CLIENT_TYPE"]);

        request()
            .method(method)
            .path(path)
            .header(MITHRIL_ORIGIN_TAG_HEADER, "TEST")
            .header(MITHRIL_CLIENT_TYPE_HEADER, "CLI")
            .reply(&setup_router(RouterState::new_with_origin_tag_white_list(
                dependency_manager.clone(),
                &["TEST"],
            )))
            .await;

        assert_eq!(
            initial_counter_value_with_tag + 1,
            metric.get(&["TEST", "CLI"])
        );
        assert_eq!(
            initial_counter_value_with_unknown_tags,
            metric.get(&["UNKNOWN_ORIGIN", "UNKNOWN_CLIENT_TYPE"])
        );
    }

    #[tokio::test]
    async fn test_certificate_genesis_get_ok() {
        let dependency_manager = initialize_dependencies!().await;
        dependency_manager
            .certificate_repository
            .create_certificate(fake_data::genesis_certificate("certificate_genesis_1"))
            .await
            .expect("certificate store save should have succeeded");
        dependency_manager
            .certificate_repository
            .create_certificate(fake_data::genesis_certificate("certificate_genesis_2"))
            .await
            .expect("certificate store save should have succeeded");

        let method = Method::GET.as_str();
        let path = "/certificate/genesis";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::OK,
        )
        .unwrap();

        let returned_genesis: CertificateMessage = serde_json::from_slice(response.body()).unwrap();
        assert_eq!("certificate_genesis_2", &returned_genesis.hash);
    }

    #[tokio::test]
    async fn test_certificate_genesis_get_ko_404() {
        let dependency_manager = initialize_dependencies!().await;

        let method = Method::GET.as_str();
        let path = "/certificate/genesis";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::NOT_FOUND,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_certificate_when_error_on_retrieving_certificate_genesis_returns_ko_500() {
        let mut dependency_manager = initialize_dependencies!().await;
        let mut message_service = MockMessageService::new();
        message_service
            .expect_get_latest_genesis_certificate_message_by_chain()
            .returning(|_| Err(anyhow!("an error")));
        dependency_manager.message_service = Arc::new(message_service);

        let method = Method::GET.as_str();
        let path = "/certificate/genesis";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &Null,
            &response,
            &StatusCode::INTERNAL_SERVER_ERROR,
        )
        .unwrap();
    }

    // ========================================================================
    // CHAIN-SPECIFIC ENDPOINT TESTS (Multi-Chain Support)
    // ========================================================================

    #[tokio::test]
    async fn test_cardano_certificates_get_ok() {
        let dependency_manager = initialize_dependencies!().await;
        dependency_manager
            .certificate_repository
            .create_certificate(fake_data::genesis_certificate("{certificate_hash}"))
            .await
            .expect("certificate store save should have succeeded");

        let method = Method::GET.as_str();
        let path = "/cardano/certificates";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ethereum_certificates_get_ok() {
        let dependency_manager = initialize_dependencies!().await;

        let method = Method::GET.as_str();
        let path = "/ethereum/certificates";

        let response = request()
            .method(method)
            .path(path)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        // Returns OK with empty list (no Ethereum certificates yet)
        assert_eq!(response.status(), StatusCode::OK);
    }
}
