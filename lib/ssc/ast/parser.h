#ifndef SSC_AST_PARSER_H
#define SSC_AST_PARSER_H

#include "lexer.h"
#include "nodes.h"

#include <map>

namespace ssc
{
    class Parser
    {
    private:
        Tokens tokens;
        unsigned curTokIndex;
        unique_ptr<Token> curTok;
        std::map<std::string, int> binopPrecedence = {
            {"=", 2},
            {"<", 10},
            {">", 10},
            {"<=", 10},
            {">=", 10},
            {"==", 10},
            {"!=", 10},
            {"+", 20},
            {"-", 20},
            {"*", 40},
            {"/", 40},
            {".", 50},
            {"->", 50},
        };
        bool parsingInGlobalScope = true;

        void error(std::string msg);
        void errIfCurTokNotType(TokenType type, std::string msg);
        void getNextToken();
        int getTokenPrecedence();
        unique_ptr<Node> parseToken(const unique_ptr<Token> &tok);

        unique_ptr<Node> parsePub();
        unique_ptr<FunctionDeclaration> parseFn(bool isPub = false);
        Parameter parseParameter();
        std::string parseType();
        unique_ptr<VariableDeclaration> parseVariableDeclaration(bool isPub = false, bool isMut = false);
        unique_ptr<Expression> parseExpression(bool needsSemicolon = true);
        unique_ptr<Expression> parsePrimary();
        unique_ptr<Expression> parseBinopRHS(int expressionPrecedence, unique_ptr<Expression> lhs);
        unique_ptr<Expression> parseNumberExpression();
        unique_ptr<Node> parseMut(bool isPub = false);

    public:
        Nodes parseTokens(Tokens tokens);
    };
} // namespace ssc

// #include "ast.h"

// namespace ssc
// {
//     class Parser
//     {
//     private:
//         int curTokIndex;
//         std::shared_ptr<ssc::Token> curTok;
//         ssc::Tokens toks;
//         std::map<std::string, int> binopPrecedence;
//         std::vector<std::string> structTypes;

//         unique_ptr<Node> parse_token(const std::shared_ptr<ssc::Token> &token);

//         //! Expression Parsing
//         unique_ptr<Expression> parseExpression(bool needs_semicolon = true);
//         unique_ptr<Expression> parsePrimary();
//         unique_ptr<Expression> parseBinopRhs(int expression_precedence, unique_ptr<Expression> lhs);
//         unique_ptr<Expression> parseParenExpression();
//         unique_ptr<Expression> parseNumberExpression();
//         unique_ptr<Expression> parseStringLiteralExpression();
//         unique_ptr<Expression> parseIdentifierExpression();
//         unique_ptr<Expression> parseUnaryPrefixOperationExpression();
//         unique_ptr<Expression> parseStructTypeDeclaration();
//         unique_ptr<Expression> parseStructValueExpression();
//         unique_ptr<Expression> parseSquareBracketExpression(std::unique_ptr<Expression> expr);

//         //! Variable Declaration Parsing
//         unique_ptr<VariableDeclaration> parseVariableDeclaration(bool is_struct = false);
//         unique_ptr<VariableDeclaration> parseStructVarDeclaration();

//         //! Function Parsing
//         unique_ptr<FunctionDeclaration> parseFunctionDeclaration();
//         unique_ptr<CodeBlock> parseCodeBlock();

//         //! Other
//         unique_ptr<IfStatement> parseIfStatement();
//         unique_ptr<ReturnStatement> parseReturnStatement();
//         unique_ptr<ImportStatement> parseImportStatement();

//         //! Common
//         void getNextToken();
//         int getTokenPrecedence();
//         void throwIfCurTokNotType(ssc::TokenType type, const char *msg, int line, int position);
//         std::string getType();
//         void error(const char *msg, int row, int col);

//     public:
//         Nodes parse_tokens(ssc::Tokens token);

//         std::vector<std::string> getStructTypes();
//     };
// } // namespace ssc

#endif