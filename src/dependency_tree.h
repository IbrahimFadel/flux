#ifndef DEPENDENCY_TREE_H
#define DEPENDENCY_TREE_H

#include "common.h"
#include "lexer.h"
#include "ast.h"
#include "parser.h"

#include <filesystem>
#include <map>

namespace fs = std::filesystem;

using std::unique_ptr;

struct Dependency_Function
{
    std::string name;
    std::vector<std::string> param_types;
    std::string return_type;
};

struct Dependency_Tree
{
    std::vector<std::pair<fs::path, std::vector<Dependency_Function *>>> nodes;
    std::vector<std::pair<int, int>> connections;
};

Dependency_Tree *generate_dependency_tree(const Nodes &nodes, std::string file_path);
static void make_all_connections_with_path(int base_index, fs::path resolved_path, Dependency_Tree *tree);
static fs::path resolve_path(std::string path, std::string base);
void print_deependency_tree(Dependency_Tree *tree);

#endif