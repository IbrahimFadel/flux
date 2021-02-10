#ifndef AST_H
#define AST_H

#include <string.h>
#include <memory>
#include <vector>
#include <map>
#include <llvm/IR/Value.h>

#include "lexer.h"

using std::unique_ptr;

class Function_Declaration;
class Code_Block;
class Variable_Declaration;
class Expression;

#include <llvm/IR/Value.h>

class Node
{
private:
public:
    virtual llvm::Value *code_gen() = 0;
};

class Expression : public Node
{
private:
public:
    virtual llvm::Value *code_gen() = 0;
};

class Number_Expression : public Expression
{
private:
    double value;

public:
    Number_Expression(double value) : value(value){};
    llvm::Value *code_gen();
};

class Variable_Reference_Expression : public Expression
{
private:
    std::string name;

public:
    Variable_Reference_Expression(std::string name) : name(name){};
    llvm::Value *code_gen();
};

class Binary_Operation_Expression : public Expression
{
private:
    Token_Type op;
    unique_ptr<Expression> lhs;
    unique_ptr<Expression> rhs;

public:
    Binary_Operation_Expression(Token_Type op, unique_ptr<Expression> lhs, unique_ptr<Expression> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
    llvm::Value *code_gen();
};

class Unary_Prefix_Operation_Expression : public Expression
{
private:
    Token_Type op;
    unique_ptr<Expression> value;

public:
    Unary_Prefix_Operation_Expression(Token_Type op, unique_ptr<Expression> value) : op(op), value(std::move(value)){};
    llvm::Value *code_gen();
};

class Function_Declaration : public Node
{
private:
    std::string name;
    std::map<std::string, std::string> params;
    std::string return_type;
    std::unique_ptr<Code_Block> then;

    std::map<std::string, llvm::Value *> variables;

public:
    Function_Declaration(std::string name, std::map<std::string, std::string> params, std::string return_type, std::unique_ptr<Code_Block> then) : name(name), params(params), return_type(return_type), then(std::move(then)){};
    llvm::Value *code_gen();

    void set_variable(std::string name, llvm::Value *v);
    llvm::Value *get_variable(std::string name);
};

class Code_Block : public Node
{
private:
    std::vector<unique_ptr<Node>> nodes;

public:
    Code_Block(std::vector<unique_ptr<Node>> nodes) : nodes(std::move(nodes)){};
    llvm::Value *code_gen();
};

class Variable_Declaration : public Node
{
private:
    std::string name;
    std::string type;
    unique_ptr<Expression> value;

public:
    Variable_Declaration(std::string name, std::string type, unique_ptr<Expression> value) : name(name), type(type), value(std::move(value)){};
    llvm::Value *code_gen();
};

class Object_Type_Expression : public Expression
{
private:
    std::string name;
    std::map<std::string, std::string> properties;

public:
    Object_Type_Expression(std::string name, std::map<std::string, std::string> properties) : name(name), properties(properties){};
    llvm::Value *code_gen();
};

class If_Statement : public Expression
{
private:
    std::vector<unique_ptr<Expression>> conditions;
    std::vector<Token_Type> condition_separators;
    unique_ptr<Code_Block> then;

public:
    If_Statement(std::vector<unique_ptr<Expression>> conditions, std::vector<Token_Type> condition_separators, unique_ptr<Code_Block> then) : conditions(std::move(conditions)), condition_separators(condition_separators), then(std::move(then)){};
    llvm::Value *code_gen();
};

class Return_Statement : public Expression
{
private:
    unique_ptr<Expression> value;

public:
    Return_Statement(unique_ptr<Expression> value) : value(std::move(value)){};
    llvm::Value *code_gen();
};

class Function_Call_Expression : public Expression
{
private:
    std::string name;
    std::vector<unique_ptr<Expression>> params;

public:
    Function_Call_Expression(std::string name, std::vector<unique_ptr<Expression>> params) : name(name), params(std::move(params)){};
    llvm::Value *code_gen();
};

class Import_Statement : public Expression
{
private:
    std::string path;

public:
    Import_Statement(std::string path) : path(path){};
    llvm::Value *code_gen();
    std::string get_path();
};

typedef std::vector<unique_ptr<Node>> Nodes;

#endif