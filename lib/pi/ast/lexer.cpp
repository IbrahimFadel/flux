#include "lexer.h"

using namespace ssc;

void Lexer::printTokens(const Tokens &tokens)
{
    for (auto &token : tokens)
    {
        std::cout << "[ " << token->type << " : " << token->value << " ] -- " << token->row << ", " << token->col << '\n';
    }
}

// void Lexer::error(std::string msg)
// {

//     std::cerr << "\033[1;31m"
//               << "Lexer Error: "
//               << "\033[0m" << msg << std::endl;
//     exit(1);
// }

void Lexer::reset()
{
    // In case `tokenize` is called again, reset everything
    fileContent.clear();
    tokens.clear();
    token.clear();
    row = 1;
    col = 1;
    state = LexerState::normal;
}

Tokens Lexer::tokenize(std::vector<std::string> content)
{
    reset();
    fileContent = content;

    for (auto &line : fileContent)
    {
        for (int i = 0; i < line.size(); i++)
        {
            char c = line[i];

            switch (state)
            {
            case LexerState::normal:
            {
                if (c == '"')
                {
                    state = LexerState::string;
                    continue;
                }
                else if (c == '/' && line[i + 1] == '/')
                {
                    state = LexerState::lineComment;
                    i++;
                    continue;
                }
                else if (c == '/' && line[i + 1] == '*')
                {
                    state = LexerState::blockComment;
                    i++;
                    continue;
                }
                break;
            }
            case LexerState::string:
            {
                if (c == '"' && line[i - 1] != '\\')
                {
                    addValidLexeme();
                    state = LexerState::normal;
                    continue;
                }
                break;
            }
            case LexerState::lineComment:
            {
                if (c != '\n')
                {
                    continue;
                }
                state = LexerState::normal;
                continue;
            }
            case LexerState::blockComment:
            {
                if (c == '*' && line[i + 1] == '/')
                {
                    state = LexerState::normal;
                    i++;
                    continue;
                }
                continue;
            }
            }

            if (state == LexerState::string)
            {
                token += c;
                continue;
            }

            switch (c)
            {
            case '\n':
                addValidLexeme();
                continue;
            case ' ':
                addValidLexeme();
                continue;
            case '(':
                addValidLexeme(c);
                continue;
            case ')':
                addValidLexeme(c);
                continue;
            case ';':
                addValidLexeme(c);
                continue;
            case ',':
                addValidLexeme(c);
                continue;
            case '-':
            {
                if (line[i + 1] == '>')
                {
                    addValidLexeme();
                    token += "->";
                    addValidLexeme();
                    i++;
                    continue;
                }
                else
                {
                    addValidLexeme(c);
                    continue;
                }
                break;
            }
            case '+':
                addValidLexeme(c);
                continue;
            case '*':
                addValidLexeme(c);
                continue;
            case '/':
                addValidLexeme(c);
                continue;
            default:
                break;
            }

            token += c;

            col++;
        }

        row++;
        col = 1;
    }

    if (token.size() > 0)
    {
        addValidLexeme();
    }

    auto tok = constructToken(TokenType::tokEOF);
    tok->row = -1;
    tok->col = -1;
    tok->value = "EOF";
    tokens.push_back(std::move(tok));

    return std::move(tokens);
}

void Lexer::addValidLexeme(char c)
{
    if (state != LexerState::string)
    {
        std::string::iterator endPos = std::remove(token.begin(), token.end(), ' ');
        token.erase(endPos, token.end());
    }

    unique_ptr<Token> tok;

    if (token == "pub")
        tok = constructToken(TokenType::tokPub);
    else if (token == "fn")
        tok = constructToken(TokenType::tokFn);
    else if (token == "mut")
        tok = constructToken(TokenType::tokMut);
    else if (token == "return")
        tok = constructToken(TokenType::tokReturn);
    else if (token == "if")
        tok = constructToken(TokenType::tokIf);
    else if (token == "for")
        tok = constructToken(TokenType::tokFor);
    else if (token == "while")
        tok = constructToken(TokenType::tokWhile);
    else if (token == "class")
        tok = constructToken(TokenType::tokClass);
    else if (token == "constructor")
        tok = constructToken(TokenType::tokConstructor);
    else if (token == "new")
        tok = constructToken(TokenType::tokNew);

    else if (token == "i64")
        tok = constructToken(TokenType::tokI64);
    else if (token == "u64")
        tok = constructToken(TokenType::tokU64);
    else if (token == "i32")
        tok = constructToken(TokenType::tokI32);
    else if (token == "u32")
        tok = constructToken(TokenType::tokU32);
    else if (token == "i16")
        tok = constructToken(TokenType::tokI16);
    else if (token == "u16")
        tok = constructToken(TokenType::tokU16);
    else if (token == "i8")
        tok = constructToken(TokenType::tokI8);
    else if (token == "u8")
        tok = constructToken(TokenType::tokU8);
    else if (token == "f64")
        tok = constructToken(TokenType::tokF64);
    else if (token == "f32")
        tok = constructToken(TokenType::tokF32);
    else if (token == "bool")
        tok = constructToken(TokenType::tokBool);
    else if (token == "nullptr")
        tok = constructToken(TokenType::tokNullptr);
    else if (token == "void")
        tok = constructToken(TokenType::tokVoid);

    else if (token == "->")
        tok = constructToken(TokenType::tokArrow);
    else if (token == ".")
        tok = constructToken(TokenType::tokPeriod);
    else if (token == "=")
        tok = constructToken(TokenType::tokEq);
    else if (token == "+=")
        tok = constructToken(TokenType::tokPlusEq);
    else if (token == "-=")
        tok = constructToken(TokenType::tokMinusEq);
    else if (token == "+")
        tok = constructToken(TokenType::tokPlus);
    else if (token == "-")
        tok = constructToken(TokenType::tokMinus);
    else if (token == "*")
        tok = constructToken(TokenType::tokAsterisk);
    else if (token == "/")
        tok = constructToken(TokenType::tokSlash);
    else if (token == "==")
        tok = constructToken(TokenType::tokCompareEq);
    else if (token == "!=")
        tok = constructToken(TokenType::tokCompareNe);
    else if (token == ">")
        tok = constructToken(TokenType::tokCompareGt);
    else if (token == "<")
        tok = constructToken(TokenType::tokCompareLt);
    else if (token == ">=")
        tok = constructToken(TokenType::tokCompareGtEq);
    else if (token == "<=")
        tok = constructToken(TokenType::tokCompareLtEq);
    else if (token == "||")
        tok = constructToken(TokenType::tokOr);
    else if (token == "&&")
        tok = constructToken(TokenType::tokAnd);
    else if (token == "&")
        tok = constructToken(TokenType::tokAmpersand);

    else if (token == "(")
        tok = constructToken(TokenType::tokOpenParen);
    else if (token == ")")
        tok = constructToken(TokenType::tokCloseParen);
    else if (token == "{")
        tok = constructToken(TokenType::tokOpenCurlyBracket);
    else if (token == "}")
        tok = constructToken(TokenType::tokCloseCurlyBracket);
    else if (token == "[")
        tok = constructToken(TokenType::tokOpenSquareBracket);
    else if (token == "]")
        tok = constructToken(TokenType::tokCloseSquareBracket);
    else if (token == ";")
        tok = constructToken(TokenType::tokSemicolon);
    else if (token == ",")
        tok = constructToken(TokenType::tokComma);

    else if (token == "true")
        tok = constructToken(TokenType::tokTrue);
    else if (token == "false")
        tok = constructToken(TokenType::tokFalse);
    else
    {
        if (state == LexerState::string)
        {
            tok = constructToken(TokenType::tokStringLit);
        }
        else if (isNumber(token.c_str()))
        {
            tok = constructToken(TokenType::tokNumberLit);
        }
        else
        {
            tok = constructToken(TokenType::tokIdentifier);
        }
    }

    if (tok && token.size() > 0)
    {
        tokens.push_back(std::move(tok));
        token.clear();
    }
    if (c)
    {
        token = std::string(1, c);
        addValidLexeme();
        col++;
    }
}

unique_ptr<Token> Lexer::constructToken(TokenType type)
{
    auto tok = std::make_unique<Token>();
    tok->row = row;
    int correctedCol = col;
    if (token.size() > 1)
        correctedCol -= token.size();
    tok->col = correctedCol;
    tok->value = token;
    tok->type = type;
    return tok;
}

bool Lexer::isNumber(const char *str)
{
    if (isFloatingPoint(token.c_str()))
        return true;
    return !token.empty() && std::find_if(token.begin(),
                                          token.end(), [](char c) { return !std::isdigit(c); }) == token.end();
}

bool Lexer::isFloatingPoint(const char *str)
{
    char *endptr = 0;
    strtod(str, &endptr);

    if (*endptr != '\0' || endptr == str)
        return false;
    return true;
}
