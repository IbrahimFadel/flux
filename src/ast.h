#ifndef AST_H
#define AST_H

#include <string.h>
#include <memory>
#include <vector>
#include <variant>
#include <map>

#include "common.h"
#include "lexer.h"

#include <llvm/IR/Value.h>

using namespace llvm;

using std::unique_ptr;

class Node;
class Expression_Node;
class Binary_Operation_Expression_Node;
class Number_Expression_Node;
class Function_Node;
class Prototype_Node;
class Then_Node;
class Variable_Declaration_Node;
class If_Node;
class Condition_Node;
class Function_Call_Node;
class Variable_Reference_Node;
class Import_Node;
class Variable_Assignment_Node;
class Object_Node;
class Object_Expression;
class String_Expression;
class Return_Node;

enum Node_Type
{
    ExpressionNode,
    FunctionNode
};

enum Variable_Type
{
    type_null = -1,
    type_i64,
    type_i32,
    type_i16,
    type_i8,
    type_float,
    type_double,
    type_bool,
    type_void,
    type_string,
    type_object
};

class Node
{
private:
    Node_Type node_type;
    unique_ptr<Node> node_value;

public:
    void set_node_type(Node_Type node_type);
    virtual Value *code_gen() = 0;
    virtual void print() = 0;
};

class Expression_Node : public Node
{
private:
    Node_Type node_type;

public:
    Value *code_gen();
};

class Binary_Operation_Expression_Node : public Node
{
private:
    std::string op;
    unique_ptr<Node> lhs, rhs;

public:
    Binary_Operation_Expression_Node(std::string op, unique_ptr<Node> lhs, unique_ptr<Node> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
    Value *code_gen();
    void print();
};

class Number_Expression_Node : public Node
{
private:
    double value;
    Variable_Type variable_type = Variable_Type::type_null;

public:
    Number_Expression_Node(double value) : value(value){};
    Value *code_gen();
    void print();
};

class Function_Node : public Node
{
private:
    unique_ptr<Prototype_Node> prototype;
    unique_ptr<Then_Node> then;

    std::map<std::string, Value *> variable_ptrs;
    std::map<std::string, Value *> variable_values;

public:
    Function_Node(unique_ptr<Prototype_Node> prototype, unique_ptr<Then_Node> then) : prototype(std::move(prototype)), then(std::move(then)){};
    Value *code_gen();
    void print();

    void set_variable_ptr(std::string name, Value *ptr);
    void set_variable_value(std::string name, Value *v);

    Value *get_variable_value(std::string name);
};

class Prototype_Node : public Node
{
private:
    std::string name;
    std::vector<Variable_Type> param_types;
    std::vector<std::string> param_names;
    Variable_Type return_type;

public:
    Prototype_Node(std::string name, std::vector<Variable_Type> param_types, std::vector<std::string> param_names, Variable_Type return_type) : name(name), param_types(param_types), param_names(param_names), return_type(return_type){};
    Value *code_gen();
    Function *code_gen_proto();
    void print();
    void create_argument_allocas(Function *function);

    std::string get_name();
    Variable_Type get_return_type();
};

class Then_Node : public Node
{
private:
    std::vector<std::unique_ptr<Node>> nodes;

public:
    Then_Node(std::vector<std::unique_ptr<Node>> nodes) : nodes(std::move(nodes)){};
    Value *code_gen();
    void print();
};

class Variable_Declaration_Node : public Node
{
private:
    std::string name;
    Variable_Type type;
    unique_ptr<Node> value;

public:
    Variable_Declaration_Node(std::string name, Variable_Type type, unique_ptr<Node> value) : name(name), type(type), value(std::move(value)){};
    Value *code_gen();
    void print();
};

class If_Node : public Node
{
private:
    std::vector<unique_ptr<Condition_Node>> conditions;
    std::vector<Token_Type> condition_separators;
    unique_ptr<Then_Node> then;

public:
    If_Node(std::vector<unique_ptr<Condition_Node>> conditions, std::vector<Token_Type> condition_separators, unique_ptr<Then_Node> then) : conditions(std::move(conditions)), condition_separators(condition_separators), then(std::move(then)){};
    Value *code_gen();
    void print();
};

class Condition_Node : public Node
{
private:
    unique_ptr<Node> lhs;
    Token_Type op;
    unique_ptr<Node> rhs;

public:
    Condition_Node(unique_ptr<Node> lhs, Token_Type op, unique_ptr<Node> rhs) : lhs(std::move(lhs)), op(op), rhs(std::move(rhs)){};
    Value *code_gen();
    void print();
};

class Function_Call_Node : public Node
{
private:
    std::string name;
    std::vector<std::unique_ptr<Node>> parameters;

public:
    Function_Call_Node(std::string name, std::vector<std::unique_ptr<Node>> parameters) : name(name), parameters(std::move(parameters)){};
    Value *code_gen();
    void print();
};

class Variable_Reference_Node : public Expression_Node
{
private:
    std::string name;

public:
    Variable_Reference_Node(std::string name) : name(name){};
    Value *code_gen();
    void print();
};

class Import_Node : public Node
{
private:
    std::string path;

public:
    Import_Node(std::string path) : path(path){};
    Value *code_gen();
    void print();
};

class For_Node : public Node
{
private:
    std::unique_ptr<Variable_Declaration_Node> variable;
    std::unique_ptr<Node> condition;
    std::unique_ptr<Variable_Assignment_Node> assignment;
    std::unique_ptr<Then_Node> then;

public:
    For_Node(std::unique_ptr<Variable_Declaration_Node> variable, std::unique_ptr<Node> condition, std::unique_ptr<Variable_Assignment_Node> assignment, std::unique_ptr<Then_Node> then) : variable(std::move(variable)), condition(std::move(condition)), assignment(std::move(assignment)), then(std::move(then)){};
    Value *code_gen();
    void print();
};

class Variable_Assignment_Node : public Node
{
private:
    std::string name;
    unique_ptr<Node> value;

public:
    Variable_Assignment_Node(std::string name, unique_ptr<Node> value) : name(name), value(std::move(value)){};
    Value *code_gen();
    void print();
};

class Object_Node : public Node
{
private:
    std::string name;
    std::map<std::string, Variable_Type> properties;

public:
    Object_Node(std::string name, std::map<std::string, Variable_Type> properties) : name(name), properties(properties){};
    Value *code_gen();
    void print();
};

class Object_Expression : public Node
{
private:
    std::vector<unique_ptr<Variable_Assignment_Node>> properties;

public:
    Object_Expression(std::vector<unique_ptr<Variable_Assignment_Node>> properties) : properties(std::move(properties)){};
    Value *code_gen();
    void print();
};

class String_Expression : public Node
{
private:
    std::string value;

public:
    String_Expression(std::string value) : value(value){};
    Value *code_gen();
    void print();
};

class Return_Node : public Node
{
private:
    unique_ptr<Node> value;

public:
    Return_Node(unique_ptr<Node> value) : value(std::move(value)){};
    void print();
    Value *code_gen();
};

typedef std::vector<unique_ptr<Node>> Nodes;

#endif