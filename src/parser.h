#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"

#include <string>
#include <memory>
#include <vector>
#include <variant>

using std::unique_ptr;

enum Node_Types
{
    NumberExpressionNode,
    VariableExpressionNode,
    BinaryExpressionNode,
    CallExpressionNode
};

class Node
{
public:
    Node_Types type;
};

class Expression_Node
{
public:
    Expression_Node();
};

class Number_Expression_Node : public Expression_Node
{
private:
    double value;

public:
    Number_Expression_Node(double value) : value(value){};
};

class Variable_Expression_Node : public Expression_Node
{
private:
    std::string name;

public:
    Variable_Expression_Node(std::string name) : name(name) {}
};

class Binary_Expression_Node : public Expression_Node
{
private:
    std::string op;
    unique_ptr<Expression_Node> lhs, rhs;

public:
    Binary_Expression_Node(std::string op, unique_ptr<Expression_Node> lhs, unique_ptr<Expression_Node> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
};

class Call_Expression_Node
{
private:
    std::string callee;
    std::vector<unique_ptr<Expression_Node>> args;

public:
    Call_Expression_Node(const std::string &callee, std::vector<unique_ptr<Expression_Node>> args) : callee(callee), args(std::move(args)){};
};

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens);
unique_ptr<Node> parse_token(std::shared_ptr<Token> token);

#endif