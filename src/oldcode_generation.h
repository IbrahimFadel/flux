#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "common.h"
#include "parser.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Verifier.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/LegacyPassManagers.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>

#include <llvm/Support/TargetRegistry.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/Host.h>
#include <llvm/Support/FileSystem.h>

#include <llvm/Target/TargetOptions.h>
#include <llvm/Target/TargetMachine.h>

#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Bitcode/BitcodeReader.h>

#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar.h>
#include <llvm/Transforms/Scalar/GVN.h>

enum Scopes
{
    global,
    function
};

static llvm::LLVMContext context;
static std::unique_ptr<llvm::Module> module;
static llvm::IRBuilder<> builder(context);
static std::unique_ptr<llvm::legacy::FunctionPassManager> function_pass_manager;
static std::vector<std::shared_ptr<llvm::Module>> modules;
static std::vector<std::string> module_names;
static int current_module = 0;
static std::string project_root = "/home/ibrahim/dev/sandscript";
static std::string build_dir = "/home/ibrahim/dev/sandscript/ssbuild";
static bool defined_putchar = false;

static std::map<std::string, llvm::Value *> global_variables;
static std::map<std::string, std::map<std::string, llvm::Value *>> function_variables;
static std::string current_function;

static std::map<std::string, Function_Node *> functions;
static Scopes scope = Scopes::global;

static std::map<std::string, llvm::StructType *> objects;

static std::map<std::string, std::unique_ptr<Expression_Node>> global_variables_awaiting_initialization;
static llvm::Value *construct_global_variable_assign_function();
static std::string global_variable_assign_function_name = "__assign_global_variables";

void module_to_obj(std::shared_ptr<llvm::Module> module, std::string path);
std::shared_ptr<llvm::Module> code_gen_nodes(std::vector<std::unique_ptr<Node>> nodes);
void code_gen_node(std::unique_ptr<Node> node);
void initialize_fpm();
static void define_object_properties(llvm::Value *ptr, std::unique_ptr<Expression_Node> expr);
static llvm::Type *ss_type_to_llvm_type(Variable_Types type);
static llvm::Type *get_object_type(std::string object_type_name);
static llvm::Type *variable_type_to_llvm_ptr_type(Variable_Types type);
static llvm::AllocaInst *create_entry_block_alloca(llvm::Function *TheFunction,
                                                   const std::string &VarName, llvm::Type *type);
static llvm::Type *bitwidth_to_llvm_type(unsigned int bitwidth);
static llvm::Constant *get_zeroed_variable(llvm::Type *type);
static llvm::Value *get_ptr_or_value_with_type(llvm::Value *val, Variable_Types type);
static void define_putchar();

static llvm::Value *error_v(const char *Str);
static void print(llvm::Value *v);
static void print_ty(llvm::Type *ty);

#endif