#ifndef PARSER_H
#define PARSER_H

#include "lexer.h"

#include <string>
#include <memory>
#include <vector>
#include <variant>
#include <map>

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

class Call_Expression_Node : public Expression_Node
{
private:
    std::string callee;
    std::vector<unique_ptr<Expression_Node>> args;

public:
    Call_Expression_Node(const std::string &callee, std::vector<unique_ptr<Expression_Node>> args) : callee(callee), args(std::move(args)){};
};

class Prototype_Node : public Expression_Node
{
private:
    std::string name;
    std::vector<std::string> args;

public:
    Prototype_Node(const std::string &name, std::vector<std::string> args) : name(name), args(args) {}
};

class Function_Node : public Expression_Node
{
    std::unique_ptr<Expression_Node> Proto;
    std::unique_ptr<Expression_Node> Body;

public:
    Function_Node(std::unique_ptr<Expression_Node> Proto,
                  std::unique_ptr<Expression_Node> Body)
        : Proto(std::move(Proto)), Body(std::move(Body)) {}
};

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens);
unique_ptr<Node> parse_token(std::shared_ptr<Token> tokens);

static std::string identifier_string;
static double num_val;
static int cur_tok;
static int get_next_token();
static int get_tok();
static std::unique_ptr<Expression_Node> error(const char *str);
static std::unique_ptr<Prototype_Node> error_p(const char *str);
static std::map<char, int> bin_op_precedence;
static int get_tok_precedence();

static std::unique_ptr<Expression_Node> parse_number_expression();
static std::unique_ptr<Expression_Node> parse_paren_expression();
static std::unique_ptr<Expression_Node> parse_identifier_expression();
static std::unique_ptr<Expression_Node> parse_primary();
static std::unique_ptr<Expression_Node> parse_bin_op_rhs(int expr_prec, std::unique_ptr<Expression_Node> lhs);
static std::unique_ptr<Expression_Node> parse_expression();
static std::unique_ptr<Expression_Node> parse_prototype();
static std::unique_ptr<Expression_Node> parse_fn_declaration();

#endif