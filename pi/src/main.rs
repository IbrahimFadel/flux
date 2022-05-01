use std::fs;

// use indexmap::IndexMap;
use pi_syntax::{ast, ast::AstNode};
// use pi_cfg::*;
// use pi_codegen::lower_mir_module;
use pi_error::{filesystem::FileId, *};
// use pi_lexer::*;
// use pi_mir::*;
// use pi_codegen::*;
use pi_hir::*;
use pi_parser::*;
// use pi_typecheck::*;

// fn parse_file(
// 	project_dir: String,
// 	file_name: String,
// 	file_ast_map: &mut IndexMap<FileId, AST>,
// 	err_reporting: &mut PIErrorReporting,
// ) {
// 	let path = project_dir.clone() + "src/" + file_name.as_str() + ".pi";
// 	let input = fs::read_to_string(path.clone()).expect(&format!("could not read file: {}", &path));
// 	let file_id = err_reporting
// 		.add_file(path, input.clone())
// 		.expect("could not add file");
// 	let (toks, errs) = tokenize(&input, file_id);
// 	err_reporting.report(&errs);
// 	let (ast, errs) = parse_tokens(file_name.to_owned(), input, toks, file_id);
// 	err_reporting.report(&errs);

// 	for mod_ in &ast.mods {
// 		parse_file(
// 			project_dir.clone(),
// 			mod_.name.to_string(),
// 			file_ast_map,
// 			err_reporting,
// 		)
// 	}
// 	file_ast_map.insert(file_id, ast);
// }

fn main() {
	let src = fs::read_to_string("examples/main.pi").unwrap();
	let cst = parse(src.as_str(), FileId(0));

	let root = ast::Root::cast(cst.syntax()).unwrap();
	println!("{}", cst.debug_tree());

	let (db, functions) = pi_hir::lower(root);
	println!("{:#?}", db);
	println!("{:#?}", functions);

	// let g = ungrammar::Grammar;
	// let s = fs::read_to_string("pi.ungram").unwrap();
	// let grammar: ungrammar::Grammar = s.parse().unwrap();
	// println!("{:#?}", grammar);

	// let res = parse("hi");
	// println!("{:#?}", res);

	// let project_dir = String::from("examples/crate-1/");
	// let cfg = parse_cfg(project_dir.as_str());

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

	// let mut file_ast_map: IndexMap<FileId, AST> = IndexMap::new();
	// let mut err_reporting = PIErrorReporting::new();

	// parse_file(
	// 	project_dir.clone(),
	// 	"main".to_owned(),
	// 	&mut file_ast_map,
	// 	&mut err_reporting,
	// );

	// for (id, ast) in file_ast_map.iter() {
	// 	let path = project_dir.clone() + "ast" + &id.0.to_string() + ".txt";
	// 	let _ = fs::write(path, ast.to_string());
	// }

	// let typecheck_result = typecheck_ast(&mut file_ast_map, &err_reporting);

	// for (id, ast) in file_ast_map.iter() {
	// 	let path = project_dir.clone() + "ast_typechecked" + &id.0.to_string() + ".txt";
	// 	let _ = fs::write(path, ast.to_string());
	// }

	// if let Some(err) = typecheck_result.err() {
	// 	err_reporting.report(&Vec::from([err]));
	// 	process::exit(1);
	// }

	// for (_, ast) in file_ast_map.iter() {
	// 	let mir_module = lower_ast(ast);
	// 	lower_mir_module(mir_module, &cfg.compilation);
	// }

	// let (codegen_ctx, err) = codegen_ast(&mut file_ast_map, &cfg.compilation);
	// if let Some(err) = err {
	// 	err_reporting.report(&Vec::from([err]));
	// }
	// let path = project_dir + "module.ll";
	// codegen_ctx.write_to_file(&path);
	// codegen_ctx.dispose();
}
