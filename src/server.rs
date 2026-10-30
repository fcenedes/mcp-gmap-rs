use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rmcp::{
    ErrorData, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
        ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    },
    service::{RequestContext, RoleServer},
    transport::{
        stdio,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
        },
    },
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::tools::{EngineState, Output};

#[derive(Clone)]
pub(crate) struct MapsServer(pub EngineState);

impl ServerHandler for MapsServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("mcp-gmap-rs",env!("CARGO_PKG_VERSION")))
            .with_instructions("Google Maps read-only tools. Results may incur Google API charges. Preserve Google Maps attribution, review authors, photo authors and AI summary disclosures when displaying content.")
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.0.catalog.get(name).ok().map(|d| d.tool.clone())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(
            self.0.catalog.iter().map(|d| d.tool.clone()).collect(),
        ))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        self.0
            .catalog
            .get(&request.name)
            .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;
        let args = serde_json::Value::Object(request.arguments.unwrap_or_default());
        let result = match self.0.call(&request.name, args).await {
            Ok(Output::Json(value)) => {
                let mut result =
                    CallToolResult::success(vec![ContentBlock::text(value.to_string())]);
                result.structured_content = Some(value);
                result
            }
            Ok(Output::Image(data)) => CallToolResult::success(vec![ContentBlock::image(
                STANDARD.encode(data),
                "image/png",
            )]),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error.to_string())]),
        };
        Ok(result.into())
    }
}

pub(crate) async fn serve_stdio(engine: EngineState) -> anyhow::Result<()> {
    let service = MapsServer(engine).serve(stdio()).await?;
    tokio::select! {
        result=service.waiting()=>{result?;},
        _=shutdown_signal()=>{},
    }
    Ok(())
}

pub(crate) struct HttpOptions {
    pub listen: SocketAddr,
    pub token: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub allowed_origins: Vec<String>,
}

pub(crate) fn http_router(
    engine: EngineState,
    options: &HttpOptions,
    cancellation: CancellationToken,
) -> Router {
    let mut config = StreamableHttpServerConfig::default()
        .with_allowed_origins(options.allowed_origins.clone())
        .enforce_origin_validation();
    config.cancellation_token = cancellation;
    config.max_request_body_bytes = 1024 * 1024;
    if !options.allowed_hosts.is_empty() {
        config = config.with_allowed_hosts(options.allowed_hosts.clone());
    }
    let service = StreamableHttpService::new(
        move || Ok(MapsServer(engine.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let mcp = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(
            Arc::new(options.token.clone()),
            authorize,
        ));
    Router::new()
        .merge(mcp)
        .route("/health", get(|| async { "ok" }))
}

pub(crate) async fn serve_http(engine: EngineState, options: HttpOptions) -> anyhow::Result<()> {
    if !options.listen.ip().is_loopback() && options.token.is_none() {
        anyhow::bail!("MCP_HTTP_TOKEN is required when binding to a non-loopback address");
    }
    let listener = TcpListener::bind(options.listen).await?;
    let address = listener.local_addr()?;
    let cancellation = CancellationToken::new();
    let app = http_router(engine, &options, cancellation.clone());
    eprintln!("MCP Streamable HTTP listening on http://{address}/mcp");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            cancellation.cancel();
        })
        .await?;
    Ok(())
}

async fn authorize(
    State(token): State<Arc<Option<String>>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(token) = token.as_ref() {
        let provided = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));
        if provided != Some(token.as_str()) {
            return (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                "unauthorized",
            )
                .into_response();
        }
    }
    next.run(request).await
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {_=tokio::signal::ctrl_c()=>{},_=terminate.recv()=>{}}
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
