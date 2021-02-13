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

  auto main_file_content = get_file_content(file_path.c_str());
  auto main_tokens = tokenize(main_file_content);
  // print_tokens(main_tokens);

  auto main_nodes = parse_tokens(main_tokens);
  // print_nodes(nodes);

  auto dependency_tree = generate_dependency_tree(main_nodes, file_path);
  // print_deependency_tree(dependency_tree);

  llvm::LLVMContext context;
  for (auto &node : dependency_tree->nodes)
  {
    auto module = new llvm::Module(node.first.filename().string(), context);
    auto file_content = get_file_content(node.first.c_str());
    auto tokens = tokenize(file_content);
    auto nodes = parse_tokens(tokens);

    create_module(nodes, options, node.first, dependency_tree, module);

    print_module(module);
    write_module_to_file(module, "../main.ll");

    // module_to_obj(module, node.first.replace_extension("o").string());
  }
  // std::string files;
  // for (auto &node : dependency_tree->nodes)
  // {
  //   files += node.first.string() + " ";
  // }
  // std::string command = "clang " + files + "-o " + options.output_path;
  // std::system(command.c_str());

  // for (auto &node : dependency_tree->nodes)
  // {
  //   if (std::remove(node.first.c_str()))
  //   {
  //     cout << "Error removing object files" << endl;
  //     exit(1);
  //   }
  // }

  //! The entire process of linking the object files looks really gross (especially the system()) <- think about all this more

  return 0;
}
