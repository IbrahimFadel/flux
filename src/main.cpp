#include <iostream>
#include <fstream>
#include <istream>

#include "options.h"
#include "common.h"
#include "lexer.h"
#include "parser.h"
#include "dependency_tree.h"
#include "code_generation.h"

using std::cout;
using std::endl;

int main(int argc, const char **argv)
{
  CompilerOptions options;

  std::vector<std::string> arguments(argv, argv + argc);
  int i = 0;
  for (auto &arg : arguments)
  {
    if (arg == "--optimize")
      options.optimize = true;
    else if (arg == "--output" || arg == "-o")
      options.output_path = std::string(arguments[i + 1]);
    else if (arg == "--print" || arg == "-p")
      options.print_module = true;
    i++;
  }

  std::string file_path = argv[1];
  auto file_name = fs::canonical(file_path).filename().string();

  auto file_content = get_file_content(file_path.c_str());
  auto tokens = tokenize(file_content);
  // print_tokens(tokens);

  auto nodes = parse_tokens(tokens);
  // print_nodes(nodes);

  auto dependency_tree = generate_dependency_tree(nodes, file_path);
  print_deependency_tree(dependency_tree);

  llvm::LLVMContext context;
  auto module = new llvm::Module(file_name, context);
  create_module(std::move(nodes), options, file_path, dependency_tree, module);
  // print_module(module);
  write_module_to_file(module, "../out.ll");

  // auto program = std::make_unique<Program>("test");
  // auto module = code_gen_nodes(std::move(nodes), options, std::move(program));

  // module_to_obj(std::move(module));

  return 0;
}
