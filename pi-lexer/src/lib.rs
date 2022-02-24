use std::{borrow::BorrowMut, str::Chars};

use codespan::{ByteIndex, ByteOffset, RawIndex, Span};
use nom::AsChar;
use token::{lookup_keyword, Token, TokenKind};
mod token;
use pi_error::{PIError, PIErrorCode};

#[derive(Debug, Clone, Copy)]
enum Mode {
    Normal,
    String,
    LineComment,
    BlockComment,
}

#[derive(Debug)]
struct PISpan<'a> {
    offset: usize,
    line: u32,
    fragment: &'a str,
    errors: &'a mut Vec<PIError>,
}

impl PISpan<'_> {
    // pub fn new(src: &str) -> PISpan {
    //     PISpan {
    //         offset: 0,
    //         line: 1,
    //         fragment: src,
    //         errors: vec![],
    //     }
    // }

    pub fn to_span(&self) -> Span {
        let start = self.offset() as u32;
        let end = start + self.fragment().len() as u32;
        Span::new(start, end)
    }

    pub fn fragment(&self) -> &str {
        self.fragment
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn ch(&self) -> char {
        match self.fragment.chars().nth(self.offset) {
            Some(x) => x,
            _ => '\0',
        }
    }

    pub fn next(&mut self) {
        self.offset += 1;
    }

    pub fn peek(&self) -> char {
        match self.fragment.chars().nth(self.offset + 1) {
            Some(x) => x,
            _ => '\0',
        }
    }
}

pub fn tokenize(program: &str) {
    // let src = PISpan::new(program);
    let mut errors = vec![];
    let src = PISpan {
        offset: 0,
        line: 1,
        fragment: program,
        errors: &mut errors,
    };
    let mode = Mode::Normal;

    let toks = std::iter::from_fn(move || {
        let _ = get_tok(src, mode)?;
        // src = rest;
        // println!("{}", src);

        // Some(out)
        // Some(Token::new(TokenKind::LineComment, src.to_span()))
        Some(1)
    });
    let arr: Vec<u32> = toks.collect();
    println!("{:?}", arr);
}

fn get_tok<'a>(src: PISpan, mode: Mode) -> Option<(PISpan, Token)> {
    let mut tok = Token::new();
    match mode {
        Mode::Normal => {
            scan_whitespace(src);

            let ch = src.ch();
            if ch.is_alpha() {
                tok.span = scan_ident(src);
                if tok.span.to_string().len() > 1 {
                    tok.kind = lookup_keyword(tok.span.to_string().as_str());
                } else {
                    tok.kind = TokenKind::Ident;
                }
            } else if ch.is_digit(10) || (ch == '.' && src.peek().is_digit(10)) {
                tok = scan_number(src);
            }

            Some((src, tok))
        }
        _ => None,
    }
}

fn scan_number(mut src: PISpan) -> Token {
    let initial_offset = src.offset();
    let mut tok = Token::new();

    let mut base: u8 = 10;

    if src.ch() != '.' {
        tok.kind = TokenKind::Int;
        if src.ch() == '0' {
            src.next();
            match src.ch() {
                'x' => base = 16,
                '8' => base = 8,
                'b' => base = 2,
                _ => base = 10,
            }
        }

        scan_digits(src, base);
    }

    if src.ch() == '.' {
        tok.kind = TokenKind::Float;
        if base != 10 {
            src.errors.push(PIError {
                msg: "invalid digit",
                code: PIErrorCode::LexInvalidDigit,
            })
        }
    }

    return tok;
}

fn scan_digits(mut src: PISpan, base: u8) {
    while src.ch().is_digit(base as u32) {
        src.next();
    }
}

fn scan_ident(mut src: PISpan) -> Span {
    let initial_offset = src.offset();
    src.next();
    while src.ch().is_alpha() || src.ch().is_digit(10) {
        src.next();
    }
    return Span::new(initial_offset as u32, src.offset as u32);
}

fn scan_whitespace(mut src: PISpan) {
    loop {
        match src.ch() {
            ' ' | '\t' | '\n' | '\r' => src.next(),
            _ => (),
        }
    }
}
