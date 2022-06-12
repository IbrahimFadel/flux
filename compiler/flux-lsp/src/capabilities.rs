use tower_lsp::lsp_types::*;

use crate::semantic_tokens::LEGEND_TYPE;

pub fn capabilities() -> ServerCapabilities {
	ServerCapabilities {
		text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
		completion_provider: Some(CompletionOptions {
			resolve_provider: Some(false),
			trigger_characters: Some(vec![".".to_string()]),
			work_done_progress_options: Default::default(),
			all_commit_characters: None,
		}),
		execute_command_provider: Some(ExecuteCommandOptions {
			commands: vec!["dummy.do_something".to_string()],
			work_done_progress_options: Default::default(),
		}),

		workspace: Some(WorkspaceServerCapabilities {
			workspace_folders: Some(WorkspaceFoldersServerCapabilities {
				supported: Some(true),
				change_notifications: Some(OneOf::Left(true)),
			}),
			file_operations: None,
		}),
		semantic_tokens_provider: Some(
			SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
				SemanticTokensRegistrationOptions {
					text_document_registration_options: {
						TextDocumentRegistrationOptions {
							document_selector: Some(vec![DocumentFilter {
								language: Some("flux".to_string()),
								scheme: Some("file".to_string()),
								pattern: None,
							}]),
						}
					},
					semantic_tokens_options: SemanticTokensOptions {
						work_done_progress_options: WorkDoneProgressOptions::default(),
						legend: SemanticTokensLegend {
							token_types: LEGEND_TYPE.clone().into(),
							token_modifiers: vec![],
						},
						range: Some(true),
						full: Some(SemanticTokensFullOptions::Bool(true)),
					},
					static_registration_options: StaticRegistrationOptions::default(),
				},
			),
		),
		// definition: Some(GotoCapability::default()),
		definition_provider: Some(OneOf::Left(true)),
		references_provider: Some(OneOf::Left(true)),
		rename_provider: Some(OneOf::Left(true)),
		// hover_provider: Some(HoverProviderCapability::Simple(true)),
		hover_provider: Some(HoverProviderCapability::Options(HoverOptions::default())),
		..ServerCapabilities::default()
	}
}
