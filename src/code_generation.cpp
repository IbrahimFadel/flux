#include "code_generation.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm-c/Target.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>

using namespace llvm;

static LLVMContext context;
static Module *TheModule = new Module("example", context);
static IRBuilder<> Builder(context);
static std::map<std::string, Value *> NamedValues;

void generate_llvm_ir(std::vector<Node *> nodes)
{
  LLVMInitializeNativeTarget();

  raw_ostream *os = &outs();

  for (auto &node : nodes)
  {
    Value *v;

    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      v = std::get<Constant_Declaration_Node>(node->constant_declaration_node).code_gen();
      break;
    }
    default:
      break;
    }
    v->print(*os, false);
  }

  // Value *num = ConstantFP::get(context, APFloat(2.5));
  // Value *sum = Builder.CreateAdd(num, num, "addtmp");
  // cout << "hi" << endl;
  // // num->dump();
  // raw_ostream *os = &llvm::outs();
  // num->print(*os, false);
  // outs() << "\n";
  // sum->print(*os, false);
}

llvm::Value *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    cout << expression << endl;
    // Expression_Node *expr = expression->get();
    // return ConstantFP::get(context, APFloat(2.5));
    // if (expr->type == Expression_Types::NumberExpression)
    // {
    // cout << std::get<Number_Expression_Node>(expr->number_expression).type;
    // }
    return ConstantFP::get(context, APFloat(2.5));
    break;
  }
  default:
    return ConstantFP::get(context, APFloat(2.5));
    break;
  }
}
