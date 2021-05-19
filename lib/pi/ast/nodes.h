#ifndef SSC_AST_NODES_H
#define SSC_AST_NODES_H

#include <vector>
#include <string>
#include <memory>
#include <map>
#include <variant>

using std::unique_ptr;

namespace ssc
{
    class ASTNode;
    class ASTExpression;
    class ASTFunctionDefinition;
    class ASTVariableDeclaration;
    struct Parameter;
    class ASTReturnStatement;
    class ASTClassDeclaration;
    typedef std::vector<unique_ptr<ASTNode>> Nodes;
} // namespace ssc

#include "ir/context.h"
#include "llvm/IR/Value.h"

#include "lexer.h"

namespace ssc
{
    class ASTNode
    {
    private:
    public:
        virtual llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext) = 0;
    };

    class ASTExpression : public ASTNode
    {
    private:
    public:
        virtual llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext) = 0;
        virtual std::string getType() = 0;
    };

    class ASTNumberExpression : public ASTExpression
    {
    private:
        double value;
        std::string type;

    public:
        ASTNumberExpression(double value, std::string type) : value(value), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        double getValue() { return value; }
        std::string getType() { return type; }
    };

    class ASTBinaryOperationExpression : public ASTExpression
    {
    private:
        unique_ptr<ASTExpression> lhs;
        unique_ptr<ASTExpression> rhs;
        TokenType op;
        std::string type;

        llvm::Value *codegenBinopSumDiffProdQuot(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op);
        llvm::Value *codegenBinopEq(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs);
        llvm::Value *codegenBinopComp(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op);
        llvm::Value *codegenBinopArrow(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs);

    public:
        ASTBinaryOperationExpression(unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op, std::string type) : lhs(std::move(lhs)), rhs(std::move(rhs)), op(op), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        unique_ptr<ASTExpression> &getLHS() { return lhs; }
        unique_ptr<ASTExpression> &getRHS() { return rhs; }
        std::string getType() { return type; }
    };

    class ASTVariableReferenceExpression : public ASTExpression
    {
    private:
        std::string name;
        std::string type;
        bool isMut;

    public:
        ASTVariableReferenceExpression(std::string name, std::string type, bool isMut) : name(name), type(type), isMut(isMut){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getName() { return name; }
        std::string getType() { return type; }
        bool getIsMut() { return isMut; }
    };

    struct Parameter
    {
        bool mut;
        std::string type;
        std::string name;
    };

    class ASTFunctionDefinition : public ASTNode
    {
    private:
        bool pub;
        std::string name;
        std::vector<Parameter> parameters;
        std::string returnType;
        std::vector<unique_ptr<ASTNode>> then;

        std::map<std::string, ASTVariableDeclaration *> variables;
        std::map<std::string, llvm::Value *> mutables;
        std::map<std::string, llvm::Value *> constants;

        llvm::Function *codegenPrototype(const unique_ptr<CodegenContext> &codegenContext);
        void createFunctionParamAllocas(const unique_ptr<CodegenContext> &codegenContext, llvm::Function *f);

    public:
        ASTFunctionDefinition(bool pub, std::string name, std::vector<Parameter> parameters, std::string returnType, std::vector<unique_ptr<ASTNode>> then) : pub(pub), name(name), parameters(parameters), returnType(returnType), then(std::move(then)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        void setVariable(std::string name, ASTVariableDeclaration *v) { variables[name] = v; }
        ASTVariableDeclaration *getVariable(std::string name) { return variables[name]; }
        std::map<std::string, llvm::Value *> getConstants() { return constants; };
        std::map<std::string, llvm::Value *> getMutables() { return mutables; };
        void setName(std::string n) { name = n; }
        std::string getName() { return name; }
        void setMutable(std::string name, llvm::Value *val);
        llvm::Value *getMutable(std::string name);
        void setConstant(std::string name, llvm::Value *val);
        llvm::Value *getConstant(std::string name);
        bool getPub();
        std::vector<Parameter> getParameters();
        std::string getReturnType();
        const std::vector<unique_ptr<ASTNode>> &getThen();
        void setReturnType(std::string ty) { returnType = ty; }
        std::vector<Parameter> getParams() { return parameters; }
        void setParams(std::vector<Parameter> p) { parameters = p; };
    };

    class ASTVariableDeclaration : public ASTNode
    {
    private:
        bool pub;
        bool mut;
        std::string type;
        std::string name;
        unique_ptr<ASTExpression> value;

    public:
        ASTVariableDeclaration(bool pub, bool mut, std::string type, std::string name, unique_ptr<ASTExpression> value) : pub(pub), mut(mut), type(type), name(name), value(std::move(value)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getType() { return type; };
        std::string getName() { return name; };
        unique_ptr<ASTExpression> &getValue() { return value; };
    };

    class ASTReturnStatement : public ASTNode
    {
    private:
        unique_ptr<ASTExpression> value;

    public:
        ASTReturnStatement(unique_ptr<ASTExpression> value) : value(std::move(value)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        const unique_ptr<ASTExpression> &getValue() { return value; }
    };

    class ASTTypecastExpression : public ASTExpression
    {
    private:
        unique_ptr<ASTExpression> value;
        std::string type;

    public:
        ASTTypecastExpression(unique_ptr<ASTExpression> value, std::string type) : value(std::move(value)), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getType() { return type; }
    };

    class ASTIfStatement : public ASTNode
    {
    private:
        unique_ptr<ASTExpression> condition;
        std::vector<unique_ptr<ASTNode>> then;

    public:
        ASTIfStatement(unique_ptr<ASTExpression> condition, std::vector<unique_ptr<ASTNode>> then) : condition(std::move(condition)), then(std::move(then)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
    };

    class ASTForLoop : public ASTNode
    {
    private:
        unique_ptr<ASTExpression> initialClauseExpression;
        unique_ptr<ASTVariableDeclaration> initialClauseVarDec;
        std::unique_ptr<ssc::ASTExpression> condition;
        std::unique_ptr<ssc::ASTExpression> action;
        std::vector<unique_ptr<ASTNode>> then;

    public:
        ASTForLoop(unique_ptr<ASTExpression> initialClauseExpression, std::unique_ptr<ssc::ASTExpression> condition, std::unique_ptr<ssc::ASTExpression> action, std::vector<unique_ptr<ASTNode>> then) : initialClauseExpression(std::move(initialClauseExpression)), condition(std::move(condition)), action(std::move(action)), then(std::move(then)){};

        ASTForLoop(unique_ptr<ASTVariableDeclaration> initialClauseVarDec, std::unique_ptr<ssc::ASTExpression> condition, std::unique_ptr<ssc::ASTExpression> action, std::vector<unique_ptr<ASTNode>> then) : initialClauseVarDec(std::move(initialClauseVarDec)), condition(std::move(condition)), action(std::move(action)), then(std::move(then)){};

        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
    };

    class ASTFunctionCallExpression : public ASTExpression
    {
    private:
        std::string name;
        std::vector<unique_ptr<ASTExpression>> params;
        std::string type; // type of the variable it's being put into

    public:
        ASTFunctionCallExpression(std::string name, std::vector<unique_ptr<ASTExpression>> params, std::string type) : name(name), params(std::move(params)), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
        std::string getType() { return type; };

        void setName(std::string n) { name = n; };
        std::string getName() { return name; };
        std::vector<unique_ptr<ASTExpression>> &getParams() { return params; };
        void setParams(std::vector<unique_ptr<ASTExpression>> p) { params = std::move(p); }
        void insertParamAtFrom(std::unique_ptr<ASTExpression> p) { params.insert(params.begin(), std::move(p)); }
    };

    class ASTClassDeclaration : public ASTNode
    {
    private:
        std::string name;
        unique_ptr<ASTFunctionDefinition> constructor;
        std::vector<unique_ptr<ASTVariableDeclaration>> properties;
        std::vector<unique_ptr<ASTFunctionDefinition>> methods;

    public:
        ASTClassDeclaration(std::string name, unique_ptr<ASTFunctionDefinition> constructor, std::vector<unique_ptr<ASTVariableDeclaration>> properties, std::vector<unique_ptr<ASTFunctionDefinition>> methods) : name(name), constructor(std::move(constructor)), properties(std::move(properties)), methods(std::move(methods)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::vector<unique_ptr<ASTVariableDeclaration>> const &getProperties() const { return properties; }
        unique_ptr<ASTFunctionDefinition> &getConstructor() { return constructor; }
    };

    class ASTUnaryPrefixOperationExpression : public ASTExpression
    {
    private:
        TokenType op;
        unique_ptr<ASTExpression> value;
        std::string type;

    public:
        ASTUnaryPrefixOperationExpression(TokenType op, unique_ptr<ASTExpression> value, std::string type) : op(op), value(std::move(value)), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
        std::string getType() { return type; };

        llvm::Value *codegenNew(const unique_ptr<CodegenContext> &codegenContext);
    };

    class ASTClassConstructionExpression : public ASTExpression
    {
    private:
        std::string name;
        std::vector<unique_ptr<ASTExpression>> parameters;
        std::string type;

    public:
        ASTClassConstructionExpression(std::string name, std::vector<unique_ptr<ASTExpression>> parameters) : name(name), parameters(std::move(parameters)), type(name){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
        std::string getType()
        {
            return type;
        };
    };

    class ASTNullptrExpression : public ASTExpression
    {
    private:
        std::string type; // This is the currently prefferred type (so we know what type of pointer to make)

    public:
        ASTNullptrExpression(std::string type) : type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
        std::string getType()
        {
            return type;
        };
    };

    class ASTImportStatement : public ASTNode
    {
    private:
        std::string path;

    public:
        ASTImportStatement(std::string path) : path(path){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getPath() { return path; }
    };

}; // namespace ssc

#endif