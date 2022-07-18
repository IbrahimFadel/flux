use text_size::{TextRange, TextSize};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

pub fn position_to_offset(pos: &Position, src: &str) -> u32 {
	let mut start_offset = 0;
	let mut new_lines = 0;
	for c in src.chars() {
		if c == '\n' {
			new_lines += 1;
			if new_lines == pos.line {
				break;
			}
		}
		start_offset += 1;
	}

	eprintln!("{:?}", pos);
	eprintln!("{}", start_offset);

	start_offset
}

pub fn flux_range_to_position(range: TextRange, src: &str) -> Range {
	let mut start_line_number = 0;
	let mut start_character = 0;
	let mut offset = 0;
	for c in src.chars() {
		if TextSize::from(offset) == range.start() {
			break;
		}
		if c == '\n' {
			start_line_number += 1;
			start_character = 0;
		} else {
			start_character += 1;
		}
		offset += 1;
	}
	let mut end_line_number = 0;
	let mut end_character = 0;
	for i in offset..u32::from(range.end()) {
		let c = src
			.chars()
			.nth(i as usize)
			.expect(&format!("expected character at index {}", i));
		if TextSize::from(offset) == range.end() {
			break;
		}
		if c == '\n' {
			end_line_number += 1;
			end_character = 0;
		} else {
			end_character += 1;
		}
		offset += 1;
	}
	Range {
		start: Position {
			line: start_line_number,
			character: start_character,
		},
		end: Position {
			line: end_line_number + start_line_number,
			character: end_character,
		},
	}
}

// pub fn flux_span_to_location(uri: &Url, span: &flux_error::Span, src: &str) -> Location {
// 	let range = flux_range_to_position(span.range, src);
// 	Location {
// 		uri: uri.clone(),
// 		range,
// 	}
// }
