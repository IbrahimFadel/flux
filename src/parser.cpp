#include "parser.h"

std::vector<std::unique_ptr<Node>> parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    tok_pointer = 0;
    toks = tokens;
    cur_tok = toks[tok_pointer];

    bin_op_precedence["<"] = 10;
    bin_op_precedence["+"] = 20;
    bin_op_precedence["-"] = 20;
    bin_op_precedence["*"] = 40;

    std::vector<std::unique_ptr<Node>> nodes;
    bool ate_semicolon = false;
    while (cur_tok->type != Token_Types::tok_eof)
    {
        std::unique_ptr<Node> node = std::make_unique<Node>();
        switch (cur_tok->type)
        {
        case Token_Types::tok_fn:
        {
            auto fn = parse_fn_declaration();
            ate_semicolon = false;
            node->type = Node_Types::FunctionDeclarationNode;
            node->function_node = std::move(fn);
            break;
        }
        case Token_Types::tok_import:
        {
            auto import = parse_import();
            ate_semicolon = true;
            node->type = Node_Types::ImportNode;
            node->expression_node = std::move(import);
            break;
        }
        case Token_Types::tok_i64:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i32:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i16:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i8:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        default:
            break;
        }
        nodes.push_back(std::move(node));
        if (!ate_semicolon)
        {
            get_next_token();
        }
    }

    return nodes;
}

std::unique_ptr<Variable_Node> parse_variable_declaration()
{
    const char *type_string = cur_tok->value.c_str();

    Variable_Types type = token_type_to_variable_type(cur_tok->type);

    get_next_token();

    std::string name = cur_tok->value;

    get_next_token();

    get_next_token();

    auto val = parse_expression(true, type);
    if (!val)
    {
        error("Expected expression");
        return nullptr;
    }

    // return std::make_unique<Variable_Node>(name, type, std::move(val));
    std::unique_ptr<Variable_Node> var_node = std::make_unique<Variable_Node>(name, type, std::move(val));
    return var_node;
}

std::unique_ptr<Function_Node> parse_fn_declaration()
{
    get_next_token();
    auto proto = parse_prototype();
    if (!proto)
        return nullptr;

    auto expressions = parse_fn_body();

    return std::make_unique<Function_Node>(std::move(proto), std::move(expressions), proto->get_arg_types());
}

std::unique_ptr<Prototype_Node> parse_prototype()
{
    if (cur_tok->type != Token_Types::tok_identifier)
        return error_p("Expected function name in prototype");

    std::string fn_name = cur_tok->value;

    get_next_token();

    if (cur_tok->type != Token_Types::tok_open_paren)
        return error_p("Expected '(' in prototype");

    get_next_token();
    std::vector<Variable_Types> arg_types;
    std::vector<std::string> arg_names;

    int param_counter = 0;
    while (cur_tok->type != Token_Types::tok_close_paren)
    {
        if (param_counter == 0)
        {
            arg_types.push_back(token_type_to_variable_type(cur_tok->type));
        }
        else if (param_counter == 1)
        {
            arg_names.push_back(cur_tok->value);
        }
        else if (param_counter == 2)
        {
            if (cur_tok->type == Token_Types::tok_comma)
            {
                param_counter = -1;
            }
        }

        get_next_token();
        param_counter++;
    }

    if (cur_tok->type != Token_Types::tok_close_paren)
        return error_p("Expected ')' in prototype");

    get_next_token();

    if (cur_tok->type != Token_Types::tok_arrow)
        return error_p("Expected '->' to indicate return type in prototype");

    get_next_token();

    Variable_Types return_type = token_type_to_variable_type(cur_tok->type);

    get_next_token();

    if (cur_tok->type != Token_Types::tok_open_curly_bracket)
    {
        char *err_msg;
        sprintf(err_msg, "Expected '{' on line %d position %d", cur_tok->row, cur_tok->col);
        return error_p(err_msg);
    }

    get_next_token();

    return std::make_unique<Prototype_Node>(fn_name, arg_types, arg_names, return_type);
}

std::vector<std::unique_ptr<Node>> parse_fn_body()
{
    std::vector<std::unique_ptr<Node>> nodes;
    bool ate_semicolon = false;
    while (cur_tok->type != Token_Types::tok_close_curly_bracket)
    {
        std::unique_ptr<Node> node = std::make_unique<Node>();
        switch (cur_tok->type)
        {
        case Token_Types::tok_if:
        {
            auto if_node = parse_if();
            ate_semicolon = true;
            node->type = Node_Types::IfNode;
            node->expression_node = std::move(if_node);
            break;
        }
        case Token_Types::tok_i64:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i32:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i16:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_i8:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_float:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_double:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_bool:
        {
            auto var = parse_variable_declaration();
            ate_semicolon = true;
            node->type = Node_Types::VariableDeclarationNode;
            node->variable_node = std::move(var);
            break;
        }
        case Token_Types::tok_return:
        {
            auto ret = parse_return_statement();
            ate_semicolon = true;
            node->type = Node_Types::ReturnNode;
            node->return_node = std::move(ret);
            break;
        }
        case Token_Types::tok_toi64:
        {
            auto ret = parse_typecast_expression();
            ate_semicolon = true;
            node->type = Node_Types::TypeCastNode;
            node->expression_node = std::move(ret);
            break;
        }
        case Token_Types::tok_toi32:
        {
            auto ret = parse_typecast_expression();
            ate_semicolon = true;
            node->type = Node_Types::TypeCastNode;
            node->expression_node = std::move(ret);
            break;
        }
        case Token_Types::tok_toi16:
        {
            auto ret = parse_typecast_expression();
            ate_semicolon = true;
            node->type = Node_Types::TypeCastNode;
            node->expression_node = std::move(ret);
            break;
        }
        case Token_Types::tok_toi8:
        {
            auto ret = parse_typecast_expression();
            ate_semicolon = true;
            node->type = Node_Types::TypeCastNode;
            node->expression_node = std::move(ret);
            break;
        }
        case Token_Types::tok_identifier:
        {
            auto id = parse_identifier_expression();
            switch (id->type)
            {
            case Node_Types::CallExpressionNode:
                node->type = Node_Types::CallExpressionNode;
                ate_semicolon = false;
                break;
            case Node_Types::AssignmentNode:
                node->type = Node_Types::AssignmentNode;
                ate_semicolon = true;
                break;
            default:
                node->type = Node_Types::UnknownNode;
                break;
            }
            node->expression_node = std::move(id);
            break;
        }
        default:
            break;
        }
        nodes.push_back(std::move(node));
        if (!ate_semicolon)
        {
            get_next_token();
        }
    }

    return nodes;
}

std::unique_ptr<Expression_Node> parse_expression(bool needs_semicolon, Variable_Types type)
{
    auto lhs = parse_primary(type);
    if (!lhs)
    {
        error("Error parsing primary");
        return nullptr;
    };
    auto bin_op_rhs = parse_bin_op_rhs(0, std::move(lhs), type);
    if (needs_semicolon)
        get_next_token();
    return bin_op_rhs;
}

std::unique_ptr<Expression_Node> parse_primary(Variable_Types type)
{
    switch (cur_tok->type)
    {
    case Token_Types::tok_identifier:
        return parse_identifier_expression();
    case Token_Types::tok_number:
        return parse_number_expression(type);
    case Token_Types::tok_string:
        return parse_string_expression();
    case Token_Types::tok_toi64:
        return parse_typecast_expression();
    case Token_Types::tok_toi32:
        return parse_typecast_expression();
    case Token_Types::tok_toi16:
        return parse_typecast_expression();
    case Token_Types::tok_toi8:
        return parse_typecast_expression();
    default:
        break;
    }
}

std::unique_ptr<Expression_Node> parse_string_expression()
{
    auto with_quotes = cur_tok->value;
    auto value = with_quotes.substr(1, with_quotes.size() - 2);
    get_next_token();
    return std::make_unique<String_Expression>(value);
}

std::unique_ptr<Expression_Node> parse_identifier_expression()
{
    std::string id_name = cur_tok->value;

    get_next_token();

    if (cur_tok->type == Token_Types::tok_eq)
    {
        auto prev = toks[tok_pointer - 2]->type;

        if (prev != Token_Types::tok_i64 || prev != Token_Types::tok_i32 || prev != Token_Types::tok_i16 || prev != Token_Types::tok_i8 || prev != Token_Types::tok_float || prev != Token_Types::tok_double)
        {
            get_next_token();
            auto expr = parse_expression(true);
            auto assignment_node = std::make_unique<Assignment_Node>(id_name, std::move(expr));
            assignment_node->type = Node_Types::AssignmentNode;
            return assignment_node;
        }
    }

    if (cur_tok->type != Token_Types::tok_open_paren)
        return std::make_unique<Variable_Expression_Node>(id_name);

    get_next_token();

    std::vector<std::unique_ptr<Expression_Node>> args;
    if (cur_tok->type != Token_Types::tok_close_paren)
    {
        while (true)
        {
            if (auto arg = parse_expression(false))
            {
                args.push_back(std::move(arg));
            }
            else
            {
                error("Error parsing function call parameters");
                return nullptr;
            }

            if (cur_tok->type == Token_Types::tok_close_paren)
                break;
            if (cur_tok->type != Token_Types::tok_comma)
                return error("Expected ')' or ',' in argument list");

            get_next_token();
        }
    }

    get_next_token();
    auto call_node = std::make_unique<Call_Expression_Node>(id_name, std::move(args));
    call_node->type = Node_Types::CallExpressionNode;
    return call_node;
}

std::unique_ptr<Expression_Node> parse_number_expression(Variable_Types type)
{
    auto number_expression = std::make_unique<Number_Expression_Node>(std::stod(cur_tok->value), type);
    get_next_token();
    return std::move(number_expression);
}

std::unique_ptr<Expression_Node> parse_bin_op_rhs(int expr_prec, std::unique_ptr<Expression_Node> lhs, Variable_Types type)
{
    while (true)
    {
        int tok_prec = get_tok_precedence();
        if (tok_prec < expr_prec)
        {
            return lhs;
        }

        std::string bin_op = cur_tok->value;

        get_next_token();

        auto rhs = parse_primary(type);
        if (!rhs)
        {
            error("Error parsing right hand side");
            return nullptr;
        }

        int next_prec = get_tok_precedence();
        if (tok_prec < next_prec)
        {
            rhs = parse_bin_op_rhs(tok_prec + 1, std::move(rhs));
            if (!rhs)
                return nullptr;
        }

        lhs = std::make_unique<Binary_Expression_Node>(bin_op, std::move(lhs), std::move(rhs));
    }
}

std::unique_ptr<Return_Node> parse_return_statement()
{
    get_next_token();
    auto expr = parse_expression(true, Variable_Types::type_i32);

    if (expr == 0)
    {
        error("Error parsing return expression");
        return nullptr;
    }

    return std::make_unique<Return_Node>(std::move(expr));
}

std::unique_ptr<Expression_Node> parse_typecast_expression()
{
    auto type = token_type_to_variable_type(cur_tok->type);
    get_next_token();
    get_next_token();

    auto expr = parse_expression(false);

    get_next_token();

    auto node = std::make_unique<Type_Cast_Node>(std::move(expr), type);

    return node;
}

std::unique_ptr<Expression_Node> parse_if()
{
    get_next_token(); //? eat 'if'
    get_next_token(); //? eat '('

    std::vector<std::unique_ptr<Condition_Expression>> conditions;
    std::vector<Token_Types> condition_seperators;

    while (cur_tok->type != Token_Types::tok_close_paren)
    {
        auto lhs = parse_expression(false);

        auto op = cur_tok->type;

        get_next_token(); //? eat operator

        auto rhs = parse_expression(false);

        auto condition = std::make_unique<Condition_Expression>(std::move(lhs), op, std::move(rhs));
        conditions.push_back(std::move(condition));

        if (cur_tok->type == Token_Types::tok_and || cur_tok->type == Token_Types::tok_or)
        {
            condition_seperators.push_back(cur_tok->type);
            get_next_token();
        }
    }

    get_next_token(); //? eat ')'
    get_next_token(); //? eat '{'

    auto then = parse_fn_body();

    get_next_token(); //? eat '}'

    auto if_node = std::make_unique<If_Node>(std::move(conditions), condition_seperators, std::move(then));
    return if_node;
}

std::unique_ptr<Expression_Node> parse_import()
{
    get_next_token(); //? eat 'import'
    std::string path_with_quotes = cur_tok->value;
    std::string path = path_with_quotes.substr(1, path_with_quotes.size() - 2);
    get_next_token(); //? eat string
    get_next_token(); //? eat ';'
    return std::make_unique<Import_Node>(path);
}

Variable_Types token_type_to_variable_type(Token_Types type)
{
    switch (type)
    {
    case Token_Types::tok_i64:
        return Variable_Types::type_i64;
    case Token_Types::tok_i32:
        return Variable_Types::type_i32;
    case Token_Types::tok_i16:
        return Variable_Types::type_i16;
    case Token_Types::tok_i8:
        return Variable_Types::type_i8;
    case Token_Types::tok_float:
        return Variable_Types::type_float;
    case Token_Types::tok_double:
        return Variable_Types::type_double;
    case Token_Types::tok_bool:
        return Variable_Types::type_bool;
    case Token_Types::tok_toi64:
        return Variable_Types::type_i64;
    case Token_Types::tok_toi32:
        return Variable_Types::type_i32;
    case Token_Types::tok_toi16:
        return Variable_Types::type_i16;
    case Token_Types::tok_toi8:
        return Variable_Types::type_i8;
    default:
        break;
    }
}

void get_next_token()
{
    tok_pointer++;
    cur_tok = toks[tok_pointer];
}

std::unique_ptr<Expression_Node> error(const char *str)
{
    fprintf(stderr, "LogError: %s\n", str);
    return nullptr;
}

std::unique_ptr<Prototype_Node> error_p(const char *str)
{
    error(str);
    return nullptr;
}

int get_tok_precedence()
{
    int tok_prec = bin_op_precedence[cur_tok->value];
    if (tok_prec <= 0)
        return -1;
    return tok_prec;
}

void Function_Node::set_variables(std::string name, llvm::Value *var)
{
    variables[name] = var;
}

llvm::Value *Function_Node::get_variable(std::string name)
{
    return variables[name];
}

std::vector<Variable_Types> Prototype_Node::get_arg_types()
{
    return arg_types;
}

std::unique_ptr<Prototype_Node> Function_Node::get_proto()
{
    return std::move(proto);
}

std::vector<Variable_Types> Function_Node::get_arg_types()
{
    return arg_types;
}

std::string Prototype_Node::get_name() { return name; }

std::unique_ptr<Expression_Node> Condition_Expression::get_lhs() { return std::move(lhs); };
std::unique_ptr<Expression_Node> Condition_Expression::get_rhs() { return std::move(rhs); };
Token_Types Condition_Expression::get_op() { return op; }
Variable_Types Prototype_Node::get_return_type() { return return_type; };
llvm::Value *Function_Node::get_return_value_ptr() { return return_value_ptr; };
llvm::BasicBlock *Function_Node::get_end_bb() { return end_bb; };