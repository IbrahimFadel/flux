#ifndef AST_H
#define AST_H

#include <string.h>
#include <memory>
#include <vector>
#include <variant>
#include <map>

#include "lexer.h"

using std::unique_ptr;

// #include "common.h"
// #include "lexer.h"

#include <llvm/IR/Value.h>

class Node
{
};

class Expression : public Node
{
private:
public:
    // virtual llvm::Value *codegen() = 0;
};

class Number_Expression : public Expression
{
private:
    double value;

public:
    Number_Expression(double value) : value(value){};
    // llvm::Value *codegen();
};

class Variable_Reference_Expression : public Expression
{
private:
    std::string name;

public:
    Variable_Reference_Expression(std::string name) : name(name){};
};

class Binary_Operation_Expression : public Expression
{
private:
    std::string op;
    unique_ptr<Expression> lhs;
    unique_ptr<Expression> rhs;

public:
    Binary_Operation_Expression(std::string op, unique_ptr<Expression> lhs, unique_ptr<Expression> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
    // llvm::Value *codegen();
};

class Unary_Prefix_Operation_Expression : public Expression
{
private:
    Token_Type op;
    unique_ptr<Expression> value;

public:
    Unary_Prefix_Operation_Expression(Token_Type op, unique_ptr<Expression> value) : op(op), value(std::move(value)){};
    // llvm::Value *codegen();
};

class Function_Declaration : public Node
{
private:
    std::string name;
    std::map<std::string, std::string> params;

public:
    Function_Declaration(std::string name, std::map<std::string, std::string> params) : name(name), params(params){};
};

class Code_Block : public Node
{
private:
    std::vector<unique_ptr<Node>> nodes;

public:
    Code_Block(std::vector<unique_ptr<Node>> nodes) : nodes(std::move(nodes)){};
};

class Variable_Declaration : public Node
{
private:
    std::string name;
    std::string type;
    unique_ptr<Expression> value;

public:
    Variable_Declaration(std::string name, std::string type, unique_ptr<Expression> value) : name(name), type(type), value(std::move(value)){};
};

class Object_Type_Expression : public Expression
{
private:
    std::string name;
    std::map<std::string, std::string> properties;

public:
    Object_Type_Expression(std::string name, std::map<std::string, std::string> properties) : name(name), properties(properties){};
};

#endif