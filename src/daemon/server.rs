use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer};
use std::sync::Arc;
use tracing::info;
use crate::daemon::lifecycle::DaemonLifecycle;
use crate::daemon::state::ReadOnlyStorage;
use crate::daemon::handlers::LspHandlers;

pub struct Backend {
    pub client: Client,
    pub lifecycle: Arc<DaemonLifecycle>,
    pub storage: Arc<ReadOnlyStorage>,
    pub handlers: Arc<LspHandlers>,
}

impl Backend {
    pub fn new(client: Client, lifecycle: DaemonLifecycle, storage: ReadOnlyStorage) -> Self {
        let lifecycle = Arc::new(lifecycle);
        let storage = Arc::new(storage);
        let handlers = Arc::new(LspHandlers::new(client.clone(), storage.clone()));
        Self {
            client,
            lifecycle,
            storage,
            handlers,
        }
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> tower_lsp_server::jsonrpc::Result<InitializeResult> {
        info!("LSP Server Initializing");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("LSP Server Initialized");
        self.client
            .log_message(MessageType::INFO, "ChangeGuard LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> tower_lsp_server::jsonrpc::Result<()> {
        info!("LSP Server Shutting Down");
        let _ = self.lifecycle.cleanup();
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.handlers.on_open(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.handlers.on_change(params).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.handlers.on_save(params).await;
    }

    async fn hover(&self, params: HoverParams) -> tower_lsp_server::jsonrpc::Result<Option<Hover>> {
        self.handlers.on_hover(params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> tower_lsp_server::jsonrpc::Result<Option<Vec<CodeLens>>> {
        self.handlers.on_code_lens(params).await
    }
}
