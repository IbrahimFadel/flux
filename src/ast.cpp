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
    cout << "Calling: " << name << endl;
    cout << "Args: " << endl;
    auto it = parameters.begin();
    for (auto &param : parameters)
    {
        param->print();
        cout << ' ';
    }

    cout << endl;
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

void Object_Node::print()
{
    cout << "Defining object type with name: " << name << endl;
    cout << "Properties: " << endl;
    for (auto &property : properties)
    {
        auto name = std::get<0>(property);
        auto type = std::get<1>(property);
        cout << "name=" << name << " type=" << type << endl;
    }
}

void Object_Expression::print()
{
    cout << endl;
    auto it = properties.begin();
    while (it == properties.end())
    {
        cout << it->first << " ";
        it->second->print();
        it++;
    }
}

void String_Expression::print()
{
    cout << value << endl;
}

void Return_Node::print()
{
    cout << "Returning: ";
    value->print();
}

void Node::set_node_type(Node_Type type) { node_type = type; };

std::string Prototype_Node::get_name() { return name; };
Variable_Type Prototype_Node::get_return_type() { return return_type; };

void Function_Node::set_variable_ptr(std::string name, Value *ptr) { variable_ptrs[name] = ptr; };
void Function_Node::set_variable_value(std::string name, Value *v) { variable_values[name] = v; };
Value *Function_Node::get_variable_value(std::string name) { return variable_values[name]; };

std::string Variable_Declaration_Node::get_type_name() { return type_name; }
Variable_Type Variable_Declaration_Node::get_type() { return type; };
std::string Variable_Declaration_Node::get_name() { return name; };
unique_ptr<Node> Variable_Declaration_Node::get_value() { return std::move(value); };

std::map<std::string, unique_ptr<Node>> Object_Expression::get_properties() { return std::move(properties); };

std::map<std::string, unique_ptr<Node>> Node::get_properties()
{
    std::map<std::string, unique_ptr<Node>> p;
    return std::move(p);
};
