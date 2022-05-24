use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};
use filesystem::FileId;
use text_size::TextRange;

pub mod filesystem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FluxErrorCode {
	NoCode,
	UnexpectedEOF,
	UnexpectedToken,
	UnresolvedUse,

	HirParseIntString,
	// LexUnknownChar,
	// LexFloatInWrongBase,
	// LexStringLitUnterminated,
	// LexCharLitUnterminated,
	// LexUnknownEscapeSequence,
	// LexInvalidCharLit,
	// LexMissingEndOfBlockComment,
	// LexExpectedDigitsFollowingIntPrefix,

	// ParseExpectedIdent,
	// ParseExpectedTopLevelDecl,
	// ParseExpectedIdentFnDecl,
	// ParseExpectedCommaInGenericTypeList,
	// ParseExpectedGTAfterGenericTypeList,
	// ParseExpectedCommaInParamList,
	// ParseCouldNotConvertTokKindToPrimitiveType,
	// ParseExpectedLBraceInBlock,
	// ParseExpectedRBraceInBlock,
	// ParseExpectedRParenAfterParamList,
	// ParseExpectedLParenBeforeParamList,
	// ParseUnexpectedEOF,
	// ParseExpectedIdentGenericTypeList,
	// ParseExpectedTypeExpr,
	// ParseExpectedIdentVarDecl,
	// ParseExpectedCommaInVarDeclIdentList,
	// ParseExpectedEqVarDeclIdentList,
	// ParseUnexpectedExprOperand,
	// ParseExpectedBasicLit,
	// ParseCouldNotParseInt,
	// ParseMoreValsThanIdentsVarDecl,
	// ParseMoreIdentsThanValsVarDecl,
	// ParseExpectedSemicolonAfterVarDecl,
	// ParseExpectedExprAfterCommaVarDeclValueList,
	// ParseExpectedSemicolonAfterReturnStmt,
	// ParseExpectedLParenBeforeCallExpr,
	// ParseExpectedCommaInCallArgs,
	// ParseExpectedRParenAfterCallExpr,
	// ParseExpectedSemicolonAfterExpr,
	// ParseExpectedTypeInTypeDecl,
	// ParseExpectedLBraceInStructTypeExpr,
	// ParseExpectedRBraceInStructTypeExpr,
	// ParseExpectedIdentInField,
	// ParseExpectedEqInField,
	// ParseExpectedSemicolonInField,
	// ParseExpectedStructInStructTypeExpr,
	// ParseExpectedInterfaceInInterfaceTypeExpr,
	// ParseExpectedLBraceInInterfaceTypeExpr,
	// ParseExpectedRBraceInInterfaceTypeExpr,
	// ParseExpectedSemicolonAfterMethodInInterfaceTypeMethodList,
	// ParseExpectedSemicolonAfterTypeDecl,
	// ParseExpectedSemicolonAfterModStmt,
	// ParseExpectedIdentAfterApply,
	// ParseExpectedIdentAfterTo,
	// ParseExpectedLBraceInApplyBlock,
	// ParseExpectedFnOrRBraceInApplyBlock,
	// ParseExpectedRBraceAfterApplyBlock,
	// ParseUnexpectedThisOutsideApply,
	// ParseExpectedFnInInterfaceMethod,
	// ParseExpectedCommaOrRBraceStructExpr,
	// ParseUnexpectedTokenStructExpr,
	// ParseExpectedLBraceInEnumTypeExpr,
	// ParseExpectedRBraceInEnumTypeExpr,
	// TypecheckExpectedTypeOnLHSOfPeriodToBeEnum,

	// TypecheckUnexpectedSignednessInIntLit,
	// TypecheckExpectedPeriodOpInChainedStructFieldAccess,
	// TypecheckExpectedRHSOfPeriodToBeIdent,
	// TypecheckExpectedLHSOfPeriodToBeStruct,
	// TypecheckStructDoesNotHaveField,
	// TypecheckCouldNotFindType,
	// TypecheckExpectedTypeToBeInterface,
	// TypecheckCouldNotFindMethodInInterface,
	// TypecheckInterfaceMethodRetTyDontMatch,
	// TypecheckInterfaceMethodParamsDontMatch,
	// TypecheckInterfaceMethodVisibilitiesDontMatch,
	// TypecheckNotAllInterfaceMethodsImplemented,
	// TypecheckStructExprDiffNumberFieldsAsStructTy,
	// TypecheckCouldNotFindFieldInStructExpr,
	// TypecheckCouldNotGetTypeOfVar,
	// TypecheckExpectedIntGotFloat,
	// TypecheckExpectedRHSOfEnumExprToBeIdent,
	// TypecheckMissingInitializerInEnumExpr,
	// TypecheckTooManyInitializersInEnumExpr,

	// CodegenUnknownIdentType,
	// CodegenUnknownVarReferenced,
	// CodegenCouldNotCodegenStmt,
	// CodegenCouldNotFindMethod,
	// CodegenCouldNotCmpValsOfType,
	// CodegenCouldNotCodegenTypeExpr,
	// CodegenUnknownFnReferenced
}

impl std::fmt::Display for FluxErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "E{:04}", *self as u8)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
	pub range: TextRange,
	pub file_id: FileId,
}

impl Span {
	pub fn new(range: TextRange, file_id: FileId) -> Span {
		Span { range, file_id }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FluxError {
	msg: String,
	code: FluxErrorCode,
	primary: Option<(String, Option<Span>)>,
	labels: Vec<(String, Option<Span>)>,
	notes: Vec<String>,
}

impl Default for FluxError {
	fn default() -> Self {
		Self {
			msg: String::new(),
			code: FluxErrorCode::NoCode,
			primary: None,
			labels: vec![],
			notes: vec![],
		}
	}
}

impl FluxError {
	pub fn with_msg(mut self, msg: String) -> FluxError {
		self.msg = msg;
		self
	}

	pub fn with_code(mut self, code: FluxErrorCode) -> FluxError {
		self.code = code;
		self
	}

	pub fn with_primary(mut self, msg: String, span: Option<Span>) -> FluxError {
		self.primary = Some((msg, span));
		self
	}

	pub fn with_label(mut self, msg: String, span: Option<Span>) -> FluxError {
		self.labels.push((msg, span));
		self
	}

	pub fn with_labels(mut self, labels: &mut Vec<(String, Option<Span>)>) -> FluxError {
		self.labels.append(labels);
		self
	}

	pub fn with_note(mut self, msg: String) -> FluxError {
		self.notes.push(msg);
		self
	}

	pub fn prefix_msg(mut self, msg: String) -> FluxError {
		self.msg = format!("{}{}", msg, self.msg);
		self
	}

	pub fn to_diagnostic(&self) -> Diagnostic<filesystem::FileId> {
		let mut labels: Vec<Label<filesystem::FileId>> = vec![];
		if let Some(ref primary) = self.primary {
			if let Some(ref span) = primary.1 {
				labels
					.push(Label::primary(span.file_id, span.range.clone()).with_message(primary.0.clone()));
			} else {
				labels.push(Label::primary(FileId(0), 0..0).with_message(primary.0.clone()));
			}
		}
		for (msg, span) in &self.labels {
			if let Some(span) = span {
				labels.push(Label::secondary(span.file_id, span.range.clone()).with_message(msg));
			} else {
				labels.push(Label::secondary(FileId(0), 0..0).with_message(msg));
			}
		}

		Diagnostic::error()
			.with_message(self.msg.clone())
			.with_code(self.code.to_string())
			.with_labels(labels)
			.with_notes(self.notes.clone())
	}
}

pub struct FluxErrorReporting {
	files: filesystem::Files,
	writer: StandardStream,
	config: Config,
}

impl Default for FluxErrorReporting {
	fn default() -> Self {
		let files = filesystem::Files::default();
		let writer = StandardStream::stderr(ColorChoice::Always);
		let config = codespan_reporting::term::Config::default();
		Self {
			files,
			writer,
			config,
		}
	}
}

impl FluxErrorReporting {
	pub fn add_file(&mut self, name: String, source: String) -> Option<filesystem::FileId> {
		self.files.add(name, source)
	}

	pub fn get_filename(&mut self, file_id: FileId) -> String {
		match self.files.get(file_id) {
			Ok(x) => x.name.clone(),
			_ => "illegal".to_owned(),
		}
	}

	pub fn get_file_id(&self, filename: &str) -> FileId {
		for (i, file) in self.files.files.iter().enumerate() {
			if file.name == *filename {
				return FileId(i as u32);
			}
		}
		FileId(0)
	}

	pub fn report(&self, errs: &[FluxError]) {
		for err in errs {
			let writer = &mut self.writer.lock();
			let _ = term::emit(writer, &self.config, &self.files, &err.to_diagnostic());
		}
	}
}
