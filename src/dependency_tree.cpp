#include "dependency_tree.h"

Dependency_Tree *generate_dependency_tree(const Nodes &nodes, std::string file_path)
{
    auto tree = new Dependency_Tree();

    auto resolved = fs::canonical(file_path);
    tree->nodes.push_back(std::make_pair(resolved, std::vector<Dependency_Function *>(0)));

    make_all_connections_with_path(0, resolved, tree);

    return tree;
}

void make_all_connections_with_path(int base_index, fs::path resolved_path, Dependency_Tree *tree)
{
    auto file_content = get_file_content(resolved_path.c_str());
    auto tokens = tokenize(file_content);
    auto left_ast_nodes = parse_tokens(tokens);
    bool follow_path = true;
    for (auto &left_ast_node : left_ast_nodes)
    {
        auto import_node = dynamic_cast<Import_Statement *>(left_ast_node.get());
        if (import_node != nullptr)
        {
            auto path = resolve_path(import_node->get_path(), resolved_path.parent_path());

            for (auto &conn : tree->connections)
            {
                if (tree->nodes[conn.first].first == path)
                    follow_path = false;
            }

            auto file_content = get_file_content(path.c_str());
            auto tokens = tokenize(file_content);
            auto right_ast_nodes = parse_tokens(tokens);
            std::vector<Dependency_Function *> fn_dep_nodes;

            for (auto &right_ast_node : right_ast_nodes)
            {
                auto fn_node = dynamic_cast<Function_Declaration *>(right_ast_node.get());
                if (fn_node != nullptr)
                {
                    auto dep_fn = new Dependency_Function();
                    dep_fn->name = fn_node->get_name();
                    std::vector<std::string> param_types;
                    auto params = fn_node->get_params();
                    for (auto it = params.begin(), end = params.end(); it != end; it++)
                    {
                        param_types.push_back(it->second);
                    }
                    dep_fn->param_types = param_types;
                    dep_fn->return_type = fn_node->get_return_type();
                    fn_dep_nodes.push_back(dep_fn);
                }
            }

            tree->nodes.push_back(std::make_pair(path, fn_dep_nodes));
            tree->connections.push_back(std::make_pair(base_index, tree->nodes.size() - 1));
            if (follow_path)
                make_all_connections_with_path(base_index + 1, path, tree);
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
        cout << fs::relative(tree->nodes[conn.first].first).string() << " connected to " << fs::relative(tree->nodes[conn.second].first).string() << endl;
    }
}