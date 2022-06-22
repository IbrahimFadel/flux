// use dashmap::DashMap;
// use flux_error::FileId;
// use flux_hir::{hir::FnDecl, lower, HirModule};
// use flux_parser::{parse, Parse};
// use flux_syntax::{ast, ast::AstNode};
// use smol_str::SmolStr;
// use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

// use flux_lsp::{
// 	capabilities, completion,
// 	position::{flux_span_to_location, position_to_offset},
// 	semantic_tokens::{self},
// };

// #[derive(Debug)]
// struct Backend {
// 	client: Client,
// 	hir_module_map: DashMap<Url, HirModule>,
// 	cst_map: DashMap<Url, Parse>,
// 	file_uri_map: DashMap<FileId, Url>,
// 	file_id_map: DashMap<Url, FileId>,
// 	file_source_map: DashMap<Url, String>,
// 	semantic_token_map: DashMap<Url, Vec<SemanticToken>>,
// }

// #[tower_lsp::async_trait]
// impl LanguageServer for Backend {
// 	async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
// 		Ok(InitializeResult {
// 			server_info: None,
// 			capabilities: capabilities::capabilities(),
// 		})
// 	}

// 	async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
// 		self
// 			.client
// 			.log_message(MessageType::INFO, "completion")
// 			.await;

// 		let hir_module = self
// 			.hir_module_map
// 			.get(&params.text_document_position.text_document.uri)
// 			.expect("expected module");

// 		let src = self
// 			.file_source_map
// 			.get(&params.text_document_position.text_document.uri)
// 			.expect("expected source");

// 		let names = completion::get_completion_items(
// 			&*hir_module,
// 			&params.text_document_position.position,
// 			&*src,
// 		);

// 		Ok(Some(CompletionResponse::Array(names)))
// 	}

// 	async fn goto_definition(
// 		&self,
// 		params: GotoDefinitionParams,
// 	) -> Result<Option<GotoDefinitionResponse>> {
// 		self
// 			.client
// 			.log_message(MessageType::INFO, format!("goto definition: {:#?}", params))
// 			.await;

// 		Ok(None)
// 	}

// 	async fn semantic_tokens_full(
// 		&self,
// 		params: SemanticTokensParams,
// 	) -> Result<Option<SemanticTokensResult>> {
// 		let semantic_tokens = self
// 			.semantic_token_map
// 			.get(&params.text_document.uri)
// 			.expect("expected semantic tokens");
// 		Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
// 			result_id: None,
// 			data: semantic_tokens.clone(),
// 		})))
// 	}

// 	async fn semantic_tokens_range(
// 		&self,
// 		params: SemanticTokensRangeParams,
// 	) -> Result<Option<SemanticTokensRangeResult>> {
// 		let semantic_tokens = self
// 			.semantic_token_map
// 			.get(&params.text_document.uri)
// 			.expect("expected semantic tokens");
// 		Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
// 			result_id: None,
// 			data: semantic_tokens.clone(),
// 		})))
// 	}

// 	async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
// 		self.client.log_message(MessageType::INFO, "hover").await;
// 		let uri = params.text_document_position_params.text_document.uri;
// 		let pos = params.text_document_position_params.position;

// 		let hir_module = self.hir_module_map.get(&uri).expect("expected hir module");
// 		let src = self.file_source_map.get(&uri).expect("expected source");
// 		eprintln!("{:?}", pos);

// 		let mut closest_function: Option<(u32, &FnDecl)> = None;
// 		hir_module.functions.iter().for_each(|f| {
// 			let off = position_to_offset(&pos, &*src);
// 			eprintln!("{:?}", off);
// 			if u32::from(f.name.span.range.end()) < off {
// 				if let Some(closest) = closest_function {
// 					if off > closest.0 {
// 						closest_function = Some((off, f));
// 					}
// 				} else {
// 					closest_function = Some((off, f));
// 				}
// 			}
// 		});

// 		// if let Some((_, f)) = closest_function {
// 		// 	for stmt in &f.block.0 {
// 		// 		if let Some(stmt) = stmt {
// 		// 			eprintln!("{:?}", stmt.span.range);
// 		// 			eprintln!("{:?}", pos);
// 		// 		}
// 		// 	}
// 		// }

// 		Ok(None)
// 	}

// 	async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
// 		self
// 			.client
// 			.log_message(MessageType::INFO, "formatting")
// 			.await;
// 		Ok(None)
// 	}

// 	async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
// 		let file_id = self
// 			.file_id_map
// 			.get(&params.text_document.uri)
// 			.expect("expected file id");
// 		self
// 			.on_change(
// 				TextDocumentItem {
// 					uri: params.text_document.uri,
// 					language_id: String::from("flux"),
// 					version: params.text_document.version,
// 					text: std::mem::take(&mut params.content_changes[0].text),
// 				},
// 				*file_id,
// 			)
// 			.await;
// 	}

// 	async fn initialized(&self, _: InitializedParams) {
// 		self
// 			.client
// 			.log_message(MessageType::INFO, "Flux Language Server Initialized")
// 			.await;
// 	}

// 	async fn did_save(&self, params: DidSaveTextDocumentParams) {
// 		self
// 			.client
// 			.log_message(
// 				MessageType::INFO,
// 				format!("saved file `{}`", params.text_document.uri),
// 			)
// 			.await;
// 	}

// 	async fn did_open(&self, params: DidOpenTextDocumentParams) {
// 		self
// 			.client
// 			.log_message(
// 				MessageType::INFO,
// 				format!("opened f `{}`", params.text_document.uri),
// 			)
// 			.await;

// 		let file_id = if let Some(id) = self.file_id_map.get(&params.text_document.uri) {
// 			id.clone()
// 		} else {
// 			let id = FileId(self.file_id_map.len() as u32);
// 			self
// 				.file_id_map
// 				.insert(params.text_document.uri.clone(), id);
// 			self
// 				.file_uri_map
// 				.insert(id, params.text_document.uri.clone());
// 			self.file_source_map.insert(
// 				params.text_document.uri.clone(),
// 				params.text_document.text.clone(),
// 			);
// 			id
// 		};

// 		self.on_change(params.text_document, file_id).await;
// 	}

// 	async fn shutdown(&self) -> Result<()> {
// 		Ok(())
// 	}
// }

// impl Backend {
// 	async fn on_change(&self, params: TextDocumentItem, file_id: FileId) {
// 		self
// 			.file_source_map
// 			.insert(params.uri.clone(), params.text.clone());
// 		let (hir_module, errors, semantic_tokens) = {
// 			let src = params.text.clone();
// 			let mut cst = parse(src.as_str(), file_id);
// 			self.cst_map.insert(params.uri.clone(), cst.clone());
// 			let root = ast::Root::cast(cst.syntax()).unwrap();
// 			let semantic_tokens = semantic_tokens::cst_to_semantic_tokens(&root, &params.text);
// 			let (module, mut errs) = lower(vec![SmolStr::from(params.uri.clone())], root, file_id);
// 			errs.append(&mut cst.errors);
// 			(module, errs, semantic_tokens)
// 		};

// 		self
// 			.semantic_token_map
// 			.insert(params.uri.clone(), semantic_tokens);

// 		// let res = flux_typecheck::typecheck_hir_modules(
// 		// 	&mut [hir_module.clone()],
// 		// 	&HashMap::new(),
// 		// 	&HashMap::new(),
// 		// );
// 		// if let Some(err) = res.err() {
// 		// 	eprintln!("{:#?}", err);
// 		// 	errors.push(err);
// 		// }
// 		self.hir_module_map.insert(params.uri.clone(), hir_module);

// 		let diagnostics: Vec<Diagnostic> = errors
// 			.iter()
// 			.map(|err| self.flux_error_to_diagnostic(err))
// 			.collect();

// 		self
// 			.client
// 			.publish_diagnostics(params.uri, diagnostics, None)
// 			.await;
// 	}

// 	fn flux_error_to_diagnostic(&self, err: &flux_error::FluxError) -> Diagnostic {
// 		let range = if let Some(primary) = &err.primary {
// 			if let Some(span) = &primary.1 {
// 				let uri = self
// 					.file_uri_map
// 					.get(&span.file_id)
// 					.expect("expected to find file uri");

// 				let src = self
// 					.file_source_map
// 					.get(&uri)
// 					.expect("expected to find file source");
// 				flux_span_to_location(&*uri, span, &*src).range
// 			} else {
// 				Range::new(Position::new(0, 0), Position::new(0, 0))
// 			}
// 		} else {
// 			Range::new(Position::new(0, 0), Position::new(0, 0))
// 		};

// 		let mut diagnostic_related_informations = vec![];
// 		for label in &err.labels {
// 			if let Some(span) = &label.1 {
// 				let uri = self
// 					.file_uri_map
// 					.get(&span.file_id)
// 					.expect("expected to find file uri");

// 				let src = self
// 					.file_source_map
// 					.get(&uri)
// 					.expect("expected to find file source");
// 				diagnostic_related_informations.push(DiagnosticRelatedInformation {
// 					location: flux_span_to_location(&*uri, span, &*src),
// 					message: label.0.clone(),
// 				})
// 			}
// 		}
// 		Diagnostic::new(
// 			range,
// 			Some(DiagnosticSeverity::ERROR),
// 			Some(NumberOrString::Number(err.code as i32)),
// 			None,
// 			err.msg.clone(),
// 			Some(diagnostic_related_informations),
// 			None,
// 		)
// 	}
// }

// #[tokio::main]
// async fn main() {
// 	let stdin = tokio::io::stdin();
// 	let stdout = tokio::io::stdout();

// 	let (service, socket) = LspService::build(|client| Backend {
// 		client,
// 		hir_module_map: DashMap::new(),
// 		cst_map: DashMap::new(),
// 		file_uri_map: DashMap::new(),
// 		file_id_map: DashMap::new(),
// 		file_source_map: DashMap::new(),
// 		semantic_token_map: DashMap::new(),
// 	})
// 	.finish();
// 	Server::new(stdin, stdout, socket).serve(service).await;
// }

fn main() {}
