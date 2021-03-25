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
            {"||", 5},
            {"&&", 3},
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
        std::string currentlyPreferredType;
        std::string currentFunctionType;
        std::string currentFunctionName;
        std::map<std::string, std::map<std::string, std::string>> functionVariableTypes;
        std::map<std::string, std::map<std::string, bool>> functionVarRefsMutable;
        std::map<std::string, std::vector<std::string>> functionParamTypes;

        void error(std::string msg);
        void errIfCurTokNotType(TokenType type, std::string msg);
        void getNextToken();
        int getTokenPrecedence();
        unique_ptr<ASTNode> parseToken(const unique_ptr<Token> &tok);

        unique_ptr<ASTNode> parsePub();
        unique_ptr<ASTNode> parseMut(bool isPub = false);
        unique_ptr<ASTVariableDeclaration> parseVariableDeclaration(bool isPub = false, bool isMut = false);
        unique_ptr<ASTFunctionDeclaration> parseFn(bool isPub = false);
        Parameter parseParameter();
        std::string parseType();
        unique_ptr<ASTExpression> parseExpression(bool needsSemicolon = true);
        unique_ptr<ASTExpression> parsePrimary();
        unique_ptr<ASTExpression> parseBinopRHS(int expressionPrecedence, unique_ptr<ASTExpression> lhs);
        unique_ptr<ASTExpression> parseNumberExpression();
        unique_ptr<ASTExpression> parseParenExpression();
        unique_ptr<ASTExpression> parseIdentifierExpression();
        unique_ptr<ASTExpression> parseFunctionCallExpression(std::string fnName);
        unique_ptr<ASTExpression> parseTypecast();

        unique_ptr<ASTReturnStatement> parseReturn();
        unique_ptr<ASTIfStatement> parseIfStatement();
        unique_ptr<ASTForLoop> parseForLoop();

    public:
        Nodes parseTokens(Tokens tokens);
    };
} // namespace ssc

#endif