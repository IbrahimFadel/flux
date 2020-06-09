#include "code_generation.h"

#include <sstream>

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
static std::map<std::string, llvm::LoadInst *> LoadedVariables;

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
}

llvm::Value *Variable_Declaration_Node::code_gen()
{
  auto ai = new llvm::AllocaInst(llvm::Type::getInt32Ty(context), 0, "indexLoc");
  return ai;
}

Number evaluate_number_expression(Number_Expression_Node expr)
{
  Number num;

  std::vector<float> term_values;
  std::vector<float> term_values_values;
  float value = 0;
  bool is_int = true;
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
            if (is_number(term.numbers[i + 1]))
            {
              if (is_float(term.numbers[i + 1]))
              {
                is_int = false;
              }
              term_value = term_values[term_values.size() - 1] * std::stof(term.numbers[i + 1]);
            }
            else
            {
            }

            //TODO Check if it's a variable.
          }
          else
          {
            if (is_float(term.numbers[i]) || is_float(term.numbers[i + 1]))
            {
              is_int = false;
            }
            term_value = std::stof(term.numbers[i]) * std::stof(term.numbers[i + 1]);
          }

          term_values.push_back(term_value);
        }
        else if (op == "/")
        {
          if (term_values.size() > 0)
          {
            if (is_float(term.numbers[i + 1]))
            {
              is_int = false;
            }
            term_value = term_values[term_values.size() - 1] / std::stof(term.numbers[i + 1]);
          }
          else
          {
            if (is_float(term.numbers[i]) || is_float(term.numbers[i + 1]))
            {
              is_int = false;
            }
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
      if (is_float(term.numbers[0]))
      {
        is_int = false;
      }
      term_value = std::stof(term.numbers[0]);
      term_values_values.push_back(term_value);
    }
  }

  for (auto &val : term_values_values)
  {
    value += val;
  }

  if (is_int)
  {
    num.int_number = (int)value;
    num.type = Number_Types::IntNumber;
  }
  else
  {
    num.float_number = value;
    num.type = Number_Types::FloatNumber;
  }

  return num;
}

llvm::Value *Return_Node::code_gen()
{
  Expression_Node *expr = return_expression.get();
  return expr->code_gen();
}

llvm::Value *Number_Expression_Node::code_gen()
{
  Number num = evaluate_number_expression(*this);
  switch (num.type)
  {
  case Number_Types::IntNumber:
  {
    uint64_t v = (uint64_t)std::get<int>(num.int_number);
    auto integer = llvm::ConstantInt::get(context, llvm::APInt(32, v, true));
    return integer;
    break;
  }
  case Number_Types::FloatNumber:
    return llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
  default:
    break;
  }
}

llvm::Value *Expression_Node::code_gen()
{
  if (type == Expression_Types::NumberExpression)
  {
    return std::get<Number_Expression_Node>(number_expression).code_gen();
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

llvm::Type *get_llvm_variable_type(int type)
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    return llvm::Type::getFloatTy(context);
    break;
  }
  case Variable_Types::IntType:
  {
    return llvm::Type::getInt32Ty(context);
  }
  default:
    return nullptr;
  }
}

llvm::Value *create_bin_op_instruction(Variable_Types type, llvm::Value *LHS, llvm::Value *RHS, std::string name, std::string op)
{
  llvm::Value *BinOpInst;
  if (op == "*")
  {
    if (type == Variable_Types::FloatType)
    {
      BinOpInst = Builder.CreateFMul(LHS, RHS, name);
    }
    else
    {
      BinOpInst = Builder.CreateMul(LHS, RHS, name);
    }
  }
  else if (op == "+")
  {
    if (type == Variable_Types::FloatType)
    {
      BinOpInst = Builder.CreateFAdd(LHS, RHS, name);
    }
    else
    {
      BinOpInst = Builder.CreateAdd(LHS, RHS, name);
    }
  }

  return BinOpInst;
}

void function_variable_code_gen(Variable_Declaration_Node *variable_declaration_node, llvm::BasicBlock *BB)
{
  llvm::Type *variable_type = get_llvm_variable_type(variable_declaration_node->type);

  auto AI = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);

  Number_Expression_Node number_expression = std::get<Number_Expression_Node>(variable_declaration_node->expression->number_expression);

  std::vector<llvm::Value *> TermValues;
  bool single_number = false;

  for (auto &term : number_expression.terms)
  {
    if (term.ops.size() == 0)
    {
      single_number = true;
      llvm::Value *Value = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(term.numbers[0])));
      auto StorInst = new llvm::StoreInst(Value, AI, BB);
      auto LoadInst = new llvm::LoadInst(AI, variable_declaration_node->name + "_loaded", BB);
    }
    else
    {
      auto AllocateTerm = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);
      std::vector<llvm::LoadInst *> BinOpInsts;
      int i = 0;
      for (auto &op : term.ops)
      {
        if (i > 0)
        {
          llvm::Value *LHS = BinOpInsts[BinOpInsts.size() - 1];
          llvm::Value *RHS = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(term.numbers[i + 1])));

          auto BinOpInstAlloc = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);
          auto BinOpInst = create_bin_op_instruction(variable_declaration_node->type, LHS, RHS, variable_declaration_node->name, op);
          auto StoreInst = new llvm::StoreInst(BinOpInst, BinOpInstAlloc, BB);
          auto LoadInst = new llvm::LoadInst(BinOpInstAlloc, variable_declaration_node->name, BB);

          BinOpInsts.push_back(LoadInst);
        }
        else
        {
          llvm::Value *LHS = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(term.numbers[i])));
          llvm::Value *RHS = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(term.numbers[i + 1])));

          auto BinOpInstAlloc = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);
          auto BinOpInst = Builder.CreateFMul(LHS, RHS, variable_declaration_node->name);
          auto StoreInst = new llvm::StoreInst(BinOpInst, BinOpInstAlloc, BB);
          auto LoadInst = new llvm::LoadInst(BinOpInstAlloc, variable_declaration_node->name, BB);

          BinOpInsts.push_back(LoadInst);
        }

        if (i == term.ops.size() - 1)
        {
          TermValues.push_back(BinOpInsts[BinOpInsts.size() - 1]);
        }

        i++;
      }
    }
  }

  if (single_number)
  {
    return;
  }

  std::vector<llvm::LoadInst *> BinOpInsts;
  int i = 0;
  for (auto &op : number_expression.ops)
  {
    if (i > 0)
    {
      llvm::Value *LHS = BinOpInsts[BinOpInsts.size() - 1];
      llvm::Value *RHS = TermValues[i + 1];

      auto ALLOC = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);
      auto BINOP = create_bin_op_instruction(variable_declaration_node->type, LHS, RHS, variable_declaration_node->name, op);
      auto STORE = new llvm::StoreInst(BINOP, ALLOC, BB);
      auto LOAD = new llvm::LoadInst(ALLOC, variable_declaration_node->name, BB);
      BinOpInsts.push_back(LOAD);
    }
    else
    {
      llvm::Value *LHS = TermValues[i];
      llvm::Value *RHS = TermValues[i + 1];

      auto ALLOC = new llvm::AllocaInst(variable_type, NULL, variable_declaration_node->name, BB);
      auto BINOP = create_bin_op_instruction(variable_declaration_node->type, LHS, RHS, variable_declaration_node->name, op);
      auto STORE = new llvm::StoreInst(BINOP, ALLOC, BB);
      auto LOAD = new llvm::LoadInst(ALLOC, variable_declaration_node->name, BB);
      BinOpInsts.push_back(LOAD);
    }
    i++;
  }

  auto STORE = new llvm::StoreInst(BinOpInsts[BinOpInsts.size() - 1], AI, BB);
  llvm::Value *LOAD = new llvm::LoadInst(AI, variable_declaration_node->name + "_loaded", BB);
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
        llvm::Value *v = return_node->code_gen();
        Builder.CreateRet(v);
      }
      break;
    }
    case Node_Types::VariableDeclarationNode:
    {
      Variable_Declaration_Node *variable_declaration_node = std::get<Variable_Declaration_Node *>(node->variable_declaration_node);
      function_variable_code_gen(variable_declaration_node, BB);
      break;
    }
    default:
      break;
    }
  }

  llvm::verifyFunction(*TheFunction);

  return TheFunction;
}

llvm::Value *Then::code_gen()
{
  for (auto &node : nodes)
  {
  }
}

llvm::Value *Term::code_gen()
{
  std::vector<std::vector<std::string>> pairs;
  std::vector<std::string> pair;
  int i = 1;
  for (auto &num : numbers)
  {
    pair.push_back(num);
    if (i == numbers.size())
    {
      pairs.push_back(pair);
      pair.clear();
    }
    if (i % 2 == 0 && i != 0)
    {
      pairs.push_back(pair);
      pair.clear();
    }
    i++;
  }

  for (auto &bin_op : pairs)
  {
    llvm::Value *L = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(bin_op[0])));
    llvm::Value *R = llvm::ConstantFP::get(context, llvm::APFloat(std::stof(bin_op[1])));
    return Builder.CreateAdd(L, R, "addtmp");
  }
}

llvm::Constant *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {

    Number_Expression_Node node = std::get<Number_Expression_Node>(expression->number_expression);
    for (auto &term : node.terms)
    {
      llvm::Value *v = term.code_gen();
    }
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

bool is_float(std::string myString)
{
  int decimals_found = 0;
  for (auto &c : myString)
  {
    if (c == '.')
    {
      decimals_found++;
    }
    else if (!isdigit(c))
    {
      return false;
    }
  }

  return decimals_found == 1;
}