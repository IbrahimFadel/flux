use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};
use filesystem::FileId;
use text_size::TextRange;

pub mod filesystem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PIErrorCode {
	NoCode,
	UnexpectedEOF,
	UnexpectedToken,

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

impl std::fmt::Display for PIErrorCode {
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
pub struct PIError {
	msg: String,
	code: PIErrorCode,
	primary: (String, Option<Span>),
	labels: Vec<(String, Option<Span>)>,
}

impl Default for PIError {
	fn default() -> Self {
		Self {
			msg: String::new(),
			code: PIErrorCode::NoCode,
			primary: (String::new(), None),
			labels: vec![],
		}
	}
}

impl PIError {
	pub fn with_msg(mut self, msg: String) -> PIError {
		self.msg = msg;
		self
	}

	pub fn with_code(mut self, code: PIErrorCode) -> PIError {
		self.code = code;
		self
	}

	pub fn with_primary(mut self, msg: String, span: Option<Span>) -> PIError {
		self.primary = (msg, span);
		self
	}

	pub fn with_label(mut self, msg: String, span: Option<Span>) -> PIError {
		self.labels.push((msg, span));
		self
	}

	pub fn to_diagnostic(&self) -> Diagnostic<filesystem::FileId> {
		let mut labels: Vec<Label<filesystem::FileId>> = vec![];
		if let Some(span) = &self.primary.1 {
			labels.push(
				Label::secondary(span.file_id, span.range.clone()).with_message(self.primary.0.clone()),
			);
		} else {
			labels.push(Label::secondary(FileId(0), 0..0).with_message(self.primary.0.clone()));
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
	}
}

pub struct PIErrorReporting {
	files: filesystem::Files,
	writer: StandardStream,
	config: Config,
}

impl Default for PIErrorReporting {
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

impl PIErrorReporting {
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

	pub fn report(&self, errs: &[PIError]) {
		for err in errs {
			let writer = &mut self.writer.lock();
			let _ = term::emit(writer, &self.config, &self.files, &err.to_diagnostic());
		}
	}
}
