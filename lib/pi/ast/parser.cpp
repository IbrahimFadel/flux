#include "parser.h"

using namespace ssc;

Nodes Parser::parseTokens(Tokens toks)
{
    tokens = std::move(toks);
    curTokIndex = 0;
    curTok = std::move(tokens[curTokIndex]);

    std::vector<unique_ptr<ASTNode>> nodes;

    while (curTok->type != TokenType::tokEOF)
    {
        auto node = parseToken(curTok);
        nodes.push_back(std::move(node));
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

unique_ptr<ASTNode> Parser::parseToken(const unique_ptr<Token> &tok)
{
    switch (tok->type)
    {
    case TokenType::tokPub:
        return parsePub();
    case TokenType::tokMut:
        return parseMut();
    case TokenType::tokFn:
        return parseFn();
    case TokenType::tokReturn:
        return parseReturn();
    case TokenType::tokIf:
        return parseIfStatement();
    case TokenType::tokFor:
        return parseForLoop();
    case TokenType::tokClass:
        return parseClassDeclaration();
    case TokenType::tokImport:
        return parseImportStatement();
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
    case TokenType::tokIdentifier:
    {
        if (std::count(classTypes.begin(), classTypes.end(), tok->value))
        {
            return parseVariableDeclaration();
        }
        return parseExpression();
    }
    default:
        error("Could not parse token: " + curTok->value);
        break;
    }

    return nullptr;
}

unique_ptr<ASTImportStatement> Parser::parseImportStatement()
{
    getNextToken(); //? eat 'import'

    errIfCurTokNotType(TokenType::tokStringLit, "Expected string literal after 'import'");
    std::string path = curTok->value;
    getNextToken(); //? eat path

    errIfCurTokNotType(TokenType::tokSemicolon, "Expected ';' after import path");
    getNextToken(); //? eat ';'

    return std::make_unique<ASTImportStatement>(path);
}

unique_ptr<ASTClassDeclaration> Parser::parseClassDeclaration()
{
    getNextToken(); //? eat 'class'

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier after 'class'");
    std::string name = curTok->value;
    getNextToken(); //? eat name

    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' in class definition");
    getNextToken(); //? eat '{'

    Parameter structParam;
    structParam.mut = false;
    structParam.type = name + "*";
    structParam.name = "this";

    std::vector<unique_ptr<ASTVariableDeclaration>> properties;
    std::vector<unique_ptr<ASTFunctionDefinition>> methods;
    unique_ptr<ASTFunctionDefinition> constructor = nullptr;

    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        if (curTok->type == TokenType::tokConstructor)
        {
            constructor = parseConstructor(name + ".constructor", name);
        }
        else if (curTok->type == TokenType::tokFn)
        {
            auto method = parseFn();
            auto methodParams = method->getParams();
            methodParams.insert(methodParams.begin(), structParam);
            method->setParams(methodParams);
            method->setName(name + "." + method->getName());
            methods.push_back(std::move(method));
        }
        else if (curTok->type == TokenType::tokPub)
        {
            auto node = parsePub();

            auto var = dynamic_cast<ASTVariableDeclaration *>(node.get());
            unique_ptr<ASTVariableDeclaration> varDec;
            if (var != nullptr)
            {
                node.release();
                varDec.reset(var);

                properties.push_back(std::move(varDec));
                continue;
            }

            auto fn = dynamic_cast<ASTFunctionDefinition *>(node.get());
            unique_ptr<ASTFunctionDefinition> fnDec;
            if (fn != nullptr)
            {
                node.release();
                fnDec.reset(fn);

                auto methodParams = fnDec->getParams();
                methodParams.insert(methodParams.begin(), structParam);
                fnDec->setParams(methodParams);
                fnDec->setName(name + "." + fnDec->getName());

                methods.push_back(std::move(fnDec));
            }
        }
        else if (curTok->type == TokenType::tokMut)
        {
            auto node = parseMut();

            auto var = dynamic_cast<ASTVariableDeclaration *>(node.get());
            unique_ptr<ASTVariableDeclaration> varDec;
            if (var != nullptr)
            {
                node.release();
                varDec.reset(var);

                properties.push_back(std::move(varDec));
            }
        }
        else
        {
            auto var = parseVariableDeclaration();
            properties.push_back(std::move(var));
        }
    }

    getNextToken(); //? eat '}'

    classTypes.push_back(name);

    return std::make_unique<ASTClassDeclaration>(name, std::move(constructor), std::move(properties), std::move(methods));
}

unique_ptr<ASTFunctionDefinition> Parser::parseConstructor(std::string name, std::string className)
{
    getNextToken(); //? eat 'constructor'

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' after 'constructor'");
    getNextToken(); //? eat '('

    Parameter classParam = {
        .mut = false,
        .type = className + "*",
        .name = "this"};

    std::vector<Parameter> params = {classParam};

    classConstructorParamTypes[className] = {};

    while (curTok->type != TokenType::tokCloseParen)
    {
        auto param = parseParameter();
        params.push_back(param);
        classConstructorParamTypes[className].push_back(param.type);
        if (curTok->type == TokenType::tokComma)
        {
            getNextToken(); //? eat ','
        }
        else if (curTok->type != TokenType::tokCloseParen)
        {
            error("Expected ')' at end of constructor parameter list");
        }
    }

    getNextToken(); //? eat ')'

    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' in constructor");
    getNextToken(); //? eat '{'

    std::vector<unique_ptr<ASTNode>> then;

    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        auto node = parseToken(curTok);
        then.push_back(std::move(node));
    }

    getNextToken(); //? eat '}'

    return std::make_unique<ASTFunctionDefinition>(true, name, params, std::string("void"), std::move(then));
}

unique_ptr<ASTForLoop> Parser::parseForLoop()
{
    getNextToken(); //? eat 'for'

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' after 'for'");
    getNextToken(); //? eat '('

    unique_ptr<ASTExpression> initialClauseExpression = nullptr;
    unique_ptr<ASTVariableDeclaration> initialClauseVarDec = nullptr;

    if (curTok->type == TokenType::tokIdentifier)
    {
        initialClauseExpression = parseExpression();
    }
    else
    {
        if (curTok->type == TokenType::tokMut)
        {
            getNextToken(); //? eat 'mut'
            initialClauseVarDec = parseVariableDeclaration(false, true);
        }
        else
        {
            initialClauseVarDec = parseVariableDeclaration();
        }
    }

    auto conditionExpression = parseExpression();
    auto actionExpression = parseExpression(false);

    errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' after 'for'");
    getNextToken(); //? eat ')'
    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' after 'for'");
    getNextToken(); //? eat '{'

    std::vector<unique_ptr<ASTNode>> then;
    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        auto node = parseToken(curTok);
        then.push_back(std::move(node));
    }

    getNextToken(); //? eat '}'

    if (initialClauseExpression)
    {
        return std::make_unique<ASTForLoop>(std::move(initialClauseExpression), std::move(conditionExpression), std::move(actionExpression), std::move(then));
    }
    return std::make_unique<ASTForLoop>(std::move(initialClauseVarDec), std::move(conditionExpression), std::move(actionExpression), std::move(then));
}

unique_ptr<ASTReturnStatement> Parser::parseReturn()
{
    getNextToken(); //? eat 'return'
    currentlyPreferredType = currentFunctionType;
    auto value = parseExpression();
    return std::make_unique<ASTReturnStatement>(std::move(value));
}

unique_ptr<ASTNode> Parser::parseMut(bool isPub)
{
    getNextToken(); //? eat 'mut'
    return parseVariableDeclaration(isPub, true);
}

unique_ptr<ASTExpression> Parser::parseExpression(bool needsSemicolon)
{
    auto lhs = parsePrimary();
    if (!lhs)
        error("Error parsing primary");

    auto binop_node = parseBinopRHS(0, std::move(lhs));

    if (needsSemicolon)
    {
        errIfCurTokNotType(TokenType::tokSemicolon, "Expected ';' at end of expression");
        getNextToken(); //? eat ';'
    }
    return binop_node;
}

unique_ptr<ASTExpression> Parser::parsePrimary()
{
    switch (curTok->type)
    {
    case TokenType::tokNumberLit:
        return parseNumberExpression();
    case TokenType::tokIdentifier:
        return parseIdentifierExpression();
    case TokenType::tokOpenParen:
        return parseParenExpression();
    case TokenType::tokNew:
        return parseUnaryPrefixOperationExpression();
    case TokenType::tokI64:
        return parseTypecast();
    case TokenType::tokU64:
        return parseTypecast();
    case TokenType::tokI32:
        return parseTypecast();
    case TokenType::tokU32:
        return parseTypecast();
    case TokenType::tokI16:
        return parseTypecast();
    case TokenType::tokU16:
        return parseTypecast();
    case TokenType::tokI8:
        return parseTypecast();
    case TokenType::tokU8:
        return parseTypecast();
    case TokenType::tokBool:
        return parseTypecast();
    case TokenType::tokF64:
        return parseTypecast();
    case TokenType::tokF32:
        return parseTypecast();
    case TokenType::tokNullptr:
        return parseNullptr();
    }
    return nullptr;
}

unique_ptr<ASTNullptrExpression> Parser::parseNullptr()
{
    getNextToken(); //? eat 'nullptr'
    return std::make_unique<ASTNullptrExpression>(currentlyPreferredType);
}

unique_ptr<ASTUnaryPrefixOperationExpression> Parser::parseUnaryPrefixOperationExpression()
{
    TokenType op = curTok->type;
    getNextToken(); //? eat operator

    auto value = parseExpression(false);

    return std::make_unique<ASTUnaryPrefixOperationExpression>(op, std::move(value), currentlyPreferredType);
}

unique_ptr<ASTIfStatement> Parser::parseIfStatement()
{
    getNextToken(); //? eat 'if'

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' after 'if'");
    getNextToken(); //? eat '('

    auto condition = parseExpression(false);

    errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' after 'if'");
    getNextToken(); //? eat ')'
    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' after 'if'");
    getNextToken(); //? eat '{'

    std::vector<unique_ptr<ASTNode>> then;
    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        auto node = parseToken(curTok);
        then.push_back(std::move(node));
    }
    getNextToken(); //? eat '}'

    return std::make_unique<ASTIfStatement>(std::move(condition), std::move(then));
}

unique_ptr<ASTExpression> Parser::parseTypecast()
{
    auto type = parseType();

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' in typecast to " + type);

    getNextToken(); //? eat '('

    auto expr = parseExpression(false);

    errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' in typecast to " + type);
    getNextToken(); //? eat ')'

    return std::make_unique<ASTTypecastExpression>(std::move(expr), type);
}

unique_ptr<ASTExpression> Parser::parseNumberExpression()
{
    double value = std::stod(curTok->value);
    getNextToken(); //? eat number
    return std::make_unique<ASTNumberExpression>(value, currentlyPreferredType);
}

unique_ptr<ASTExpression> Parser::parseIdentifierExpression()
{
    std::string name = curTok->value;
    getNextToken(); //? eat id

    std::cout << "Parse id: " << name << '\n';

    // currentlyPreferredType = functionVariableTypes[currentFunctionName][name];

    if (curTok->type == TokenType::tokOpenParen)
    {
        if (std::count(classTypes.begin(), classTypes.end(), name))
        {
            //? This is a class construction
            return parseClassConstructionExpression(name);
        }
        else
        {
            return parseFunctionCallExpression(name);
        }
    }

    return std::make_unique<ASTVariableReferenceExpression>(name, functionVariableTypes[currentFunctionName][name], functionVarRefsMutable[currentFunctionName][name]);
}

unique_ptr<ASTClassConstructionExpression> Parser::parseClassConstructionExpression(std::string className)
{
    getNextToken(); //? eat '('

    std::vector<unique_ptr<ASTExpression>> parameters;

    int i = 0;
    while (curTok->type != TokenType::tokCloseParen)
    {
        currentlyPreferredType = classConstructorParamTypes[className][i];
        auto param = parseExpression(false);
        parameters.push_back(std::move(param));

        if (curTok->type == TokenType::tokComma)
        {
            getNextToken(); //? eat ','
        }
        else
        {
            errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' at end of class construction parameter listing");
        }
        i++;
    }

    getNextToken(); //? eat ')'

    return std::make_unique<ASTClassConstructionExpression>(className, std::move(parameters));
}

unique_ptr<ASTExpression> Parser::parseFunctionCallExpression(std::string fnName)
{
    getNextToken(); //? eat '('

    // if (!functionParamTypes.count(fnName) && !std::count(classTypes.begin(), classTypes.end(), fnName))
    // {
    //     error("Function call to '" + fnName + "' which has not been parsed yet");
    // }

    std::cout << "fn call: " << fnName << '\n';

    std::vector<unique_ptr<ASTExpression>> params;
    int i = 0;
    while (curTok->type != TokenType::tokCloseParen)
    {
        std::map<std::string, std::vector<std::string>>::iterator it = functionParamTypes.find(fnName);
        if (it != functionParamTypes.end())
        {
            currentlyPreferredType = it->second[i];
        }
        // std::cout << "fn call: " << fnName << '\n';
        auto param = parseExpression(false);
        params.push_back(std::move(param));

        if (curTok->type == TokenType::tokComma)
        {
            getNextToken(); //? eat ','
        }
        else
        {
            errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' at end of function call expression");
        }
        i++;
    }

    getNextToken(); //? eat ')'

    return std::make_unique<ASTFunctionCallExpression>(fnName, std::move(params), currentlyPreferredType);
}

unique_ptr<ASTExpression> Parser::parseParenExpression()
{
    getNextToken(); //? eat '('
    auto expr = parseExpression(false);
    errIfCurTokNotType(TokenType::tokCloseParen, "Expected ')' at end of paren expression");
    getNextToken(); //? eat ')'
    return expr;
}

unique_ptr<ASTExpression> Parser::parseBinopRHS(int expressionPrecedence, unique_ptr<ASTExpression> lhs)
{
    while (true)
    {
        int tok_precedence = getTokenPrecedence();
        if (tok_precedence < expressionPrecedence)
            return lhs;

        TokenType binop = curTok->type;

        getNextToken(); //? eat operator

        currentlyPreferredType = lhs->getType();

        auto rhs = parsePrimary();
        if (!rhs)
            error("Error parsing binary operator right hand side");

        if (binop == TokenType::tokArrow)
        {
            currentlyPreferredType = rhs->getType();
        }

        int next_precedence = getTokenPrecedence();
        if (tok_precedence < next_precedence)
        {
            rhs = parseBinopRHS(tok_precedence + 1, std::move(rhs));
            if (!rhs)
                error("Error parsing binary operator right hand side");
        }

        lhs = std::make_unique<ASTBinaryOperationExpression>(std::move(lhs), std::move(rhs), binop, currentlyPreferredType);
    }

    return nullptr;
}

unique_ptr<ASTNode> Parser::parsePub()
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

unique_ptr<ASTVariableDeclaration> Parser::parseVariableDeclaration(bool isPub, bool isMut)
{
    if (isPub && !parsingInGlobalScope)
    {
        std::string msg = "Cannot declare public variable outside global scope. Line " + std::to_string(curTok->row) + ", col " + std::to_string(curTok->col);
        error(msg);
    }

    std::string varType = parseType();
    currentlyPreferredType = varType;

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier following 'pub'");
    std::string varName = curTok->value;
    getNextToken(); //? eat name

    functionVariableTypes[currentFunctionName][varName] = varType;
    functionVarRefsMutable[currentFunctionName][varName] = isMut;

    if (curTok->type == TokenType::tokSemicolon)
    {
        getNextToken(); //? eat ';'
        return std::make_unique<ASTVariableDeclaration>(isPub, isMut, varType, varName, nullptr);
    }

    errIfCurTokNotType(TokenType::tokEq, "Expected '=' following variable name");
    getNextToken(); //? eat '='

    auto value = parseExpression();
    return std::make_unique<ASTVariableDeclaration>(isPub, isMut, varType, varName, std::move(value));
}

unique_ptr<ASTFunctionDefinition> Parser::parseFn(bool isPub)
{
    getNextToken(); //? eat 'fn'

    errIfCurTokNotType(TokenType::tokIdentifier, "Expected identifier following 'fn'");
    std::string functionName = curTok->value;
    currentFunctionName = functionName;
    getNextToken(); //? eat fn name

    functionParamTypes[functionName] = {};

    errIfCurTokNotType(TokenType::tokOpenParen, "Expected '(' following function name");
    getNextToken(); //? eat '('

    std::vector<Parameter> parameters;
    while (curTok->type != TokenType::tokCloseParen)
    {
        auto param = parseParameter();
        parameters.push_back(param);
        functionParamTypes[functionName].push_back(param.type);

        functionVariableTypes[currentFunctionName][param.name] = param.type;
        functionVarRefsMutable[currentFunctionName][param.name] = param.mut;

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
    currentFunctionType = type;

    if (curTok->type == TokenType::tokSemicolon)
    {
        getNextToken(); //? eat ';'
        return std::make_unique<ASTFunctionDefinition>(isPub, functionName, parameters, type, std::vector<unique_ptr<ASTNode>>(0));
    }

    errIfCurTokNotType(TokenType::tokOpenCurlyBracket, "Expected '{' following function return type");
    getNextToken();

    parsingInGlobalScope = false;
    std::vector<unique_ptr<ASTNode>> then;
    while (curTok->type != TokenType::tokCloseCurlyBracket)
    {
        auto node = parseToken(curTok);
        then.push_back(std::move(node));
    }
    getNextToken(); //? eat '}'
    parsingInGlobalScope = true;

    return std::make_unique<ASTFunctionDefinition>(isPub, functionName, parameters, type, std::move(then));
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
