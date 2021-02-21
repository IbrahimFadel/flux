#include "parser.h"

using namespace ssc;

Nodes Parser::parseTokens(Tokens toks)
{
    tokens = std::move(toks);
    curTokIndex = 0;
    curTok = std::move(tokens[curTokIndex]);

    std::vector<unique_ptr<Node>> nodes;

    while (curTok->type != TokenType::tokEOF)
    {
        parseToken(curTok);
    }

    return nodes;
}

void Parser::error(std::string msg)
{
    std::cerr << "\033[1;31m"
              << "Parser Error: "
              << "\033[0m" << msg << std::endl;
    exit(1);
}

void Parser::errIfCurTokNotType(TokenType type, std::string msg)
{
    std::string msgWPos = msg + " at line " + std::to_string(curTok->row) + " pos " + std::to_string(curTok->col);
    if (curTok->type != type)
        error(msgWPos);
}

void Parser::getNextToken()
{
    curTokIndex++;
    curTok = std::move(tokens[curTokIndex]);
}

unique_ptr<Node> Parser::parseToken(const unique_ptr<Token> &tok)
{
    switch (tok->type)
    {
    case TokenType::tokPub:
        return parsePub();
    case TokenType::tokMut:
        return parseMut();
    case TokenType::tokI64:
        return parseVariableDeclaration();
    case TokenType::tokU64:
        return parseVariableDeclaration();
    case TokenType::tokI32:
        return parseVariableDeclaration();
    case TokenType::tokU32:
        return parseVariableDeclaration();
    case TokenType::tokI16:
        return parseVariableDeclaration();
    case TokenType::tokU16:
        return parseVariableDeclaration();
    case TokenType::tokI8:
        return parseVariableDeclaration();
    case TokenType::tokU8:
        return parseVariableDeclaration();
    case TokenType::tokF64:
        return parseVariableDeclaration();
    case TokenType::tokF32:
        return parseVariableDeclaration();
    case TokenType::tokBool:
        return parseVariableDeclaration();
    default:
        error("Could not parse token: " + curTok->value);
        break;
    }

    return nullptr;
}

unique_ptr<Node> Parser::parseMut(bool isPub)
{
    getNextToken(); //? eat 'mut'
    return parseVariableDeclaration(isPub, true);
}

unique_ptr<Expression> Parser::parseExpression(bool needsSemicolon)
{
    auto lhs = parsePrimary();
    if (!lhs)
        error("Error parsing primary");

    auto binop_node = parseBinopRHS(0, std::move(lhs));

    // if (cur_tok->type == Token_Type::tok_open_square_bracket)
    // {
    //     auto expr = parse_square_bracket_expression(std::move(binop_node));
    //     binop_node = parse_binop_rhs(0, std::move(expr));
    // }

    if (needsSemicolon)
    {
        errIfCurTokNotType(TokenType::tokSemicolon, "Expected ';' at end of expression");
        getNextToken(); //? eat ';'
    }
    return binop_node;
}

unique_ptr<Expression> Parser::parsePrimary()
{
    switch (curTok->type)
    {
    case TokenType::tokNumberLit:
        return parseNumberExpression();
        // case TokenType::tokStringLit:
        // return parse_string_literal_expression();
        // case TokenType::tokIdentifier:
        // return parse_identifier_expression();
        // case TokenType::tokOpenParen:
        // return parse_paren_expression();
        // case TokenType::tokAsterisk:
        // return parse_unary_prefix_operation_expression();
        // case TokenType::tokAmpersand:
        // return parse_unary_prefix_operation_expression();
        // case TokenType::tokOpenCurlyBracket:
        // return parse_struct_value_expression();
        // case TokenType::tokNullptr:
        // get_next_token(); //? eat 'nullptr'
        // return std::make_unique<Nullptr_Expression>();
    }
    return nullptr;
}

unique_ptr<Expression> Parser::parseNumberExpression()
{
    double value = std::stod(curTok->value);
    getNextToken(); //? eat number
    return std::make_unique<NumberExpression>(value);
}

unique_ptr<Expression> Parser::parseBinopRHS(int expressionPrecedence, unique_ptr<Expression> lhs)
{
    while (true)
    {
        int tok_precedence = getTokenPrecedence();
        if (tok_precedence < expressionPrecedence)
            return lhs;

        TokenType binop = curTok->type;

        getNextToken(); //? eat operator

        auto rhs = parsePrimary();
        if (!rhs)
            error("Error parsing binary operator right hand side");

        int next_precedence = getTokenPrecedence();
        if (tok_precedence < next_precedence)
        {
            rhs = parseBinopRHS(tok_precedence + 1, std::move(rhs));
            if (!rhs)
                error("Error parsing binary operator right hand side");
        }

        lhs = std::make_unique<BinaryOperationExpression>(std::move(lhs), std::move(rhs), binop);
    }

    return nullptr;
}

unique_ptr<Node> Parser::parsePub()
{
    getNextToken(); //? eat 'pub'

    if (curTok->type == TokenType::tokFn)
    {
        return parseFn(true);
    }
    else if (curTok->type == TokenType::tokMut)
    {
        return parseMut(true);
    }
    else
    {
        return parseVariableDeclaration(true);
    }

    // error("Unexpected token '" + curTok->value + "' following 'pub'");
    // return nullptr;
}

unique_ptr<VariableDeclaration> Parser::parseVariableDeclaration(bool isPub, bool isMut)
{
    if (isPub && !parsingInGlobalScope)
    {
        std::string msg = "Cannot declare public variable outside global scope. Line " + std::to_string(curTok->row) + ", col " + std::to_string(curTok->col);
        error(msg);
    }

    std::string varType = parseType();

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier following 'pub'");
    std::string varName = curTok->value;
    getNextToken(); //? eat name

    if (curTok->type == TokenType::tokSemicolon)
    {
        getNextToken(); //? eat ';'
        return std::make_unique<VariableDeclaration>(isPub, isMut, varType, varName, nullptr);
    }

    errIfCurTokNotType(TokenType::tokEq, "Expected '=' following variable name");
    getNextToken(); //? eat '='

    auto value = parseExpression();
    return std::make_unique<VariableDeclaration>(isPub, isMut, varType, varName, std::move(value));
}

unique_ptr<FunctionDeclaration> Parser::parseFn(bool isPub)
{
    getNextToken(); //? eat 'fn'

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier following 'fn'");
    std::string functionName = curTok->value;
    getNextToken(); //? eat fn name

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' following function name");
    getNextToken(); //? eat '('

    std::vector<Parameter> parameters;
    while (curTok->type != TokenType::tokCloseParen)
    {
        auto param = parseParameter();
        parameters.push_back(param);

        if (curTok->type == TokenType::tokComma)
        {
            getNextToken();
        }
        else if (curTok->type != TokenType::tokCloseParen)
        {
            error("Expected ')' at end of function parameter list");
        }
    }

    getNextToken(); //? eat ')'

    errIfCurTokNotType(TokenType::tokArrow, "Expected '->' following function parameter list");
    getNextToken(); //? eat '->'

    std::string type = parseType();

    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' following function return type");
    getNextToken();

    parsingInGlobalScope = false;
    std::vector<unique_ptr<Node>> then;
    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        auto node = parseToken(curTok);
        then.push_back(std::move(node));
    }
    getNextToken(); //? eat '}'
    parsingInGlobalScope = true;

    return std::make_unique<FunctionDeclaration>(isPub, functionName, parameters, type, std::move(then));
}

Parameter Parser::parseParameter()
{
    bool mut = false;
    if (curTok->type == TokenType::tokMut)
    {
        mut = true;
        getNextToken(); //? eat 'mut'
    }

    std::string type = parseType();

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier following parameter type");
    std::string name = curTok->value;
    getNextToken();

    Parameter param;
    param.mut = mut;
    param.type = type;
    param.name = name;
    return param;
}

std::string Parser::parseType()
{
    std::string type = curTok->value;
    getNextToken();
    while (curTok->type == TokenType::tokAsterisk || curTok->type == TokenType::tokAmpersand)
    {
        type += curTok->value;
        getNextToken();
    }

    return type;
}

int Parser::getTokenPrecedence()
{
    int tokPrecedence = binopPrecedence[curTok->value];
    if (tokPrecedence <= 0)
        return -1;
    return tokPrecedence;
}

// std::vector<unique_ptr<Node>> Parser::parse_tokens(ssc::Tokens tokens)
// {
//     toks = std::move(tokens);
//     curTokIndex = 0;
//     curTok = std::move(toks[curTokIndex]);

//     binopPrecedence["="] = 2;
//     binopPrecedence["<"] = 10;
//     binopPrecedence[">"] = 10;
//     binopPrecedence["=="] = 10;
//     binopPrecedence["!="] = 10;
//     binopPrecedence["+"] = 20;
//     binopPrecedence["-"] = 20;
//     binopPrecedence["*"] = 40;
//     binopPrecedence["."] = 50;
//     binopPrecedence["->"] = 50;

//     std::vector<unique_ptr<Node>> nodes;

//     while (curTok->type != ssc::TokenType::tok_eof)
//     {
//         auto node = parse_token(curTok);
//         nodes.push_back(std::move(node));
//     }

//     return nodes;
// }

// unique_ptr<Node> Parser::parse_token(const std::shared_ptr<Token> &token)
// {
//     switch (token->type)
//     {
//     case TokenType::tok_fn:
//         return parseFunctionDeclaration();
//     case TokenType::tok_i64:
//         return parseVariableDeclaration();
//     case TokenType::tok_i32:
//         return parseVariableDeclaration();
//     case TokenType::tok_i16:
//         return parseVariableDeclaration();
//     case TokenType::tok_i8:
//         return parseVariableDeclaration();
//     case TokenType::tok_bool:
//         return parseVariableDeclaration();
//     case TokenType::tok_float:
//         return parseVariableDeclaration();
//     case TokenType::tok_double:
//         return parseVariableDeclaration();
//     case TokenType::tok_string:
//         return parseVariableDeclaration();
//     case TokenType::tok_struct:
//         return parseStructTypeDeclaration();
//     case TokenType::tok_if:
//         return parseIfStatement();
//     case TokenType::tok_return:
//         return parseReturnStatement();
//     case TokenType::tok_import:
//         return parseImportStatement();
//     case TokenType::tok_identifier:
//     {
//         if (std::find(structTypes.begin(), structTypes.end(), curTok->value) != structTypes.end())
//             return parseVariableDeclaration(true);
//         else if (token->value == "string")
//             return parseVariableDeclaration();
//         return parseExpression();
//     }
//     default:
//         break;
//     }
//     return nullptr;
// }

// unique_ptr<ImportStatement> Parser::parseImportStatement()
// {
//     getNextToken(); //? eat 'import'
//     throwIfCurTokNotType(TokenType::tok_string_lit, "Expected path to module in import statement", curTok->row, curTok->col);
//     std::string path = curTok->value;
//     getNextToken(); //? eat path
//     throwIfCurTokNotType(TokenType::tok_semicolon, "Expected ';' at end of import statement", curTok->row, curTok->col);
//     getNextToken(); //? eat ';
//     return std::make_unique<ImportStatement>(path);
// }

// unique_ptr<ReturnStatement> Parser::parseReturnStatement()
// {
//     getNextToken(); //? eat 'return'
//     auto value = parseExpression();
//     return std::make_unique<ReturnStatement>(std::move(value));
// }

// unique_ptr<IfStatement> Parser::parseIfStatement()
// {
//     getNextToken(); //? eat 'if'

//     throwIfCurTokNotType(TokenType::tok_open_paren, "Expected '(' in if statement", curTok->row, curTok->col);
//     getNextToken(); //? eat '(

//     std::vector<unique_ptr<Expression>> conditions;
//     std::vector<TokenType> condition_separators;
//     while (curTok->type != TokenType::tok_close_paren)
//     {
//         auto expr = parseExpression(false);
//         conditions.push_back(std::move(expr));

//         if (curTok->type != TokenType::tok_close_paren)
//         {
//             condition_separators.push_back(curTok->type);
//             getNextToken(); //? eat condition separator
//         }
//     }

//     getNextToken(); //? eat ')'

//     throwIfCurTokNotType(TokenType::tok_open_curly_bracket, "Expected '{' in if statement", curTok->row, curTok->col);
//     getNextToken(); //? eat '{'

//     auto then = parseCodeBlock();

//     throwIfCurTokNotType(TokenType::tok_close_curly_bracket, "Expected '}' in if statement", curTok->row, curTok->col);
//     getNextToken(); //? eat '}'

//     return std::make_unique<IfStatement>(std::move(conditions), condition_separators, std::move(then));
// }

// unique_ptr<Expression> Parser::parseStructTypeDeclaration()
// {
//     getNextToken(); //? eat 'struct'

//     throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier following object token", curTok->row, curTok->col);
//     std::string name = curTok->value;
//     getNextToken(); //? eat name

//     throwIfCurTokNotType(TokenType::tok_open_curly_bracket, "Expected '{' following object token", curTok->row, curTok->col);
//     getNextToken(); //? eat '{'

//     std::map<std::string, std::string> properties;
//     std::vector<std::string> property_insetion_order;
//     while (curTok->type != TokenType::tok_close_curly_bracket)
//     {
//         std::string type = getType();

//         throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier in object property", curTok->row, curTok->col);
//         std::string prop_name = curTok->value;
//         getNextToken(); //? eat name

//         throwIfCurTokNotType(TokenType::tok_semicolon, "Expected ';' in object property", curTok->row, curTok->col);
//         getNextToken(); //? eat ';'
//         properties[prop_name] = type;
//         property_insetion_order.push_back(prop_name);
//     }

//     getNextToken(); //? eat '}'
//     throwIfCurTokNotType(TokenType::tok_semicolon, "Expected ';' in object type definition", curTok->row, curTok->col);
//     getNextToken(); //? eat ';'

//     structTypes.push_back(name);

//     return std::make_unique<StructTypeExpression>(name, properties, property_insetion_order);
// }

// unique_ptr<VariableDeclaration> Parser::parseVariableDeclaration(bool is_struct)
// {
//     if (is_struct)
//     {
//         return parseStructVarDeclaration();
//     }
//     std::string type = getType();

//     throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier following variable type", curTok->row, curTok->col);
//     std::string name = curTok->value;
//     getNextToken(); //? eat name

//     if (curTok->type == TokenType::tok_semicolon)
//     {
//         getNextToken(); //? eat ';'
//         return std::make_unique<VariableDeclaration>(name, type, nullptr);
//     }

//     throwIfCurTokNotType(TokenType::tok_eq, "Expected '=' following variable name", curTok->row, curTok->col);
//     getNextToken(); //? eat '='

//     auto value = parseExpression();

//     return std::make_unique<VariableDeclaration>(name, type, std::move(value));
// }

// unique_ptr<VariableDeclaration> Parser::parseStructVarDeclaration()
// {
//     std::string type_name = getType();

//     throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier in struct variable declaration", curTok->row, curTok->col);
//     std::string name = curTok->value;
//     getNextToken(); //? eat name

//     if (curTok->type == TokenType::tok_semicolon)
//     {
//         getNextToken(); //? eat ';'
//         return std::make_unique<VariableDeclaration>(name, type_name, nullptr, true);
//     }

//     throwIfCurTokNotType(TokenType::tok_eq, "Expected '=' in struct variable declaration", curTok->row, curTok->col);
//     getNextToken(); //? eat '='

//     auto value = parseExpression();

//     return std::make_unique<VariableDeclaration>(name, type_name, std::move(value), true);
// }

// unique_ptr<FunctionDeclaration> Parser::parseFunctionDeclaration()
// {
//     getNextToken(); //? eat 'fn'

//     throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier following 'fn'", curTok->row, curTok->col);
//     std::string name = curTok->value;
//     getNextToken(); //? eat name

//     throwIfCurTokNotType(TokenType::tok_open_paren, "Expected '(' in function declaration", curTok->row, curTok->col);
//     getNextToken(); //? eat '('

//     int i = 0;
//     std::map<std::string, std::string> params; //? Param Name, Param Type
//     std::string current_param_type = "";       //? Since the user should be able to have n number of pointers, for example, I can't just have an enum of variable types like before. Instead, I'll store types as strings in parser stage then codegen properly. Atleast -- that's the plan for now
//     std::string current_param_name = "";
//     //? While there are parameters to parse
//     while (curTok->type != TokenType::tok_close_paren)
//     {
//         if (i == 0)
//         {
//             current_param_type = getType();
//         }
//         else if (i == 1)
//         {
//             current_param_name = curTok->value;
//             getNextToken(); //? eat param name
//             params[current_param_name] = current_param_type;
//             current_param_type.clear();
//             current_param_name.clear();
//         }
//         else if (i == 2)
//         {
//             throwIfCurTokNotType(TokenType::tok_comma, "Expected ',' in parameter list", curTok->row, curTok->col);
//             getNextToken();
//             i = -1; //? This looks pretty terrible, but since i do a ++ at the end of every iteration and it needs to be 0 at the start... it's -1 here
//         }
//         i++;
//     }

//     getNextToken(); //? eat ')'

//     throwIfCurTokNotType(TokenType::tok_arrow, "Expected '->' after parameter list", curTok->row, curTok->col);
//     getNextToken(); //? eat '->'

//     std::string return_type = getType();

//     throwIfCurTokNotType(TokenType::tok_open_curly_bracket, "Expected '{' after function return type", curTok->row, curTok->col);
//     getNextToken(); //? eat '{'

//     auto then = parseCodeBlock();

//     throwIfCurTokNotType(TokenType::tok_close_curly_bracket, "Expected '}' after function code block", curTok->row, curTok->col);
//     getNextToken(); //? eat '}'

//     return std::make_unique<FunctionDeclaration>(name, params, return_type, std::move(then));
// }

// unique_ptr<CodeBlock> Parser::parseCodeBlock()
// {
//     std::vector<std::unique_ptr<Node>> nodes;

//     while (curTok->type != TokenType::tok_close_curly_bracket)
//     {
//         auto node = parse_token(std::move(curTok));
//         nodes.push_back(std::move(node));
//     }

//     return std::make_unique<CodeBlock>(std::move(nodes));
// }

// unique_ptr<Expression> Parser::parseExpression(bool needs_semicolon)
// {
//     auto lhs = parsePrimary();
//     if (!lhs)
//         error("Error parsing primary", curTok->row, curTok->col);

//     auto binop_node = parseBinopRhs(0, std::move(lhs));

//     if (curTok->type == TokenType::tok_open_square_bracket)
//     {
//         auto expr = parseSquareBracketExpression(std::move(binop_node));
//         binop_node = parseBinopRhs(0, std::move(expr));
//     }

//     if (needs_semicolon)
//     {
//         throwIfCurTokNotType(TokenType::tok_semicolon, "Expected ';' at end of expression", curTok->row, curTok->col);
//         getNextToken(); //? eat ';'
//     }
//     return binop_node;
// }

// unique_ptr<Expression> Parser::parsePrimary()
// {
//     switch (curTok->type)
//     {
//     case TokenType::tok_number:
//         return parseNumberExpression();
//     case TokenType::tok_string_lit:
//         return parseStringLiteralExpression();
//     case TokenType::tok_identifier:
//         return parseIdentifierExpression();
//     case TokenType::tok_open_paren:
//         return parseParenExpression();
//     case TokenType::tok_asterisk:
//         return parseUnaryPrefixOperationExpression();
//     case TokenType::tok_ampersand:
//         return parseUnaryPrefixOperationExpression();
//     case TokenType::tok_open_curly_bracket:
//         return parseStructValueExpression();
//     case TokenType::tok_nullptr:
//         getNextToken(); //? eat 'nullptr'
//         return std::make_unique<NullptrExpression>();
//     }
//     return nullptr;
// }

// unique_ptr<Expression> Parser::parseStructValueExpression()
// {
//     throwIfCurTokNotType(TokenType::tok_open_curly_bracket, "Expected '{' in struct variable declaration", curTok->row, curTok->col);
//     getNextToken(); //? eat '{'

//     std::map<std::string, unique_ptr<Expression>> props;
//     std::vector<std::string> property_insetion_order;
//     while (curTok->type != TokenType::tok_close_curly_bracket)
//     {
//         throwIfCurTokNotType(TokenType::tok_identifier, "Expected identifier in struct variable declaration", curTok->row, curTok->col);
//         std::string property_name = curTok->value;
//         getNextToken(); //? eat property name

//         throwIfCurTokNotType(TokenType::tok_colon, "Expected ':' in struct variable declaration", curTok->row, curTok->col);
//         getNextToken(); //? eat ':'

//         auto val = parseExpression();
//         props[property_name] = std::move(val);
//         property_insetion_order.push_back(property_name);
//     }

//     getNextToken(); //? eat '}'

//     return std::make_unique<StructValueExpression>(std::move(props), property_insetion_order);
// }

// unique_ptr<Expression> Parser::parseUnaryPrefixOperationExpression()
// {
//     TokenType op = curTok->type;
//     getNextToken(); //? eat operator
//     auto expr = parseExpression(false);
//     return std::make_unique<UnaryPrefixOperationExpression>(op, std::move(expr));
// }

// unique_ptr<Expression> Parser::parseParenExpression()
// {
//     getNextToken(); //? eat '('
//     auto expr = parseExpression(false);
//     throwIfCurTokNotType(TokenType::tok_close_paren, "Expected ')' at end of parentheses expression", curTok->row, curTok->col);
//     getNextToken(); //? eat ')'
//     return expr;
// }

// unique_ptr<Expression> Parser::parseBinopRhs(int expression_precedence, unique_ptr<Expression> lhs)
// {
//     while (true)
//     {
//         int tok_precedence = getTokenPrecedence();
//         if (tok_precedence < expression_precedence)
//             return lhs;

//         TokenType binop = curTok->type;

//         getNextToken(); //? eat operator

//         auto rhs = parsePrimary();
//         if (!rhs)
//             error("Error parsing binary operator right hand side", toks[curTokIndex - 1]->row, toks[curTokIndex - 1]->col);

//         int next_precedence = getTokenPrecedence();
//         if (tok_precedence < next_precedence)
//         {
//             rhs = parseBinopRhs(tok_precedence + 1, std::move(rhs));
//             if (!rhs)
//                 error("Error parsing binary operator right hand side", toks[curTokIndex - 1]->row, toks[curTokIndex - 1]->col);
//         }

//         lhs = std::make_unique<BinaryOperationExpression>(binop, std::move(lhs), std::move(rhs));
//     }

//     return nullptr;
// }

// unique_ptr<Expression> Parser::parseSquareBracketExpression(std::unique_ptr<Expression> expr)
// {
//     getNextToken(); //? eat '['

//     auto index = parseExpression(false);

//     throwIfCurTokNotType(TokenType::tok_close_square_bracket, "Expected ']' at end of index access", curTok->row, curTok->col);
//     getNextToken(); //? eat ']'

//     return std::make_unique<IndexAccessedExpression>(std::move(expr), std::move(index));
// }

// unique_ptr<Expression> Parser::parseNumberExpression()
// {
//     double val = std::stod(curTok->value.c_str());
//     getNextToken(); //? eat number
//     return std::make_unique<NumberExpression>(val);
// }

// unique_ptr<Expression> Parser::parseStringLiteralExpression()
// {
//     std::string val = curTok->value;
//     getNextToken(); //? eat string literal
//     return std::make_unique<StringLiteralExpression>(val);
// }

// unique_ptr<Expression> Parser::parseIdentifierExpression()
// {
//     if (toks[curTokIndex + 1]->type == TokenType::tok_open_paren)
//     {
//         std::string fn_name = curTok->value;
//         std::vector<unique_ptr<Expression>> params;
//         getNextToken(); //? eat function name
//         getNextToken(); //? eat '('
//         int i = 0;
//         while (curTok->type != TokenType::tok_close_paren)
//         {
//             auto param = parseExpression(false);
//             params.push_back(std::move(param));
//             if (curTok->type == TokenType::tok_comma)
//                 getNextToken();
//         }

//         getNextToken(); //? eat ')'
//         return std::make_unique<FunctionCallExpression>(fn_name, std::move(params));
//     }

//     auto expr = std::make_unique<VariableReferenceExpression>(curTok->value);
//     getNextToken(); //? eat variable name
//     return expr;
// }

// void Parser::getNextToken()
// {
//     curTokIndex++;
//     curTok = std::move(toks[curTokIndex]);
// }

// int Parser::getTokenPrecedence()
// {
//     int tok_precedence = binopPrecedence[curTok->value];
//     if (tok_precedence <= 0)
//         return -1;
//     return tok_precedence;
// }

// void Parser::error(const char *msg, int row, int col)
// {
//     std::cout << msg << " at line " << row << " column " << col << std::endl;
//     exit(1);
// }

// void Parser::throwIfCurTokNotType(TokenType type, const char *msg, int line, int position)
// {
//     if (curTok->type != type)
//         error(msg, line, position);
// }

// std::string Parser::getType()
// {
//     std::string type = curTok->value;
//     getNextToken();
//     while (curTok->type == TokenType::tok_asterisk || curTok->type == TokenType::tok_ampersand)
//     {
//         type += curTok->value;
//         getNextToken();
//     }

//     return type;
// }

// std::vector<std::string> Parser::getStructTypes()
// {
//     return structTypes;
// }
