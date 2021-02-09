#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "options.h"
#include "parser.h"

#include <llvm/IR/Module.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>

#include <llvm/Bitcode/BitcodeWriter.h>

#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar.h>
#include <llvm/Transforms/Scalar/GVN.h>
#include <llvm/Transforms/Utils.h>

static CompilerOptions compiler_options;
static llvm::LLVMContext context;
static unique_ptr<llvm::Module> module;
static llvm::IRBuilder<> builder(context);
static unique_ptr<llvm::legacy::FunctionPassManager> fpm;

static std::map<std::string, Function_Declaration *> functions;
static std::string current_function_name;
static llvm::Type *currently_preferred_type = llvm::Type::getInt32Ty(context);

unique_ptr<llvm::Module> code_gen_nodes(const Nodes &nodes, CompilerOptions options);
static void code_gen_node(const unique_ptr<Node> &node);
static void initialize_fpm();
static void create_function_param_allocas(llvm::Function *f, std::map<std::string, std::string> params);
static llvm::Value *create_entry_block_alloca(llvm::Function *function, const std::string &name, llvm::Type *type);
static llvm::Type *ss_type_to_llvm_type(std::string type);
static llvm::Type *ss_base_type_to_llvm_type(std::string type);
static void print_t(llvm::Type *ty);
static void print_v(llvm::Value *v);
static void print_module();
static void fatal_error(std::string msg);

static llvm::Function *code_gen_function_prototype(std::map<std::string, std::string> params, std::string return_type, std::string function_name);

// unique_ptr<Module> code_gen_nodes(const Nodes &nodes, CompilerOptions compiler_options);

// #include "options.h"
// #include "ast.h"
// #include "parser.h"

// #include <llvm/IR/Module.h>
// #include <llvm/IR/IRBuilder.h>
// #include <llvm/IR/Value.h>
// #include <llvm/IR/Verifier.h>
// #include <llvm/IR/AssemblyAnnotationWriter.h>
// #include <llvm/IR/LegacyPassManager.h>
// #include <llvm/IR/LegacyPassManagers.h>
// #include <llvm/IR/LegacyPassNameParser.h>

// #include <llvm/Transforms/InstCombine/InstCombine.h>
// #include <llvm/Transforms/Scalar.h>
// #include <llvm/Transforms/Scalar/GVN.h>
// #include <llvm/Transforms/Utils.h>

// #include <llvm/Support/Host.h>
// #include <llvm/Support/raw_ostream.h>
// #include <llvm/Support/TargetSelect.h>
// #include <llvm/Support/TargetRegistry.h>

// #include <llvm/ExecutionEngine/ExecutionEngine.h>

// #include <llvm/Pass.h>

// using namespace llvm;

// enum Scope
// {
//     global,
//     function,
// };

// static LLVMContext context;
// static std::map<int, unique_ptr<Module>> modules;
// static int current_module_pointer = 0;
// static llvm::IRBuilder<> builder(context);
// static unique_ptr<llvm::legacy::FunctionPassManager> fpm;
// static CompilerOptions compiler_options;

// static std::map<std::string, Function_Node *> functions;
// static std::string current_function_name;

// static Scope scope;
// static bool wants_reference = false;

// static std::map<std::string, unique_ptr<Expression_Node>> global_variables_awaiting_initialization;
// static std::string global_variable_assign_function_name = "__assign_global_variables";

// static Variable_Type currently_preferred_type = Variable_Type::type_i32;
// static std::map<std::string, Type *> object_types;
// static std::map<std::string, std::map<std::string, Variable_Type>> object_type_properties;

// unique_ptr<Module> code_gen_nodes(const Nodes &nodes, CompilerOptions compiler_options);
// void module_to_obj(unique_ptr<llvm::Module> m);
// static Value *code_gen_node(const unique_ptr<Node> &node);
// static void initialize_function_pass_manager();
// static void declare_printf();
// static Value *load_if_ptr(Value *v);
// static void define_object_properties(Variable_Declaration_Node *var, Value *ptr, unique_ptr<Expression_Node> expr);
// static Value *code_gen_primitive_variable_declaration(Variable_Declaration_Node *var);
// static Value *code_gen_array_variable_declaration(Variable_Declaration_Node *var);
// static Value *code_gen_string_variable_declaration(Variable_Declaration_Node *var);
// static Value *initialize_string(String_Expression *);
// static void initialize_string_type();
// static void print_v(Value *v);
// static void print_t(Value *v);
// static void print_current_module();

// static bool is_reference_type(Variable_Type type);
// static Value *create_entry_block_alloca(Function *function, const std::string &name, Type *type);
// static Type *variable_type_to_llvm_type(Variable_Type type, std::string object_type_name = "");
// static Variable_Type llvm_type_to_variable_type(llvm::Type *type);
// static void error(const char *arg);

#endif