use std::{collections::HashMap, fs};

use pi_ast::AST;
use pi_cfg::*;
use pi_error::{filesystem::FileId, *};
use pi_lexer::*;
use pi_mir::*;
use pi_parser::*;

fn parse_file(
	project_dir: String,
	file_name: String,
	file_ast_map: &mut HashMap<FileId, AST>,
	err_reporting: &mut PIErrorReporting,
) {
	let path = project_dir.clone() + "src/" + file_name.as_str() + ".pi";
	let input = fs::read_to_string(path.clone()).expect(&format!("could not read file: {}", &path));
	let file_id = err_reporting
		.add_file(path, input.clone())
		.expect("could not add file");
	let (toks, errs) = tokenize(&input, file_id);
	err_reporting.report(errs);
	let (ast, errs) = parse_tokens(file_name.to_owned(), &input, toks, file_id);
	err_reporting.report(errs);

	for mod_ in &ast.mods {
		parse_file(
			project_dir.clone(),
			mod_.name.to_string(),
			file_ast_map,
			err_reporting,
		)
	}
	file_ast_map.insert(file_id, ast);
}

fn main() {
	let project_dir = String::from("examples/crate-1/");
	let _cfg = parse_cfg(project_dir.as_str());

	// let dependency_file_paths: Vec<String> = cfg
	// 	.dependencies
	// 	.into_iter()
	// 	.map(|dep| match dep.1 {
	// 		pi_cfg::Dependency::Simple(version) => {
	// 			"tau.io/".to_owned() + dep.0.as_str() + "@" + version.as_str()
	// 		}
	// 		pi_cfg::Dependency::Detailed(details) => match details.path {
	// 			Some(x) => x,
	// 			_ => "illegal".to_owned(),
	// 		},
	// 	})
	// 	.collect();

	// // for path in &dependency_file_paths {
	// 	let relative_path = project_dir.to_owned() + path + "/src/lib.pi";
	// 	let absolute_path =
	// 		fs::canonicalize(relative_path).expect("could not canonicalize dependency path");
	// 	let input = fs::read_to_string(absolute_path).expect("could not read file");
	// 	let mut err_reporting = PIErrorReporting::new();
	// 	let file_id = err_reporting
	// 		.add_file(project_dir.to_owned() + "src/main.pi", input.clone())
	// 		.expect("could not add file");
	// 	let (toks, errs) = tokenize(&input, file_id);
	// 	err_reporting.report(errs);
	// 	let (_fns, errs) = parse_tokens(&input, toks, file_id);
	// 	err_reporting.report(errs);
	// }

	let mut file_ast_map: HashMap<FileId, AST> = HashMap::new();
	let mut err_reporting = PIErrorReporting::new();

	parse_file(
		project_dir,
		"main".to_owned(),
		&mut file_ast_map,
		&mut err_reporting,
	);

	// println!("{:#?}", file_ast_map);

	generate_mir(&file_ast_map, &mut err_reporting);
}
