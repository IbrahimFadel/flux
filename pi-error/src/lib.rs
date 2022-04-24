use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};
use filesystem::FileId;
use std::ops::Range;

pub mod filesystem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PIErrorCode {
	LexUnknownChar,
	LexFloatInWrongBase,
	LexStringLitUnterminated,
	LexCharLitUnterminated,
	LexUnknownEscapeSequence,
	LexInvalidCharLit,
	LexMissingEndOfBlockComment,
	LexExpectedDigitsFollowingIntPrefix,

	ParseExpectedIdent,
	ParseExpectedTopLevelDecl,
	ParseExpectedIdentFnDecl,
	ParseExpectedCommaInGenericTypeList,
	ParseExpectedGTAfterGenericTypeList,
	ParseExpectedCommaInParamList,
	ParseCouldNotConvertTokKindToPrimitiveType,
	ParseExpectedLBraceInBlock,
	ParseExpectedRBraceInBlock,
	ParseExpectedRParenAfterParamList,
	ParseExpectedLParenBeforeParamList,
	ParseUnexpectedEOF,
	ParseExpectedIdentGenericTypeList,
	ParseExpectedTypeExpr,
	ParseExpectedIdentVarDecl,
	ParseExpectedCommaInVarDeclIdentList,
	ParseExpectedEqVarDeclIdentList,
	ParseUnexpectedExprOperand,
	ParseExpectedBasicLit,
	ParseCouldNotParseInt,
	ParseMoreValsThanIdentsVarDecl,
	ParseMoreIdentsThanValsVarDecl,
	ParseExpectedSemicolonAfterVarDecl,
	ParseExpectedExprAfterCommaVarDeclValueList,
	ParseExpectedSemicolonAfterReturnStmt,
	ParseExpectedLParenBeforeCallExpr,
	ParseExpectedCommaInCallArgs,
	ParseExpectedRParenAfterCallExpr,
	ParseExpectedSemicolonAfterExpr,
	ParseExpectedTypeInTypeDecl,
	ParseExpectedLBraceInStructTypeExpr,
	ParseExpectedRBraceInStructTypeExpr,
	ParseExpectedIdentInField,
	ParseExpectedEqInField,
	ParseExpectedSemicolonInField,
	ParseExpectedStructInStructTypeExpr,
	ParseExpectedInterfaceInInterfaceTypeExpr,
	ParseExpectedLBraceInInterfaceTypeExpr,
	ParseExpectedRBraceInInterfaceTypeExpr,
	ParseExpectedSemicolonAfterMethodInInterfaceTypeMethodList,
	ParseExpectedSemicolonAfterTypeDecl,
	ParseExpectedSemicolonAfterModStmt,
	ParseExpectedIdentAfterApply,
	ParseExpectedIdentAfterTo,
	ParseExpectedLBraceInApplyBlock,
	ParseExpectedFnOrRBraceInApplyBlock,
	ParseExpectedRBraceAfterApplyBlock,
	ParseUnexpectedThisOutsideApply,
	ParseExpectedFnInInterfaceMethod,
	ParseExpectedCommaOrRBraceStructExpr,
	ParseUnexpectedTokenStructExpr,

	TypecheckUnexpectedSignednessInIntLit,
	TypecheckExpectedPeriodOpInChainedStructFieldAccess,
	TypecheckExpectedRHSOfPeriodToBeIdent,
	TypecheckExpectedLHSOfPeriodToBeStruct,
	TypecheckStructDoesNotHaveField,
	TypecheckCouldNotFindType,
	TypecheckExpectedTypeToBeInterface,
	TypecheckCouldNotFindMethodInInterface,
	TypecheckInterfaceMethodRetTyDontMatch,
	TypecheckInterfaceMethodParamsDontMatch,
	TypecheckInterfaceMethodVisibilitiesDontMatch,
	TypecheckNotAllInterfaceMethodsImplemented,
	TypecheckStructExprDiffNumberFieldsAsStructTy,
	TypecheckCouldNotFindFieldInStructExpr,
	TypecheckCouldNotGetTypeOfVar,
	TypecheckExpectedIntGotFloat,

	CodegenUnknownIdentType,
	CodegenUnknownVarReferenced,
	CodegenCouldNotCodegenStmt,
	CodegenCouldNotFindMethod,
	CodegenCouldNotCmpValsOfType,
	CodegenCouldNotCodegenTypeExpr,
	CodegenUnknownFnReferenced,
}

impl std::fmt::Display for PIErrorCode {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "E{:04}", *self as u8)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
	pub range: Range<usize>,
	pub file_id: FileId,
}

impl Span {
	pub fn new(range: Range<usize>, file_id: FileId) -> Span {
		Span { range, file_id }
	}
}

#[derive(Debug, PartialEq)]
pub struct PIError {
	msg: String,
	pub code: PIErrorCode,
	labels: Vec<(String, Span)>,
}

impl PIError {
	pub fn new(msg: String, code: PIErrorCode, labels: Vec<(String, Span)>) -> PIError {
		PIError { msg, code, labels }
	}

	pub fn to_diagnostic(&self) -> Diagnostic<filesystem::FileId> {
		let mut labels: Vec<Label<filesystem::FileId>> = vec![];
		for i in 0..self.labels.len() {
			if i == 0 {
				labels.push(
					Label::primary(self.labels[i].1.file_id, self.labels[i].1.range.clone())
						.with_message(self.labels[i].0.clone()),
				);
			} else {
				labels.push(
					Label::secondary(self.labels[i].1.file_id, self.labels[i].1.range.clone())
						.with_message(self.labels[i].0.clone()),
				);
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

impl PIErrorReporting {
	pub fn new() -> PIErrorReporting {
		let files = filesystem::Files::new();
		let writer = StandardStream::stderr(ColorChoice::Always);
		let config = codespan_reporting::term::Config::default();
		PIErrorReporting {
			files,
			writer,
			config,
		}
	}

	pub fn add_file(&mut self, name: String, source: String) -> Option<filesystem::FileId> {
		self.files.add(name, source)
	}

	pub fn get_filename(&mut self, file_id: FileId) -> String {
		match self.files.get(file_id) {
			Ok(x) => x.name.clone(),
			_ => "illegal".to_owned(),
		}
	}

	pub fn get_file_id(&self, filename: &String) -> FileId {
		for (i, file) in self.files.files.iter().enumerate() {
			if file.name == *filename {
				return FileId(i as u32);
			}
		}
		return FileId(0);
	}

	pub fn report(&self, errs: &Vec<PIError>) {
		for err in errs {
			let writer = &mut self.writer.lock();
			let _ = term::emit(writer, &self.config, &self.files, &err.to_diagnostic());
		}
	}
}
