#include "dependency_tree.h"

namespace fs = std::filesystem;

unique_ptr<Dependency_Tree> generate_dependency_tree(const Nodes &nodes, std::string file_path)
{
    auto tree = std::make_unique<Dependency_Tree>();
    auto resolved = fs::canonical(file_path);
    tree->base_file_path = resolved;

    auto head = new Dependency_Node();
    head->file_path = resolved;

    for (auto &node : nodes)
    {
        auto import_node = dynamic_cast<Import_Statement *>(node.get());
        if (import_node != nullptr)
        {
            auto dep_node = construct_new_dependency_node(import_node->get_path(), head);
            head->dependencies.push_back(dep_node);
        }
    }

    tree->head = head;
    return tree;
}

Dependency_Node *construct_new_dependency_node(std::string file_path, Dependency_Node *parent)
{
    auto dep_node = new Dependency_Node();
    dep_node->depth = parent->depth + 1;
    dep_node->parent = parent;

    std::string resolved_path;
    if (fs::path(file_path).is_relative())
    {
        auto parent_directory = fs::path(parent->file_path).parent_path();
        resolved_path = fs::canonical(std::string(parent_directory.string() + "/" + file_path)).string();
    }
    else
    {
        resolved_path = file_path;
    }

    dep_node->file_path = resolved_path;

    //? Don't get trapped in infinite loop when files import eachother -- need to think about the best way to do this, but for now this can work
    if (parent->parent)
    {
        if (parent->parent->file_path == resolved_path)
        {
            dep_node->dependencies = parent->parent->dependencies;
            return dep_node;
        }
    }

    //? This is just to save time, ie. don't parse a file again if it's already been parsed (copy the dependencies)
    auto parsed_files_it = parsed_files.begin();
    while (parsed_files_it != parsed_files.end())
    {
        if (parsed_files_it->first == resolved_path)
        {
            dep_node->dependencies = parsed_files_it->second->dependencies;
            return dep_node;
        }
        parsed_files_it++;
    }

    auto file_content = get_file_content(resolved_path.c_str());
    auto tokens = tokenize(file_content);
    auto ast_nodes = parse_tokens(tokens);

    for (auto &ast_node : ast_nodes)
    {
        auto import_node = dynamic_cast<Import_Statement *>(ast_node.get());
        if (import_node != nullptr)
        {
            auto dependency = construct_new_dependency_node(import_node->get_path(), dep_node);
            dep_node->dependencies.push_back(dependency);
        }
    }

    parsed_files[resolved_path] = dep_node;

    return dep_node;
}

void Dependency_Node::print()
{
    for (int i = 0; i < depth; i++)
        cout << '\t';
    cout << std::filesystem::relative(file_path).string() << "\n";
    for (auto &connected_node : dependencies)
    {
        connected_node->print();
    }
}