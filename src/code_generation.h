#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "ast.h"
#include "parser.h"

#include <llvm/IR/Module.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/Verifier.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>

#include <llvm/Support/raw_ostream.h>

using namespace llvm;

enum Scope
{
    global,
    function,
};

static LLVMContext context;
static std::map<int, unique_ptr<Module>> modules;
static int current_module_pointer = 0;
static llvm::IRBuilder<> builder(context);

static std::map<std::string, Function_Node *> functions;
static std::string current_function_name;

static Scope scope;

static std::map<std::string, unique_ptr<Expression_Node>> global_variables_awaiting_initialization;
static std::string global_variable_assign_function_name = "__assign_global_variables";

static Variable_Type currently_preferred_type = Variable_Type::type_i32;
static std::map<std::string, Type *> object_types;
static std::map<std::string, std::map<std::string, Variable_Type>> object_type_properties;

unique_ptr<Module> code_gen_nodes(const Nodes &nodes);
static Value *code_gen_node(const unique_ptr<Node> &node);
static Value *load_if_ptr(Value *v);
static Value *code_gen_object_variable_declaration(Variable_Declaration_Node *var);
static void define_object_properties(Variable_Declaration_Node *var, Value *ptr, unique_ptr<Node> expr);
static Value *code_gen_primitive_variable_declaration(Variable_Declaration_Node *var);
static void print_v(Value *v);
static void print_t(Value *v);
static void print_current_module();

static Value *create_entry_block_alloca(Function *function, const std::string &name, Type *type);
static Type *variable_type_to_llvm_type(Variable_Type type, std::string object_type_name = "");
static void error(const char *arg);

#endif