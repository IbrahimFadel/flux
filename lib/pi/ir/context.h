#ifndef SSC_IR_CONTEXT_H
#define SSC_IR_CONTEXT_H

#include <memory>
#include <string>
#include <map>

namespace ssc
{
    class CodegenContext;
}

#include "ast/nodes.h"
#include "driver/options.h"
#include "driver/dependencies.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Value.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/AssemblyAnnotationWriter.h"
#include "llvm/IR/Verifier.h"

#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar.h>
#include <llvm/Transforms/Scalar/GVN.h>
#include <llvm/Transforms/Utils.h>

using std::unique_ptr;

namespace ssc
{
    class CodegenContext
    {
    private:
        llvm::LLVMContext ctx;
        llvm::Module *mod;
        llvm::IRBuilder<> builder;
        fs::path basePath;
        unique_ptr<DependencyGraph> &dependencyGraph;
        unique_ptr<llvm::legacy::FunctionPassManager> fpm;
        llvm::DataLayout *dataLayout;

        llvm::Function *mallocFunction;
        llvm::Function *freeFunction;

        std::string currentFunctionName;
        std::map<std::string, ASTFunctionDefinition *> functions;
        std::string currentlyPreferredType;
        unique_ptr<Options> &compilerOptions;
        std::map<std::string, llvm::StructType *> structTypes;
        std::map<std::string, ASTClassDeclaration *> astClassTypes;
        std::map<std::string, std::vector<std::string>> classProperties;
        std::map<std::string, std::vector<std::string>> classMethods;

    public:
        CodegenContext(fs::path moduleName, unique_ptr<Options> &compilerOptions, unique_ptr<DependencyGraph> &depGraph) : builder(ctx), compilerOptions(compilerOptions), dependencyGraph(depGraph)
        {
            mod = new llvm::Module(moduleName.string(), ctx);
            basePath = moduleName.parent_path();
            dataLayout = new llvm::DataLayout(mod);
        };
        void error(std::string msg);
        void warning(std::string msg);
        void printModule()
        {
            auto writer = new llvm::AssemblyAnnotationWriter();
            mod->print(llvm::outs(), writer);
        }

        template <typename T>
        void print(T v)
        {
            v->print(llvm::outs());
            llvm::outs() << '\n';
        }

        void initializeFPM();
        void runFPM(llvm::Function *f);
        void declareFunction(ASTFunctionDefinition *fn);

        llvm::Type *ssTypeToLLVMType(std::string type);
        llvm::Type *ssBaseTypeToLLVMType(std::string type);
        bool isTypeSigned(std::string type);
        llvm::Value *implicityTypecastExpression(llvm::Value *v, std::string currentType, llvm::Type *targetType);
        void defineCFunctions();

        void setCurrentlyPreferredType(std::string ty) { currentlyPreferredType = ty; }
        std::string getCurrentlyPreferredType() { return currentlyPreferredType; }
        void setCurrentFunctionName(std::string name) { currentFunctionName = name; }
        std::string getCurrentFunctionName() { return currentFunctionName; }
        void setFunction(std::string name, ASTFunctionDefinition *fn) { functions[name] = fn; }
        ASTFunctionDefinition *getFunction(std::string name) { return functions[name]; }
        llvm::Module *getModule() { return mod; }
        llvm::IRBuilder<> *getBuilder() { return &builder; }
        llvm::LLVMContext *getCtx() { return &ctx; }
        unique_ptr<Options> &getCompilerOptions() { return compilerOptions; }
        void setStructType(std::string name, llvm::StructType *structType) { structTypes[name] = structType; }
        llvm::StructType *getStructType(std::string name) { return structTypes[name]; }
        void setClassProperties(std::string name, std::vector<std::string> props) { classProperties[name] = props; }
        std::vector<std::string> getClassProperties(std::string name) { return classProperties[name]; }
        void setClassMethods(std::string name, std::vector<std::string> methods) { classMethods[name] = methods; };
        std::vector<std::string> getClassMethods(std::string name) { return classMethods[name]; };
        void setASTClassType(std::string name, ASTClassDeclaration *c) { astClassTypes[name] = c; }
        ASTClassDeclaration *getASTClassType(std::string name) { return astClassTypes[name]; }
        llvm::Function *getMallocFunction() { return mallocFunction; }
        llvm::DataLayout *getDataLayout() { return dataLayout; }
        bool isClassType(std::string name);
        fs::path getBasePath() { return basePath; }
        unique_ptr<DependencyGraph> &getDependencyGraph() { return dependencyGraph; }
    };
} // namespace ssc

#endif