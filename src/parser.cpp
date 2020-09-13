#include "parser.h"

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    bin_op_precedence['<'] = 10;
    bin_op_precedence['+'] = 20;
    bin_op_precedence['-'] = 20;
    bin_op_precedence['*'] = 40;

    parse_fn_declaration();
    // for (auto &token : tokens)
    // {
    //     auto node = parse_token(token);
    // }
}

unique_ptr<Node> parse_token(std::shared_ptr<Token> token)
{
    unique_ptr<Node> node = std::make_unique<Node>();

    switch (token->type)
    {
    case Token_Types::tok_fn:
    {
        cout << "Found fn" << endl;
        break;
    }
    default:
        break;
    }

    return node;
}

std::unique_ptr<Expression_Node> parse_number_expression()
{
    auto res = std::make_unique<Number_Expression_Node>(num_val);
    get_next_token();
    return std::move(res);
}

std::unique_ptr<Expression_Node> parse_paren_expression()
{
    get_next_token();
    auto v = parse_expression();
    if (!v)
        return nullptr;
    if (cur_tok != ')')
        return error("expected ')'");
    get_next_token();
    return v;
}

std::unique_ptr<Expression_Node> parse_identifier_expression()
{
    std::string id_name = identifier_string;

    get_next_token();

    if (cur_tok != '(')
        return std::make_unique<Variable_Expression_Node>(id_name);

    get_next_token();
    std::vector<std::unique_ptr<Expression_Node>> args;
    if (cur_tok != ')')
    {
        while (true)
        {
            if (auto arg = parse_expression())
            {
                args.push_back(std::move(arg));
            }
            else
            {
                return nullptr;
            }

            if (cur_tok == ')')
                break;
            if (cur_tok != ',')
                return error("Expected ')' or ',' in argument list");

            get_next_token();
        }
    }

    get_next_token();

    return std::make_unique<Call_Expression_Node>(id_name, std::move(args));
}

std::unique_ptr<Expression_Node> parse_primary()
{
    switch (cur_tok)
    {
    case Token_Types::tok_identifier:
        return parse_identifier_expression();
    case Token_Types::tok_number:
        return parse_number_expression();
    case Token_Types::tok_open_paren:
        return parse_paren_expression();
    default:
        return error("Unknown token when expecting an expression");
    }
}

std::unique_ptr<Expression_Node> parse_bin_op_rhs(int expr_prec, std::unique_ptr<Expression_Node> lhs)
{
    while (true)
    {
        int tok_prec = get_tok_precedence();

        if (tok_prec < expr_prec)
            return lhs;

        int bin_op = cur_tok;
        get_next_token();

        auto rhs = parse_primary();
        if (!rhs)
            return nullptr;

        int next_prec = get_tok_precedence();
        if (tok_prec < next_prec)
        {
            rhs = parse_bin_op_rhs(tok_prec + 1, std::move(rhs));
            if (!rhs)
                return nullptr;
        }

        lhs = std::make_unique<Binary_Expression_Node>(std::to_string(bin_op), std::move(lhs), std::move(rhs));
    }
}

std::unique_ptr<Expression_Node> parse_expression()
{
    auto lhs = parse_primary();
    if (!lhs)
        return nullptr;
    return parse_bin_op_rhs(0, std::move(lhs));
}

std::unique_ptr<Expression_Node> parse_prototype()
{
    if (cur_tok != Token_Types::tok_identifier)
        return error("Expected function name in prototype");

    std::string fn_name = identifier_string;
    get_next_token();

    if (cur_tok != '(')
        return error_p("Expected '(' in prototype");

    std::vector<std::string> arg_names;
    while (get_next_token() == Token_Types::tok_identifier)
        arg_names.push_back(identifier_string);
    if (cur_tok != ')')
        return error_p("Expected ')' in prototype");

    get_next_token();

    return std::make_unique<Prototype_Node>(fn_name, std::move(arg_names));
}

std::unique_ptr<Expression_Node> parse_fn_declaration()
{
    get_next_token();
    // auto proto = parse_prototype();
    // if (!proto)
    //     return nullptr;

    // if (auto E = parse_expression())
    //     return std::make_unique<Function_Node>(std::move(proto), std::move(E));
    // return nullptr;
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
    if (!isascii(cur_tok))
        return -1;

    int tok_prec = bin_op_precedence[cur_tok];
    if (tok_prec <= 0)
        return -1;
    return tok_prec;
}

/**
 * 
 * TODO fix this method for the code to work then fix the syntax to my liking
 * instead of getchar() just point to a new char in the file
 * 
 * 
 */
int get_tok()
{
    int last_char = ' ';

    // Skip any whitespace.
    while (isspace(last_char))
        last_char = getchar();

    if (isalpha(last_char))
    { // identifier: [a-zA-Z][a-zA-Z0-9]*
        identifier_string = last_char;
        while (isalnum((last_char = getchar())))
            identifier_string += last_char;

        if (identifier_string == "def")
            return Token_Types::tok_fn;
        // if (identifier_string == "extern")
        // return tok_extern;
        return tok_identifier;
    }

    if (isdigit(last_char) || last_char == '.')
    { // Number: [0-9.]+
        std::string NumStr;
        do
        {
            NumStr += last_char;
            last_char = getchar();
        } while (isdigit(last_char) || last_char == '.');

        num_val = strtod(NumStr.c_str(), nullptr);
        return tok_number;
    }

    if (last_char == '#')
    {
        // Comment until end of line.
        do
            last_char = getchar();
        while (last_char != EOF && last_char != '\n' && last_char != '\r');

        if (last_char != EOF)
            return get_tok();
    }

    // Check for end of file.  Don't eat the EOF.
    // if (last_char == EOF)
    //     return tok_eof;

    // Otherwise, just return the character as its ascii value.
    int ThisChar = last_char;
    last_char = getchar();
    return ThisChar;
}

int get_next_token() { return cur_tok = get_tok(); }