#include "code_generation.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm-c/Target.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/IR/Verifier.h>
#include <llvm/IR/PassManager.h>
#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Bitcode/BitcodeReader.h>

static llvm::LLVMContext context;
static llvm::Module *TheModule = new llvm::Module("Module", context);
static llvm::IRBuilder<> Builder(context);
static std::map<std::string, llvm::Value *> NamedValues;
// static std::map<std::string, llvm::AllocaInst *> NamedValues;

void generate_llvm_ir(std::vector<Node *> nodes)
{
  LLVMInitializeNativeTarget();

  llvm::raw_ostream *os = &llvm::outs();
  llvm::StringRef oname = "llvm_ir";
  std::error_code EC;
  llvm::raw_fd_ostream *out = new llvm::raw_fd_ostream(oname, EC);

  llvm::StringRef source_file_name("test.ss");
  TheModule->setSourceFileName(source_file_name);

  for (auto &node : nodes)
  {
    llvm::Constant *constant;
    llvm::Value *variable;
    llvm::Function *prototype;
    llvm::Function *function_body;

    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      constant = std::get<Constant_Declaration_Node *>(node->constant_declaration_node)->code_gen();
      constant->print(*os);
      cout << endl;
      constant->print(*out);
      out->write('\n');

      break;
    }
    case Node_Types::VariableDeclarationNode:
    {
      variable = std::get<Variable_Declaration_Node *>(node->variable_declaration_node)->code_gen();
      variable->print(*os);
      cout << endl;
      variable->print(*out);
      out->write('\n');
      break;
    }
    case Node_Types::FunctionDeclarationNode:
    {
      prototype = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_prototype();
      function_body = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_function_body(prototype);
      prototype->print(*os);
      cout << endl;
      prototype->print(*out);
      out->write('\n');
      break;
    }
    default:
      break;
    }
  }

  std::error_code My_EC;
  llvm::raw_fd_ostream bc_out("module", EC, llvm::sys::fs::F_None);
  llvm::WriteBitcodeToFile(*TheModule, bc_out);

  bc_out.flush();
}

// static llvm::AllocaInst *CreateEntryBlockAlloca(llvm::Function *TheFunction, const std::string &VarName)
// {
//   llvm::IRBuilder<> TmpB(&TheFunction->getEntryBlock(), TheFunction->getEntryBlock().begin());
//   return TmpB.CreateAlloca(llvm::Type::getDoubleTy(context), 0, VarName.c_str());
// }

llvm::Value *Variable_Declaration_Node::code_gen()
{
  auto *ai = new llvm::AllocaInst(llvm::Type::getInt32Ty(context), 0, "indexLoc");
  return ai;
  // auto *L = llvm::ConstantInt::get(llvm::Type::getInt32Ty(context), 41);
  // auto *R = llvm::ConstantInt::get(llvm::Type::getInt32Ty(context), 42);
  // Builder.Insert(L);
  // Builder.Insert(R);
  // return Builder.CreateAdd(L, R, "addtmp");
  // switch (type)
  // {
  // case Variable_Types::FloatType:
  // {
  //   Number num = evaluate_number_expression(std::get<Number_Expression_Node>(expression->number_expression));
  //   // llvm::Value *test = llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
  //   // llvm::Value *test = llvm::V
  //   // return test;
  //   return llvm::ConstantFP::get(context, llvm::APFloat(1.0));
  //   break;
  // }
  // default:
  //   return llvm::ConstantFP::get(context, llvm::APFloat(2.5));
  //   break;
  // }
}

Number evaluate_number_expression(Number_Expression_Node expr)
{
  Number num;
  num.type = Number_Types::FloatNumber;

  std::vector<float> term_values;
  std::vector<float> term_values_values;
  float value = 0;
  for (auto &term : expr.terms)
  {
    float term_value = 0;
    int i = 0;
    if (term.ops.size() > 0)
    {
      for (auto &op : term.ops)
      {
        if (op == "*")
        {
          if (term_values.size() > 0)
          {
            term_value = term_values[term_values.size() - 1] * std::stof(term.numbers[i + 1]);
          }
          else
          {
            term_value = std::stof(term.numbers[i]) * std::stof(term.numbers[i + 1]);
          }

          term_values.push_back(term_value);
        }
        else if (op == "/")
        {
          if (term_values.size() > 0)
          {
            term_value = term_values[term_values.size() - 1] / std::stof(term.numbers[i + 1]);
          }
          else
          {
            term_value = std::stof(term.numbers[i]) / std::stof(term.numbers[i + 1]);
          }

          term_values.push_back(term_value);
        }
        i++;
      }
      term_values_values.push_back(term_values[term_values.size() - 1]);
      term_values.clear();
    }
    else
    {
      term_value = std::stof(term.numbers[0]);
      term_values_values.push_back(term_value);
    }
  }

  for (auto &val : term_values_values)
  {
    value += val;
  }

  num.float_number = value;

  return num;
}

llvm::Value *Return_Node::code_gen()
{
  if (return_expression->type == Expression_Types::NumberExpression)
  {
  }
}

llvm::Value *Expression_Node::code_gen()
{
  if (type == Expression_Types::NumberExpression)
  {
    Number num = evaluate_number_expression(std::get<Number_Expression_Node>(number_expression));
    switch (num.type)
    {
    case Number_Types::FloatNumber:
      return llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
    default:
      break;
    }
  }
  else if (type == Expression_Types::StringExpression)
  {
  }
}

llvm::Function *Function_Declaration_Node::code_gen_prototype()
{
  // std::vector<llvm::Type *> Doubles(parameters.size(), llvm::Type::getDoubleTy(context));
  std::vector<llvm::Type *> params;
  for (auto &param : parameters)
  {
    switch (param.second)
    {
    case Variable_Types::FloatType:
      params.push_back(llvm::Type::getFloatTy(context));
      break;
    case Variable_Types::IntType:
      params.push_back(llvm::Type::getInt32Ty(context));
    default:
      break;
    }
  }

  llvm::Type *function_return_type = llvm::Type::getFloatTy(context);
  switch (return_type)
  {
  case Variable_Types::FloatType:
    function_return_type = llvm::Type::getFloatTy(context);
    break;
  case Variable_Types::IntType:
    function_return_type = llvm::Type::getInt32Ty(context);
  default:
    break;
  }
  llvm::FunctionType *FT = llvm::FunctionType::get(function_return_type, params, false);

  llvm::Function *F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, name, TheModule);

  unsigned Idx = 0;
  std::map<std::string, Variable_Types>::iterator it;
  std::vector<std::string> names;
  for (it = parameters.begin(); it != parameters.end(); it++)
  {
    names.push_back(it->first);
  }
  for (auto &Arg : F->args())
  {
    Arg.setName(names[Idx]);
    Idx++;
  }

  return F;
}

llvm::Function *Function_Declaration_Node::code_gen_function_body(llvm::Function *proto)
{
  llvm::Function *TheFunction = TheModule->getFunction(proto->getName());

  if (!TheFunction)
    TheFunction = code_gen_prototype();
  if (!TheFunction)
    return nullptr;
  if (!TheFunction->empty())
  {
    std::cerr << "Error" << std::endl;
    return nullptr;
  }

  llvm::BasicBlock *BB = llvm::BasicBlock::Create(context, "entry", TheFunction);
  Builder.SetInsertPoint(BB);

  NamedValues.clear();
  for (auto &Arg : TheFunction->args())
  {
    NamedValues[Arg.getName()] = &Arg;
  }

  for (auto &node : then.nodes)
  {
    switch (node->type)
    {
    case Node_Types::ReturnNode:
    {
      if (std::get<Return_Node *>(node->return_node)->return_expression->type == Expression_Types::NumberExpression)
      {
        Return_Node *return_node = std::get<Return_Node *>(node->return_node);
        Expression_Node *expr = return_node->return_expression.get();

        llvm::Value *v = expr->code_gen();
        Builder.CreateRet(v);
      }
      break;
    }
    case Node_Types::VariableDeclarationNode:
    {
      // llvm::Value *v = std::get<Variable_Declaration_Node *>(node->variable_declaration_node)->code_gen();
      // llvm::Instruction *pi = llvm::Instruction(llvm::Type::getInt32Ty(context));
      // Builder.
      // BB->
      // Builder.Insert(v);
      break;
    }
    default:
      break;
    }
  }

  llvm::verifyFunction(*TheFunction);

  return TheFunction;
}

llvm::Constant *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    Number num = evaluate_number_expression(std::get<Number_Expression_Node>(expression->number_expression));
    llvm::Constant *test = llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
    return test;
    return llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
    break;
  }
  default:
    return llvm::ConstantFP::get(context, llvm::APFloat(2.5));
    break;
  }
}

float evaluate_float_expression(std::unique_ptr<Expression_Node> expression)
{
  return 1.0;
}