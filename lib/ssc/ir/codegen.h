#ifndef SSC_IR_CODEGEN_H
#define SSC_IR_CODEGEN_H

#include "driver/options.h"
#include "ast/parser.h"
#include "context.h"

// #include "llvm/IR/Value.h"
// #include "llvm/IR/IRBuilder.h"
// #include "llvm/IR/AssemblyAnnotationWriter.h"
// #include "llvm/Support/raw_ostream.h"

#include <llvm/IR/Module.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>
#include <llvm/IR/Verifier.h>
#include <llvm/Support/raw_ostream.h>

namespace ssc
{

    llvm::Module *codegenNodes(Nodes nodes, unique_ptr<CodegenContext> codegenContext);

    // static llvm::LLVMContext ctx;
    // static unique_ptr<llvm::Module> module;
    // static unique_ptr<llvm::IRBuilder<>> builder;
    // static std::string currentFunctionName;
    // static std::map<std::string, FunctionDeclaration *> functions;

    // static Nodes nodes;

    // unique_ptr<llvm::Module> codegenNodes(Nodes nodes);
    // void printModule(const unique_ptr<llvm::Module> &module);

    // static void error(std::string msg);
    // static llvm::Type *ssTypeToLLVMType(std::string type);
    // static llvm::Type *ssBaseTypeToLLVMType(std::string type);
    // static void codegenNode(unique_ptr<Node> node);
    // static llvm::Function *codegenFunctionPrototype(bool pub, std::string name, std::vector<Parameter> parameters, std::string returnType);

    // class CodeGenerator
    // {
    // private:
    //     // void codegenFunctionDeclaration(unique_ptr<Node> fnDecNode);
    //     // void createFunctionParamAllocas(llvm::Function *f, std::vector<Parameter> params);
    //     // void codegenVariableDeclaration(unique_ptr<Node> varDecNode);

    // public:
    // };

} // namespace ssc

// #include "driver/options.h"
// #include "parser/parser.h"
// // #include "dependency_tree.h"

// #include <llvm/IR/Module.h>
// #include <llvm/IR/IRBuilder.h>
// #include <llvm/IR/Value.h>
// #include <llvm/IR/LegacyPassManager.h>
// #include <llvm/IR/AssemblyAnnotationWriter.h>
// #include <llvm/IR/Verifier.h>

// #include <llvm/Bitcode/BitcodeWriter.h>

// #include <llvm/Transforms/InstCombine/InstCombine.h>
// #include <llvm/Transforms/Scalar.h>
// #include <llvm/Transforms/Scalar/GVN.h>
// #include <llvm/Transforms/Utils.h>

// #include <llvm/Support/TargetSelect.h>
// #include <llvm/Support/TargetRegistry.h>
// #include <llvm/Support/FileSystem.h>
// #include <llvm/Support/Host.h>
// #include <llvm/Support/raw_ostream.h>

// #include <llvm/Target/TargetOptions.h>
// #include <llvm/Target/TargetMachine.h>

// #include <lld/Common/Driver.h>

// #include <filesystem>
// #include <iostream>

// using std::cout;
// using std::endl;

// namespace fs = std::filesystem;
// // using namespace ssc::parser;

// namespace ssc
// {
//     class CodeGenerator
//     {
//     private:
//         Nodes nodes;
//         ssc::Options *compilerOptions;
//         llvm::LLVMContext context;
//         unique_ptr<llvm::Module> module;
//         unique_ptr<llvm::IRBuilder<>> builder;
//         unique_ptr<llvm::legacy::FunctionPassManager> fpm;

//         std::map<std::string, FunctionDeclaration *> functions;
//         std::string currentFunctionName;
//         llvm::Type *currentlyPreferredType;

//         std::vector<std::string> structTypeNames;
//         std::map<std::string, llvm::StructType *> llvmStructTypes;
//         std::map<std::string, std::map<std::string, std::string>> structProperties;
//         std::map<std::string, std::vector<std::string>> structPropertyInsertionOrders;

//         // unique_ptr<llvm::Module> code_gen_nodes(const Nodes &nodes, CompilerOptions options, unique_ptr<Program> parent_program);
//         void moduleToObj(llvm::Module *mod, std::string output_path);
//         // void declareImportedFunctions(Dependency_Tree *tree, fs::path path, llvm::Module *mod);
//         void codegenNode(const unique_ptr<Node> &node);
//         void initializeFPM(llvm::Module *mod);
//         void createFunctionParamAllocas(llvm::Function *f, std::map<std::string, std::string> params);
//         llvm::Value *createEntryBlockAlloca(llvm::Function *function, const std::string &name, llvm::Type *type);
//         llvm::Type *ssTypeToLLVMType(std::string type);
//         llvm::Type *ssBaseTypeToLLVMType(std::string type);
//         void declareCFunctions(llvm::Module *mod);
//         void declareStringFunctions(llvm::Module *mod);
//         void declareFunction(std::string name, std::vector<llvm::Type *> param_types, llvm::Type *return_type, llvm::Module *mod);
//         void declareStructType(std::vector<llvm::Type *> llvm_types, std::map<std::string, std::string> properties, std::vector<std::string> property_insetion_order, std::string name);
//         void printT(llvm::Type *ty);
//         void printV(llvm::Value *v);
//         void printModule(llvm::Module *mod);
//         void writeModuleToFile(llvm::Module *mod, std::string path);
//         void fatalError(std::string msg);

//         llvm::Value *codegenFunctionDeclaration(unique_ptr<FunctionDeclaration> fnDeclaration);
//         llvm::Value *codegenNumberExpression(unique_ptr<NumberExpression> numExpr);
//         llvm::Value *codegenStringLiteralExpression(unique_ptr<StringLiteralExpression> stringLitExpr);
//         llvm::Value *codegenVariableReferenceExpression(unique_ptr<VariableReferenceExpression> varRefExpr);
//         llvm::Value *codegenIndexAccessedExpression(unique_ptr<IndexAccessedExpression> indexAccessExpr);
//         llvm::Value *codegenBinaryOperationExpression(unique_ptr<BinaryOperationExpression> binopExpr);
//         llvm::Value *codegenNullptrExpression(unique_ptr<NullptrExpression> nullptrExpr);
//         llvm::Value *codegenUnaryPrefixOperationExpression(unique_ptr<UnaryPrefixOperationExpression> unaPrefixOpExpr);
//         llvm::Value *codegenCodeBlock(unique_ptr<CodeBlock> codeBlock);
//         llvm::Value *codegenVariableDeclaration(unique_ptr<VariableDeclaration> varDec);
//         llvm::Value *codegenStructTypeExpression(unique_ptr<StructTypeExpression> structTypeExpr);
//         llvm::Value *codegenStructValueExpression(unique_ptr<StructValueExpression> structValExpr);
//         llvm::Value *codegenIfStatement(unique_ptr<IfStatement> ifStatement);
//         llvm::Value *codegenReturnStatement(unique_ptr<ReturnStatement> retStatement);
//         llvm::Value *codegenImportStatement(unique_ptr<ImportStatement> importStatement);
//         llvm::Value *codegenFunctionCallExpression(unique_ptr<FunctionCallExpression> fnCallExpr);

//         llvm::Function *codeGenFunctionPrototype(std::map<std::string, std::string> params, std::string return_type, std::string function_name);

//         llvm::Value *codeGenStructVariableDeclaration(std::string name, std::string type, StructValueExpression *value);
//         llvm::Value *codeGenStringVariableDeclaration(std::string name, Expression *value);

//         llvm::Value *codeGenBinopSumDiffProdQuot(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs);
//         llvm::Value *codeGenBinopEq(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs);
//         llvm::Value *codeGenBinopArrow(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs);
//         llvm::Value *codeGenBinopPeriod(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs);
//         llvm::Value *codeGenBinopCmp(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, ssc::TokenType op);

//     public:
//         CodeGenerator(Nodes nodes, ssc::Options *compilerOptions, std::vector<std::string> structTypeNames);
//         // void create_module(const Nodes &nodes, ssc::Options options, std::string path, Dependency_Tree *tree);
//     };
// } // namespace ssc

#endif