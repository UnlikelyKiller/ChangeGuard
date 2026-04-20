use crate::daemon::state::ReadOnlyStorage;
use crate::output::lsp::map_impact_to_diagnostics;
use std::sync::Arc;
use tower_lsp_server::Client;
use tower_lsp_server::ls_types::*;
use tracing::warn;

pub struct LspHandlers {
    client: Client,
    storage: Arc<ReadOnlyStorage>,
}

impl LspHandlers {
    pub fn new(client: Client, storage: Arc<ReadOnlyStorage>) -> Self {
        Self { client, storage }
    }

    pub async fn on_open(&self, params: DidOpenTextDocumentParams) {
        self.trigger_analysis(params.text_document.uri).await;
    }

    pub async fn on_change(&self, params: DidChangeTextDocumentParams) {
        self.trigger_analysis(params.text_document.uri).await;
    }

    pub async fn on_save(&self, params: DidSaveTextDocumentParams) {
        self.trigger_analysis(params.text_document.uri).await;
    }

    async fn trigger_analysis(&self, uri: Uri) {
        let Some(path) = uri.to_file_path() else {
            return;
        };

        // In a real implementation, we would trigger a background analysis here.
        // For Track 35, we integrate with diagnostics reporting from latest state.
        match self.storage.get_latest_packet() {
            Ok(result) => {
                if let Some(packet) = result.data {
                    let diagnostics_map = map_impact_to_diagnostics(&packet);
                    let diagnostics = diagnostics_map
                        .iter()
                        .find(|(p, _)| **p == path)
                        .map(|(_, d)| d.clone())
                        .unwrap_or_default();

                    self.client
                        .publish_diagnostics(uri, diagnostics, None)
                        .await;
                }
            }
            Err(e) => warn!("Failed to get latest packet for diagnostics: {e}"),
        }
    }

    pub async fn on_hover(
        &self,
        _params: HoverParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<Hover>> {
        // Implementation for Hover: provide impact summaries
        Ok(None)
    }

    pub async fn on_code_lens(
        &self,
        _params: CodeLensParams,
    ) -> tower_lsp_server::jsonrpc::Result<Option<Vec<CodeLens>>> {
        // Implementation for CodeLens: provide risk/complexity scores
        Ok(None)
    }
}
