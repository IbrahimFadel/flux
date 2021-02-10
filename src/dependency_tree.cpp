#include "dependency_tree.h"

Dependency_Tree *generate_dependency_tree(const Nodes &nodes, std::string file_path)
{
    auto tree = new Dependency_Tree();

    auto resolved = fs::canonical(file_path);
    tree->nodes.push_back(resolved);

    construct_new_dependency_node(0, resolved, tree);

    return tree;
}

void construct_new_dependency_node(int base_index, fs::path resolved_path, Dependency_Tree *tree)
{
    auto file_content = get_file_content(resolved_path.c_str());
    auto tokens = tokenize(file_content);
    auto ast_nodes = parse_tokens(tokens);
    for (auto &node : ast_nodes)
    {
        auto import_node = dynamic_cast<Import_Statement *>(node.get());
        if (import_node != nullptr)
        {
            auto path = resolve_path(import_node->get_path(), resolved_path.parent_path());
            bool connection_already_made = false;

            for (auto &conn : tree->connections)
            {
                if (tree->nodes[conn.first] == path)
                    connection_already_made = true;
            }

            tree->nodes.push_back(path);
            tree->connections.push_back(std::make_pair<int, int>((int)base_index, tree->nodes.size() - 1));

            if (!connection_already_made)
                construct_new_dependency_node(base_index + 1, path, tree);
        }
    }
}

fs::path resolve_path(std::string path, std::string base)
{
    std::error_code ec;
    auto file = fs::canonical(std::string(base + "/" + path), ec);
    if (ec)
    {
        cout << "Error parsing dependency tree at file path: " << path << endl;
        cout << ec.message() << endl;
        exit(1);
    }
    return file;
}

void print_deependency_tree(Dependency_Tree *tree)
{
    for (auto &conn : tree->connections)
    {
        cout << tree->nodes[conn.first].string() << " connected to " << tree->nodes[conn.second].string() << endl;
    }
}