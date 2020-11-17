#include "ast.h"

void print_nodes(const Nodes &nodes)
{
    for (auto &node : nodes)
    {
        node->print();
    }
}

void Function_Node::print()
{
    cout << "Function Node" << endl;
    cout << endl;
    prototype->print();
    then->print();
}

void Number_Expression_Node::print()
{
    cout << value;
}

void Variable_Reference_Node::print()
{
    cout << name;
}

void Binary_Operation_Expression_Node::print()
{
    lhs->print();
    cout << op;
    rhs->print();
    cout << endl;
}

void Then_Node::print()
{
    cout << "Then" << endl;
    for (auto &node : nodes)
    {
        node->print();
    }
    cout << endl;
}

void Variable_Declaration_Node::print()
{
    cout << "Variable Declaration: type=" << type << " name=" << name << " value=";
    value->print();
    cout << endl;
}

void Prototype_Node::print()
{
    cout << "Prototype Node" << endl;
    cout << "Name: " << name << endl;
    for (int i = 0; i < param_types.size(); i++)
    {
        cout << "Param " << i + 1 << ": "
             << "type=" << param_types[i] << " name=" << param_names[i] << endl;
    }
    cout << endl;
}

void If_Node::print()
{
    cout << "If Statement" << endl;
    if (condition_separators.size() > 0)
    {
        for (int i = 0; i < condition_separators.size(); i++)
        {
            conditions[i]->print();
            cout << condition_separators[i] << endl;
            conditions[i + 1]->print();
        }
    }
    else
    {
        for (auto &condition : conditions)
        {
            condition->print();
        }
    }

    then->print();
}

void Condition_Node::print()
{
    lhs->print();
    cout << ' ';
    cout << op;
    cout << ' ';
    rhs->print();
    cout << endl;
}

void Function_Call_Node::print()
{
}

void Import_Node::print()
{
    cout << "Import Node" << endl;
    cout << "Path: " << path << endl;
}

void For_Node::print()
{
    cout << "For Loop" << endl;
    variable->print();
    condition->print();
    assignment->print();
    then->print();
}

void Variable_Assignment_Node::print()
{
    cout << "Assign " << name << " to ";
    value->print();
}

void Node::set_node_type(Node_Type type) { node_type = type; };
