#[macro_export]
#[cfg(test)]
macro_rules! test_file {
	($name:ident, $path:literal) => {
		paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = FileId(0);
					let source = fs::read_to_string($path).unwrap();
					let (toks, errs) = tokenize(source.as_str(), file_id);
					assert!(errs.is_empty());

					let mut input = ParseInput::new(String::from(source), &toks, file_id);

					let (ast, errs) = parse_tokens(String::from($name), source, toks, file_id);
					assert!(errs.is_empty());

					let mut settings = insta::Settings::clone_current();
					settings.set_sort_maps(true);
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_yaml_snapshot!(&ast);
					});
				}
		}
	};
}

#[macro_export]
#[cfg(test)]
macro_rules! test_expr_str {
	($name:ident, $source:literal) => {
		paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = FileId(0);
					let (toks, errs) = tokenize($source, file_id);
					assert!(errs.is_empty());

					let mut input = ParseInput::new(String::from($source), &toks, file_id);

					let expr = expr(&mut input);
					assert!(input.errs.is_empty());

					let mut output_str = String::new();
					let _ = write!(output_str, "{:#?}", expr);

					let mut settings = insta::Settings::clone_current();
					settings.set_sort_maps(true);
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_yaml_snapshot!(&expr);
					});
				}
		}
	};
}

#[macro_export]
#[cfg(test)]
macro_rules! test_type_expr_str {
	($name:ident, $source:literal) => {
		paste! {
				#[test]
				fn [<test_parse_ $name>]() {
					let file_id = FileId(0);
					let (toks, errs) = tokenize($source, file_id);
					assert!(errs.is_empty());

					let mut input = ParseInput::new(String::from($source), &toks, file_id);

					let expr = type_expr(&mut input);
					assert!(input.errs.is_empty());

					let mut settings = insta::Settings::clone_current();
					settings.set_sort_maps(true);
					settings.set_snapshot_path("./snapshots");
					settings.bind(|| {
						insta::assert_yaml_snapshot!(&expr);
					});
				}
		}
	};
}
