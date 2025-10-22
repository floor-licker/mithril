use crate::http_server::routes::middlewares;
use crate::http_server::routes::router::RouterState;
use warp::Filter;

/// Signature registration routes configuration
///
/// # Multi-Chain Signature Routing Strategy
///
/// This module implements **chain-specific signature registration** to prevent cross-contamination
/// of signatures from different blockchains. This is CRITICAL for security - mixing Cardano and
/// Ethereum signatures would result in invalid multi-signatures.
///
/// ## Design Rationale
///
/// Signatures must be kept strictly separated by chain because:
/// 1. **Different Stake Distributions**: Cardano SPO stakes ≠ Ethereum validator stakes
/// 2. **Different Cryptographic Contexts**: Signatures are over different messages
/// 3. **Invalid Aggregation**: Mixing would create cryptographically invalid certificates
///
/// ## Routing Strategy (Asymmetric for Backward Compatibility)
///
/// ### Legacy Endpoint (Backward Compatible)
/// ```text
/// POST /register-signatures   → Registers Cardano signature (implicit)
/// ```
///
/// This defaults to Cardano for backward compatibility with existing Mithril signers.
///
/// ### Chain-Specific Endpoints (Explicit)
/// ```text
/// POST /cardano/register-signatures    → Registers Cardano signature (explicit)
/// POST /ethereum/register-signatures   → Registers Ethereum signature (explicit)
/// ```
///
/// New signers should use these explicit endpoints to clearly indicate which chain they're signing for.
///
/// ## Security Implications
///
/// **CRITICAL**: The chain_type parameter determines which signature pool the signature goes into.
/// This prevents the following attack/error scenarios:
/// - Cardano signer accidentally sending to Ethereum endpoint
/// - Ethereum signatures being aggregated with Cardano stakes
/// - Cross-chain signature replay attacks
///
/// ## Implementation Note
///
/// All routes call the same handler with different chain_type parameters. The asymmetry
/// is intentional for backward compatibility, not a technical limitation.
pub fn routes(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    // Legacy route (backward compatible, defaults to Cardano)
    legacy_register_signatures(router_state)
        // Chain-specific routes (explicit chain selection)
        .or(cardano_register_signatures(router_state))
        .or(ethereum_register_signatures(router_state))
}

/// POST /register-signatures
///
/// Legacy endpoint that defaults to Cardano for backward compatibility.
/// Existing Mithril signers use this endpoint and expect Cardano behavior.
fn legacy_register_signatures(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("register-signatures")
        .and(warp::post())
        .and(warp::any().map(|| "cardano".to_string())) // Default to Cardano
        .and(warp::body::json())
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_certifier_service(router_state))
        .and(middlewares::with_single_signature_authenticator(
            router_state,
        ))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::register_signatures)
}

/// POST /cardano/register-signatures
///
/// Explicit Cardano signature registration endpoint.
fn cardano_register_signatures(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("cardano" / "register-signatures")
        .and(warp::post())
        .and(warp::any().map(|| "cardano".to_string()))
        .and(warp::body::json())
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_certifier_service(router_state))
        .and(middlewares::with_single_signature_authenticator(
            router_state,
        ))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::register_signatures)
}

/// POST /ethereum/register-signatures
///
/// Explicit Ethereum signature registration endpoint.
fn ethereum_register_signatures(
    router_state: &RouterState,
) -> impl Filter<Extract = (impl warp::Reply + use<>,), Error = warp::Rejection> + Clone + use<> {
    warp::path!("ethereum" / "register-signatures")
        .and(warp::post())
        .and(warp::any().map(|| "ethereum".to_string()))
        .and(warp::body::json())
        .and(middlewares::with_logger(router_state))
        .and(middlewares::with_certifier_service(router_state))
        .and(middlewares::with_single_signature_authenticator(
            router_state,
        ))
        .and(middlewares::with_metrics_service(router_state))
        .and_then(handlers::register_signatures)
}

mod handlers {
    use slog::{Logger, debug, warn};
    use std::convert::Infallible;
    use std::sync::Arc;
    use warp::http::StatusCode;

    use mithril_common::messages::{RegisterSignatureMessageHttp, TryFromMessageAdapter};

    use crate::{
        MetricsService, SingleSignatureAuthenticator,
        http_server::routes::reply,
        message_adapters::FromRegisterSingleSignatureAdapter,
        services::{CertifierService, CertifierServiceError, SignatureRegistrationStatus},
        unwrap_to_internal_server_error,
    };

    const METRICS_HTTP_ORIGIN: &str = "HTTP";

    /// Register Signatures with chain awareness
    ///
    /// This handler serves both legacy and chain-specific routes.
    /// The chain_type parameter is injected by the route filters.
    ///
    /// **CRITICAL**: The chain_type determines which signature pool this signature goes into.
    /// Mixing Cardano and Ethereum signatures would create invalid certificates.
    pub async fn register_signatures(
        chain_type: String,
        message: RegisterSignatureMessageHttp,
        logger: Logger,
        certifier_service: Arc<dyn CertifierService>,
        single_signer_authenticator: Arc<SingleSignatureAuthenticator>,
        metrics_service: Arc<MetricsService>,
    ) -> Result<impl warp::Reply, Infallible> {
        debug!(logger, ">> register_signatures"; "payload" => ?message, "chain_type" => &chain_type);

        metrics_service
            .get_signature_registration_total_received_since_startup()
            .increment(&[METRICS_HTTP_ORIGIN]);

        let signed_entity_type = message.signed_entity_type.clone();
        let signed_message = message.signed_message.clone();

        let mut single_signature = match FromRegisterSingleSignatureAdapter::try_adapt(message) {
            Ok(signature) => signature,
            Err(err) => {
                warn!(logger,"register_signatures::payload decoding error"; "error" => ?err, "chain_type" => &chain_type);

                return Ok(reply::bad_request(
                    "Could not decode signature payload".to_string(),
                    err.to_string(),
                ));
            }
        };

        unwrap_to_internal_server_error!(
            single_signer_authenticator
                .authenticate(&mut single_signature, &signed_message)
                .await,
            logger => "single_signer_authenticator::error"
        );

        if !single_signature.is_authenticated() {
            debug!(logger, "register_signatures::unauthenticated_signature"; "chain_type" => &chain_type);
            return Ok(reply::bad_request(
                "Could not authenticate signature".to_string(),
                "Signature could not be authenticated".to_string(),
            ));
        }

        // Register signature with chain type to keep pools separate
        match certifier_service
            .register_single_signature(&signed_entity_type, &single_signature, &chain_type)
            .await
        {
            Err(err) => match err.downcast_ref::<CertifierServiceError>() {
                Some(CertifierServiceError::AlreadyCertified(signed_entity_type)) => {
                    debug!(logger,"register_signatures::open_message_already_certified"; "signed_entity_type" => ?signed_entity_type, "chain_type" => &chain_type);
                    Ok(reply::gone(
                        "already_certified".to_string(),
                        err.to_string(),
                    ))
                }
                Some(CertifierServiceError::Expired(signed_entity_type)) => {
                    debug!(logger,"register_signatures::open_message_expired"; "signed_entity_type" => ?signed_entity_type, "chain_type" => &chain_type);
                    Ok(reply::gone("expired".to_string(), err.to_string()))
                }
                Some(CertifierServiceError::NotFound(signed_entity_type)) => {
                    debug!(logger,"register_signatures::not_found"; "signed_entity_type" => ?signed_entity_type, "chain_type" => &chain_type);
                    Ok(reply::empty(StatusCode::NOT_FOUND))
                }
                Some(_) | None => {
                    warn!(logger,"register_signatures::error"; "error" => ?err, "chain_type" => &chain_type);
                    Ok(reply::server_error(err))
                }
            },
            Ok(SignatureRegistrationStatus::Registered) => Ok(reply::empty(StatusCode::CREATED)),
            Ok(SignatureRegistrationStatus::Buffered) => Ok(reply::empty(StatusCode::ACCEPTED)),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use mithril_common::entities::ClientError;
    use std::sync::Arc;
    use warp::http::{Method, StatusCode};
    use warp::test::request;

    use mithril_api_spec::APISpec;
    use mithril_common::{
        entities::SignedEntityType, messages::RegisterSignatureMessageHttp, test::double::Dummy,
    };

    use crate::{
        SingleSignatureAuthenticator, initialize_dependencies,
        services::{CertifierServiceError, MockCertifierService, SignatureRegistrationStatus},
    };

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
    async fn test_register_signatures_increments_signature_registration_total_received_since_startup_metric()
     {
        let method = Method::POST.as_str();
        let path = "/register-signatures";
        let dependency_manager = Arc::new(initialize_dependencies!().await);
        let initial_counter_value = dependency_manager
            .metrics_service
            .get_signature_registration_total_received_since_startup()
            .get(&["HTTP"]);

        request()
            .method(method)
            .path(path)
            .json(&RegisterSignatureMessageHttp::dummy())
            .reply(&setup_router(RouterState::new_with_dummy_config(
                dependency_manager.clone(),
            )))
            .await;

        assert_eq!(
            initial_counter_value + 1,
            dependency_manager
                .metrics_service
                .get_signature_registration_total_received_since_startup()
                .get(&["HTTP"])
        );
    }

    #[tokio::test]
    async fn test_register_signatures_try_to_authenticate_signature_with_signed_message() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .withf(|_, signature, _| signature.is_authenticated())
            .once()
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Registered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp {
            signed_message: "message".to_string(),
            ..RegisterSignatureMessageHttp::dummy()
        };

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;
    }

    #[tokio::test]
    async fn test_register_signatures_return_400_if_authentication_fail() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service.expect_register_single_signature().never();
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_reject_everything());

        let message = RegisterSignatureMessageHttp {
            signed_message: "message".to_string(),
            ..RegisterSignatureMessageHttp::dummy()
        };

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::BAD_REQUEST,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ok_201() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Registered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp::dummy();

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::CREATED,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ok_202() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Buffered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp::dummy();

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::ACCEPTED,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ko_400() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Registered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let mut message = RegisterSignatureMessageHttp::dummy();
        message.signature = "invalid-signature".to_string();

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::BAD_REQUEST,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ko_404() {
        let signed_entity_type = SignedEntityType::dummy();
        let message = RegisterSignatureMessageHttp::dummy();
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| {
                Err(CertifierServiceError::NotFound(signed_entity_type).into())
            });
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::NOT_FOUND,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ko_410_when_already_certified() {
        let signed_entity_type = SignedEntityType::dummy();
        let message = RegisterSignatureMessageHttp::dummy();
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| {
                Err(CertifierServiceError::AlreadyCertified(signed_entity_type).into())
            });
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        let response_body: ClientError = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(
            response_body,
            ClientError::new(
                "already_certified",
                CertifierServiceError::AlreadyCertified(SignedEntityType::dummy()).to_string()
            )
        );

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::GONE,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ko_410_when_expired() {
        let message = RegisterSignatureMessageHttp::dummy();
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| {
                Err(CertifierServiceError::Expired(SignedEntityType::dummy()).into())
            });
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        let response_body: ClientError = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(
            response_body,
            ClientError::new(
                "expired",
                CertifierServiceError::Expired(SignedEntityType::dummy()).to_string()
            )
        );

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::GONE,
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_register_signatures_post_ko_500() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .return_once(move |_, _, _| Err(anyhow!("an error occurred")));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp::dummy();

        let method = Method::POST.as_str();
        let path = "/register-signatures";

        let response = request()
            .method(method)
            .path(path)
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        APISpec::verify_conformity(
            APISpec::get_default_spec_file_from(crate::http_server::API_SPEC_LOCATION),
            method,
            path,
            "application/json",
            &message,
            &response,
            &StatusCode::INTERNAL_SERVER_ERROR,
        )
        .unwrap();
    }

    // ========================================================================
    // CHAIN-SPECIFIC ENDPOINT TESTS (Multi-Chain Support)
    // ========================================================================

    #[tokio::test]
    async fn test_cardano_register_signatures_post_ok() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .withf(|_, _, chain_type| chain_type == "cardano")
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Registered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp::dummy();
        let response = request()
            .method(Method::POST.as_str())
            .path("/cardano/register-signatures")
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_ethereum_register_signatures_post_ok() {
        let mut mock_certifier_service = MockCertifierService::new();
        mock_certifier_service
            .expect_register_single_signature()
            .withf(|_, _, chain_type| chain_type == "ethereum")
            .return_once(move |_, _, _| Ok(SignatureRegistrationStatus::Registered));
        let mut dependency_manager = initialize_dependencies!().await;
        dependency_manager.certifier_service = Arc::new(mock_certifier_service);
        dependency_manager.single_signer_authenticator =
            Arc::new(SingleSignatureAuthenticator::new_that_authenticate_everything());

        let message = RegisterSignatureMessageHttp::dummy();
        let response = request()
            .method(Method::POST.as_str())
            .path("/ethereum/register-signatures")
            .json(&message)
            .reply(&setup_router(RouterState::new_with_dummy_config(Arc::new(
                dependency_manager,
            ))))
            .await;

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
