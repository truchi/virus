// 🚨 This file is generated by `cargo xtask-lsp`

use super::*;

use super::notifications::*;
use super::requests::*;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                     LspClientNotification                                      //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

impl<'client, W: AsyncWrite + Unpin> super::LspClientNotification<'client, W> {
    /// @see [`CancelRequest`](super::notifications::CancelRequest).
    pub async fn cancel_request(
        &mut self,
        params: super::structures::CancelParams,
    ) -> std::io::Result<()> {
        self.client.send_notification::<CancelRequest>(params).await
    }
    /// @see [`Progress`](super::notifications::Progress).
    pub async fn progress(
        &mut self,
        params: super::structures::ProgressParams,
    ) -> std::io::Result<()> {
        self.client.send_notification::<Progress>(params).await
    }
    /// @see [`SetTrace`](super::notifications::SetTrace).
    pub async fn set_trace(
        &mut self,
        params: super::structures::SetTraceParams,
    ) -> std::io::Result<()> {
        self.client.send_notification::<SetTrace>(params).await
    }
    /// @see [`Exit`](super::notifications::Exit).
    pub async fn exit(&mut self) -> std::io::Result<()> {
        self.client.send_notification::<Exit>(()).await
    }
    /// @see [`Initialized`](super::notifications::Initialized).
    pub async fn initialized(
        &mut self,
        params: super::structures::InitializedParams,
    ) -> std::io::Result<()> {
        self.client.send_notification::<Initialized>(params).await
    }
    /// @see [`NotebookDocumentDidChange`](super::notifications::NotebookDocumentDidChange).
    pub async fn notebook_document_did_change(
        &mut self,
        params: super::structures::DidChangeNotebookDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<NotebookDocumentDidChange>(params)
            .await
    }
    /// @see [`NotebookDocumentDidClose`](super::notifications::NotebookDocumentDidClose).
    pub async fn notebook_document_did_close(
        &mut self,
        params: super::structures::DidCloseNotebookDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<NotebookDocumentDidClose>(params)
            .await
    }
    /// @see [`NotebookDocumentDidOpen`](super::notifications::NotebookDocumentDidOpen).
    pub async fn notebook_document_did_open(
        &mut self,
        params: super::structures::DidOpenNotebookDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<NotebookDocumentDidOpen>(params)
            .await
    }
    /// @see [`NotebookDocumentDidSave`](super::notifications::NotebookDocumentDidSave).
    pub async fn notebook_document_did_save(
        &mut self,
        params: super::structures::DidSaveNotebookDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<NotebookDocumentDidSave>(params)
            .await
    }
    /// @see [`TextDocumentDidChange`](super::notifications::TextDocumentDidChange).
    pub async fn text_document_did_change(
        &mut self,
        params: super::structures::DidChangeTextDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<TextDocumentDidChange>(params)
            .await
    }
    /// @see [`TextDocumentDidClose`](super::notifications::TextDocumentDidClose).
    pub async fn text_document_did_close(
        &mut self,
        params: super::structures::DidCloseTextDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<TextDocumentDidClose>(params)
            .await
    }
    /// @see [`TextDocumentDidOpen`](super::notifications::TextDocumentDidOpen).
    pub async fn text_document_did_open(
        &mut self,
        params: super::structures::DidOpenTextDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<TextDocumentDidOpen>(params)
            .await
    }
    /// @see [`TextDocumentDidSave`](super::notifications::TextDocumentDidSave).
    pub async fn text_document_did_save(
        &mut self,
        params: super::structures::DidSaveTextDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<TextDocumentDidSave>(params)
            .await
    }
    /// @see [`TextDocumentWillSave`](super::notifications::TextDocumentWillSave).
    pub async fn text_document_will_save(
        &mut self,
        params: super::structures::WillSaveTextDocumentParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<TextDocumentWillSave>(params)
            .await
    }
    /// @see [`WindowWorkDoneProgressCancel`](super::notifications::WindowWorkDoneProgressCancel).
    pub async fn window_work_done_progress_cancel(
        &mut self,
        params: super::structures::WorkDoneProgressCancelParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WindowWorkDoneProgressCancel>(params)
            .await
    }
    /// @see [`WorkspaceDidChangeConfiguration`](super::notifications::WorkspaceDidChangeConfiguration).
    pub async fn workspace_did_change_configuration(
        &mut self,
        params: super::structures::DidChangeConfigurationParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidChangeConfiguration>(params)
            .await
    }
    /// @see [`WorkspaceDidChangeWatchedFiles`](super::notifications::WorkspaceDidChangeWatchedFiles).
    pub async fn workspace_did_change_watched_files(
        &mut self,
        params: super::structures::DidChangeWatchedFilesParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidChangeWatchedFiles>(params)
            .await
    }
    /// @see [`WorkspaceDidChangeWorkspaceFolders`](super::notifications::WorkspaceDidChangeWorkspaceFolders).
    pub async fn workspace_did_change_workspace_folders(
        &mut self,
        params: super::structures::DidChangeWorkspaceFoldersParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidChangeWorkspaceFolders>(params)
            .await
    }
    /// @see [`WorkspaceDidCreateFiles`](super::notifications::WorkspaceDidCreateFiles).
    pub async fn workspace_did_create_files(
        &mut self,
        params: super::structures::CreateFilesParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidCreateFiles>(params)
            .await
    }
    /// @see [`WorkspaceDidDeleteFiles`](super::notifications::WorkspaceDidDeleteFiles).
    pub async fn workspace_did_delete_files(
        &mut self,
        params: super::structures::DeleteFilesParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidDeleteFiles>(params)
            .await
    }
    /// @see [`WorkspaceDidRenameFiles`](super::notifications::WorkspaceDidRenameFiles).
    pub async fn workspace_did_rename_files(
        &mut self,
        params: super::structures::RenameFilesParams,
    ) -> std::io::Result<()> {
        self.client
            .send_notification::<WorkspaceDidRenameFiles>(params)
            .await
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                        LspClientRequest                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

impl<'client, W: AsyncWrite + Unpin> super::LspClientRequest<'client, W> {
    /// @see [`CallHierarchyIncomingCalls`](super::requests::CallHierarchyIncomingCalls).
    pub async fn call_hierarchy_incoming_calls(
        &mut self,
        params: super::structures::CallHierarchyIncomingCallsParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<CallHierarchyIncomingCallsResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<CallHierarchyIncomingCalls>(params)
            .await
    }
    /// @see [`CallHierarchyOutgoingCalls`](super::requests::CallHierarchyOutgoingCalls).
    pub async fn call_hierarchy_outgoing_calls(
        &mut self,
        params: super::structures::CallHierarchyOutgoingCallsParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<CallHierarchyOutgoingCallsResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<CallHierarchyOutgoingCalls>(params)
            .await
    }
    /// @see [`CodeActionResolve`](super::requests::CodeActionResolve).
    pub async fn code_action_resolve(
        &mut self,
        params: super::structures::CodeAction,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::CodeAction, Error<()>>>,
            >,
    > {
        self.client.send_request::<CodeActionResolve>(params).await
    }
    /// @see [`CodeLensResolve`](super::requests::CodeLensResolve).
    pub async fn code_lens_resolve(
        &mut self,
        params: super::structures::CodeLens,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::CodeLens, Error<()>>>,
            >,
    > {
        self.client.send_request::<CodeLensResolve>(params).await
    }
    /// @see [`CompletionItemResolve`](super::requests::CompletionItemResolve).
    pub async fn completion_item_resolve(
        &mut self,
        params: super::structures::CompletionItem,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::CompletionItem, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<CompletionItemResolve>(params)
            .await
    }
    /// @see [`DocumentLinkResolve`](super::requests::DocumentLinkResolve).
    pub async fn document_link_resolve(
        &mut self,
        params: super::structures::DocumentLink,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::DocumentLink, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<DocumentLinkResolve>(params)
            .await
    }
    /// @see [`Initialize`](super::requests::Initialize).
    pub async fn initialize(
        &mut self,
        params: super::structures::InitializeParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<
                        super::structures::InitializeResult,
                        Error<super::structures::InitializeError>,
                    >,
                >,
            >,
    > {
        self.client.send_request::<Initialize>(params).await
    }
    /// @see [`InlayHintResolve`](super::requests::InlayHintResolve).
    pub async fn inlay_hint_resolve(
        &mut self,
        params: super::structures::InlayHint,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::InlayHint, Error<()>>>,
            >,
    > {
        self.client.send_request::<InlayHintResolve>(params).await
    }
    /// @see [`Shutdown`](super::requests::Shutdown).
    pub async fn shutdown(
        &mut self,
    ) -> std::io::Result<impl '_ + futures::Future<Output = std::io::Result<Result<Null, Error<()>>>>>
    {
        self.client.send_request::<Shutdown>(()).await
    }
    /// @see [`TextDocumentCodeAction`](super::requests::TextDocumentCodeAction).
    pub async fn text_document_code_action(
        &mut self,
        params: super::structures::CodeActionParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentCodeActionResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentCodeAction>(params)
            .await
    }
    /// @see [`TextDocumentCodeLens`](super::requests::TextDocumentCodeLens).
    pub async fn text_document_code_lens(
        &mut self,
        params: super::structures::CodeLensParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<Output = std::io::Result<Result<TextDocumentCodeLensResult, Error<()>>>>,
    > {
        self.client
            .send_request::<TextDocumentCodeLens>(params)
            .await
    }
    /// @see [`TextDocumentColorPresentation`](super::requests::TextDocumentColorPresentation).
    pub async fn text_document_color_presentation(
        &mut self,
        params: super::structures::ColorPresentationParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<Vec<super::structures::ColorPresentation>, Error<()>>,
                >,
            >,
    > {
        self.client
            .send_request::<TextDocumentColorPresentation>(params)
            .await
    }
    /// @see [`TextDocumentCompletion`](super::requests::TextDocumentCompletion).
    pub async fn text_document_completion(
        &mut self,
        params: super::structures::CompletionParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentCompletionResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentCompletion>(params)
            .await
    }
    /// @see [`TextDocumentDeclaration`](super::requests::TextDocumentDeclaration).
    pub async fn text_document_declaration(
        &mut self,
        params: super::structures::DeclarationParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentDeclarationResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentDeclaration>(params)
            .await
    }
    /// @see [`TextDocumentDefinition`](super::requests::TextDocumentDefinition).
    pub async fn text_document_definition(
        &mut self,
        params: super::structures::DefinitionParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentDefinitionResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentDefinition>(params)
            .await
    }
    /// @see [`TextDocumentDiagnostic`](super::requests::TextDocumentDiagnostic).
    pub async fn text_document_diagnostic(
        &mut self,
        params: super::structures::DocumentDiagnosticParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<
                        super::type_aliases::DocumentDiagnosticReport,
                        Error<super::structures::DiagnosticServerCancellationData>,
                    >,
                >,
            >,
    > {
        self.client
            .send_request::<TextDocumentDiagnostic>(params)
            .await
    }
    /// @see [`TextDocumentDocumentColor`](super::requests::TextDocumentDocumentColor).
    pub async fn text_document_document_color(
        &mut self,
        params: super::structures::DocumentColorParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<Vec<super::structures::ColorInformation>, Error<()>>,
                >,
            >,
    > {
        self.client
            .send_request::<TextDocumentDocumentColor>(params)
            .await
    }
    /// @see [`TextDocumentDocumentHighlight`](super::requests::TextDocumentDocumentHighlight).
    pub async fn text_document_document_highlight(
        &mut self,
        params: super::structures::DocumentHighlightParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentDocumentHighlightResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentDocumentHighlight>(params)
            .await
    }
    /// @see [`TextDocumentDocumentLink`](super::requests::TextDocumentDocumentLink).
    pub async fn text_document_document_link(
        &mut self,
        params: super::structures::DocumentLinkParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentDocumentLinkResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentDocumentLink>(params)
            .await
    }
    /// @see [`TextDocumentDocumentSymbol`](super::requests::TextDocumentDocumentSymbol).
    pub async fn text_document_document_symbol(
        &mut self,
        params: super::structures::DocumentSymbolParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentDocumentSymbolResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentDocumentSymbol>(params)
            .await
    }
    /// @see [`TextDocumentFoldingRange`](super::requests::TextDocumentFoldingRange).
    pub async fn text_document_folding_range(
        &mut self,
        params: super::structures::FoldingRangeParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentFoldingRangeResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentFoldingRange>(params)
            .await
    }
    /// @see [`TextDocumentFormatting`](super::requests::TextDocumentFormatting).
    pub async fn text_document_formatting(
        &mut self,
        params: super::structures::DocumentFormattingParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentFormattingResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentFormatting>(params)
            .await
    }
    /// @see [`TextDocumentHover`](super::requests::TextDocumentHover).
    pub async fn text_document_hover(
        &mut self,
        params: super::structures::HoverParams,
    ) -> std::io::Result<
        impl '_ + futures::Future<Output = std::io::Result<Result<TextDocumentHoverResult, Error<()>>>>,
    > {
        self.client.send_request::<TextDocumentHover>(params).await
    }
    /// @see [`TextDocumentImplementation`](super::requests::TextDocumentImplementation).
    pub async fn text_document_implementation(
        &mut self,
        params: super::structures::ImplementationParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentImplementationResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentImplementation>(params)
            .await
    }
    /// @see [`TextDocumentInlayHint`](super::requests::TextDocumentInlayHint).
    pub async fn text_document_inlay_hint(
        &mut self,
        params: super::structures::InlayHintParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentInlayHintResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentInlayHint>(params)
            .await
    }
    /// @see [`TextDocumentInlineValue`](super::requests::TextDocumentInlineValue).
    pub async fn text_document_inline_value(
        &mut self,
        params: super::structures::InlineValueParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentInlineValueResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentInlineValue>(params)
            .await
    }
    /// @see [`TextDocumentLinkedEditingRange`](super::requests::TextDocumentLinkedEditingRange).
    pub async fn text_document_linked_editing_range(
        &mut self,
        params: super::structures::LinkedEditingRangeParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentLinkedEditingRangeResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentLinkedEditingRange>(params)
            .await
    }
    /// @see [`TextDocumentMoniker`](super::requests::TextDocumentMoniker).
    pub async fn text_document_moniker(
        &mut self,
        params: super::structures::MonikerParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<Output = std::io::Result<Result<TextDocumentMonikerResult, Error<()>>>>,
    > {
        self.client
            .send_request::<TextDocumentMoniker>(params)
            .await
    }
    /// @see [`TextDocumentOnTypeFormatting`](super::requests::TextDocumentOnTypeFormatting).
    pub async fn text_document_on_type_formatting(
        &mut self,
        params: super::structures::DocumentOnTypeFormattingParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentOnTypeFormattingResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentOnTypeFormatting>(params)
            .await
    }
    /// @see [`TextDocumentPrepareCallHierarchy`](super::requests::TextDocumentPrepareCallHierarchy).
    pub async fn text_document_prepare_call_hierarchy(
        &mut self,
        params: super::structures::CallHierarchyPrepareParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentPrepareCallHierarchyResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentPrepareCallHierarchy>(params)
            .await
    }
    /// @see [`TextDocumentPrepareRename`](super::requests::TextDocumentPrepareRename).
    pub async fn text_document_prepare_rename(
        &mut self,
        params: super::structures::PrepareRenameParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentPrepareRenameResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentPrepareRename>(params)
            .await
    }
    /// @see [`TextDocumentPrepareTypeHierarchy`](super::requests::TextDocumentPrepareTypeHierarchy).
    pub async fn text_document_prepare_type_hierarchy(
        &mut self,
        params: super::structures::TypeHierarchyPrepareParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentPrepareTypeHierarchyResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentPrepareTypeHierarchy>(params)
            .await
    }
    /// @see [`TextDocumentRangeFormatting`](super::requests::TextDocumentRangeFormatting).
    pub async fn text_document_range_formatting(
        &mut self,
        params: super::structures::DocumentRangeFormattingParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentRangeFormattingResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentRangeFormatting>(params)
            .await
    }
    /// @see [`TextDocumentReferences`](super::requests::TextDocumentReferences).
    pub async fn text_document_references(
        &mut self,
        params: super::structures::ReferenceParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentReferencesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentReferences>(params)
            .await
    }
    /// @see [`TextDocumentRename`](super::requests::TextDocumentRename).
    pub async fn text_document_rename(
        &mut self,
        params: super::structures::RenameParams,
    ) -> std::io::Result<
        impl '_ + futures::Future<Output = std::io::Result<Result<TextDocumentRenameResult, Error<()>>>>,
    > {
        self.client.send_request::<TextDocumentRename>(params).await
    }
    /// @see [`TextDocumentSelectionRange`](super::requests::TextDocumentSelectionRange).
    pub async fn text_document_selection_range(
        &mut self,
        params: super::structures::SelectionRangeParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentSelectionRangeResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentSelectionRange>(params)
            .await
    }
    /// @see [`TextDocumentSemanticTokensFull`](super::requests::TextDocumentSemanticTokensFull).
    pub async fn text_document_semantic_tokens_full(
        &mut self,
        params: super::structures::SemanticTokensParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentSemanticTokensFullResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentSemanticTokensFull>(params)
            .await
    }
    /// @see [`TextDocumentSemanticTokensFullDelta`](super::requests::TextDocumentSemanticTokensFullDelta).
    pub async fn text_document_semantic_tokens_full_delta(
        &mut self,
        params: super::structures::SemanticTokensDeltaParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<TextDocumentSemanticTokensFullDeltaResult, Error<()>>,
                >,
            >,
    > {
        self.client
            .send_request::<TextDocumentSemanticTokensFullDelta>(params)
            .await
    }
    /// @see [`TextDocumentSemanticTokensRange`](super::requests::TextDocumentSemanticTokensRange).
    pub async fn text_document_semantic_tokens_range(
        &mut self,
        params: super::structures::SemanticTokensRangeParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentSemanticTokensRangeResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentSemanticTokensRange>(params)
            .await
    }
    /// @see [`TextDocumentSignatureHelp`](super::requests::TextDocumentSignatureHelp).
    pub async fn text_document_signature_help(
        &mut self,
        params: super::structures::SignatureHelpParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentSignatureHelpResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentSignatureHelp>(params)
            .await
    }
    /// @see [`TextDocumentTypeDefinition`](super::requests::TextDocumentTypeDefinition).
    pub async fn text_document_type_definition(
        &mut self,
        params: super::structures::TypeDefinitionParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentTypeDefinitionResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentTypeDefinition>(params)
            .await
    }
    /// @see [`TextDocumentWillSaveWaitUntil`](super::requests::TextDocumentWillSaveWaitUntil).
    pub async fn text_document_will_save_wait_until(
        &mut self,
        params: super::structures::WillSaveTextDocumentParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TextDocumentWillSaveWaitUntilResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TextDocumentWillSaveWaitUntil>(params)
            .await
    }
    /// @see [`TypeHierarchySubtypes`](super::requests::TypeHierarchySubtypes).
    pub async fn type_hierarchy_subtypes(
        &mut self,
        params: super::structures::TypeHierarchySubtypesParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TypeHierarchySubtypesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TypeHierarchySubtypes>(params)
            .await
    }
    /// @see [`TypeHierarchySupertypes`](super::requests::TypeHierarchySupertypes).
    pub async fn type_hierarchy_supertypes(
        &mut self,
        params: super::structures::TypeHierarchySupertypesParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<TypeHierarchySupertypesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<TypeHierarchySupertypes>(params)
            .await
    }
    /// @see [`WorkspaceDiagnostic`](super::requests::WorkspaceDiagnostic).
    pub async fn workspace_diagnostic(
        &mut self,
        params: super::structures::WorkspaceDiagnosticParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<
                    Result<
                        super::structures::WorkspaceDiagnosticReport,
                        Error<super::structures::DiagnosticServerCancellationData>,
                    >,
                >,
            >,
    > {
        self.client
            .send_request::<WorkspaceDiagnostic>(params)
            .await
    }
    /// @see [`WorkspaceExecuteCommand`](super::requests::WorkspaceExecuteCommand).
    pub async fn workspace_execute_command(
        &mut self,
        params: super::structures::ExecuteCommandParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<WorkspaceExecuteCommandResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<WorkspaceExecuteCommand>(params)
            .await
    }
    /// @see [`WorkspaceSymbol`](super::requests::WorkspaceSymbol).
    pub async fn workspace_symbol(
        &mut self,
        params: super::structures::WorkspaceSymbolParams,
    ) -> std::io::Result<
        impl '_ + futures::Future<Output = std::io::Result<Result<WorkspaceSymbolResult, Error<()>>>>,
    > {
        self.client.send_request::<WorkspaceSymbol>(params).await
    }
    /// @see [`WorkspaceWillCreateFiles`](super::requests::WorkspaceWillCreateFiles).
    pub async fn workspace_will_create_files(
        &mut self,
        params: super::structures::CreateFilesParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<WorkspaceWillCreateFilesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<WorkspaceWillCreateFiles>(params)
            .await
    }
    /// @see [`WorkspaceWillDeleteFiles`](super::requests::WorkspaceWillDeleteFiles).
    pub async fn workspace_will_delete_files(
        &mut self,
        params: super::structures::DeleteFilesParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<WorkspaceWillDeleteFilesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<WorkspaceWillDeleteFiles>(params)
            .await
    }
    /// @see [`WorkspaceWillRenameFiles`](super::requests::WorkspaceWillRenameFiles).
    pub async fn workspace_will_rename_files(
        &mut self,
        params: super::structures::RenameFilesParams,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<WorkspaceWillRenameFilesResult, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<WorkspaceWillRenameFiles>(params)
            .await
    }
    /// @see [`WorkspaceSymbolResolve`](super::requests::WorkspaceSymbolResolve).
    pub async fn workspace_symbol_resolve(
        &mut self,
        params: super::structures::WorkspaceSymbol,
    ) -> std::io::Result<
        impl '_
            + futures::Future<
                Output = std::io::Result<Result<super::structures::WorkspaceSymbol, Error<()>>>,
            >,
    > {
        self.client
            .send_request::<WorkspaceSymbolResolve>(params)
            .await
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //
//                                       LspClientResponse                                        //
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ //

impl<'client, W: AsyncWrite + Unpin> super::LspClientResponse<'client, W> {
    /// @see [`ClientRegisterCapability`](super::requests::ClientRegisterCapability).
    pub async fn client_register_capability(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<ClientRegisterCapability>(id, data)
            .await
    }
    /// @see [`ClientUnregisterCapability`](super::requests::ClientUnregisterCapability).
    pub async fn client_unregister_capability(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<ClientUnregisterCapability>(id, data)
            .await
    }
    /// @see [`WindowShowDocument`](super::requests::WindowShowDocument).
    pub async fn window_show_document(
        &mut self,
        id: Option<Id>,
        data: Result<super::structures::ShowDocumentResult, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WindowShowDocument>(id, data)
            .await
    }
    /// @see [`WindowShowMessageRequest`](super::requests::WindowShowMessageRequest).
    pub async fn window_show_message_request(
        &mut self,
        id: Option<Id>,
        data: Result<WindowShowMessageRequestResult, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WindowShowMessageRequest>(id, data)
            .await
    }
    /// @see [`WindowWorkDoneProgressCreate`](super::requests::WindowWorkDoneProgressCreate).
    pub async fn window_work_done_progress_create(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WindowWorkDoneProgressCreate>(id, data)
            .await
    }
    /// @see [`WorkspaceApplyEdit`](super::requests::WorkspaceApplyEdit).
    pub async fn workspace_apply_edit(
        &mut self,
        id: Option<Id>,
        data: Result<super::structures::ApplyWorkspaceEditResult, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceApplyEdit>(id, data)
            .await
    }
    /// @see [`WorkspaceCodeLensRefresh`](super::requests::WorkspaceCodeLensRefresh).
    pub async fn workspace_code_lens_refresh(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceCodeLensRefresh>(id, data)
            .await
    }
    /// @see [`WorkspaceConfiguration`](super::requests::WorkspaceConfiguration).
    pub async fn workspace_configuration(
        &mut self,
        id: Option<Id>,
        data: Result<Vec<super::type_aliases::LspAny>, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceConfiguration>(id, data)
            .await
    }
    /// @see [`WorkspaceDiagnosticRefresh`](super::requests::WorkspaceDiagnosticRefresh).
    pub async fn workspace_diagnostic_refresh(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceDiagnosticRefresh>(id, data)
            .await
    }
    /// @see [`WorkspaceInlayHintRefresh`](super::requests::WorkspaceInlayHintRefresh).
    pub async fn workspace_inlay_hint_refresh(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceInlayHintRefresh>(id, data)
            .await
    }
    /// @see [`WorkspaceInlineValueRefresh`](super::requests::WorkspaceInlineValueRefresh).
    pub async fn workspace_inline_value_refresh(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceInlineValueRefresh>(id, data)
            .await
    }
    /// @see [`WorkspaceSemanticTokensRefresh`](super::requests::WorkspaceSemanticTokensRefresh).
    pub async fn workspace_semantic_tokens_refresh(
        &mut self,
        id: Option<Id>,
        data: Result<Null, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceSemanticTokensRefresh>(id, data)
            .await
    }
    /// @see [`WorkspaceWorkspaceFolders`](super::requests::WorkspaceWorkspaceFolders).
    pub async fn workspace_workspace_folders(
        &mut self,
        id: Option<Id>,
        data: Result<WorkspaceWorkspaceFoldersResult, Error<()>>,
    ) -> std::io::Result<()> {
        self.client
            .send_response::<WorkspaceWorkspaceFolders>(id, data)
            .await
    }
}
