#ifndef DEPENDENCY_TREE_H
#define DEPENDENCY_TREE_H

#include "common.h"
#include "lexer.h"
#include "ast.h"
#include "parser.h"

#include <filesystem>
#include <map>

using std::unique_ptr;

struct Dependency_Node
{
    std::vector<Dependency_Node *> dependencies;
    Dependency_Node *parent;
    std::string file_path;
    int depth = 0;
    void print();
};

struct Dependency_Tree
{
    Dependency_Node *head;
    std::string base_file_path;
};

unique_ptr<Dependency_Tree> generate_dependency_tree(const Nodes &nodes, std::string file_path);
static Dependency_Node *construct_new_dependency_node(std::string file_path, Dependency_Node *parent_dependency_node);
static std::map<std::string, Dependency_Node *> parsed_files;
static int depth = 1; //? This is just for printing the tree

#endif