#include "codegen.h"

using namespace ssc;

void CodeGenerator::codegenNodes(Nodes _nodes)
{
    nodes = std::move(_nodes);
}

// using namespace ssc;

// CodeGenerator::CodeGenerator(Nodes nodes, ssc::Options *compilerOptions, std::vector<std::string> structTypeNames) : nodes(std::move(nodes)), compilerOptions(compilerOptions), structTypeNames(structTypeNames)
// {
//     module = std::make_unique<llvm::Module>(compilerOptions->getInputFilePath(), context);
//     builder = std::make_unique<llvm::IRBuilder<>>(context);
//     currentlyPreferredType = llvm::Type::getInt32Ty(context);
// }

// void CodeGenerator::moduleToObj(llvm::Module *mod, std::string output_path)
// {
//     auto target_triple = llvm::sys::getDefaultTargetTriple();

//     llvm::InitializeAllTargetInfos();
//     llvm::InitializeAllTargets();
//     llvm::InitializeAllTargetMCs();
//     llvm::InitializeAllAsmParsers();
//     llvm::InitializeAllAsmPrinters();

//     std::string error;
//     auto target = llvm::TargetRegistry::lookupTarget(target_triple, error);
//     if (!target)
//     {
//         llvm::errs() << error;
//         exit(1);
//     }

//     auto CPU = "generic";
//     auto features = "";

//     llvm::TargetOptions opt;
//     auto rm = llvm::Optional<llvm::Reloc::Model>();
//     auto target_machine = target->createTargetMachine(target_triple, CPU, features, opt, rm);

//     mod->setDataLayout(target_machine->createDataLayout());
//     mod->setTargetTriple(target_triple);

//     std::error_code ec;
//     llvm::raw_fd_ostream dest(output_path, ec, llvm::sys::fs::F_None);
//     if (ec)
//     {
//         llvm::errs() << "Could not open file: " << ec.message();
//         exit(1);
//     }

//     llvm::legacy::PassManager pass;
//     auto file_type = llvm::CGFT_ObjectFile;

//     if (target_machine->addPassesToEmitFile(pass, dest, nullptr, file_type))
//     {
//         llvm::errs() << "Target Machine cannot emit a file of this type";
//         exit(1);
//     }

//     pass.run(*mod);
//     dest.flush();
// }

// void CodeGenerator::declareCFunctions(llvm::Module *mod)
// {
//     std::vector<llvm::Type *> free_param_types;
//     free_param_types.push_back(llvm::Type::getInt8PtrTy(context));
//     auto free_return_type = llvm::Type::getVoidTy(context);
//     declareFunction("free", free_param_types, free_return_type, mod);

//     std::vector<llvm::Type *> malloc_param_types;
//     malloc_param_types.push_back(llvm::Type::getInt64Ty(context));
//     auto malloc_return_type = llvm::Type::getInt8PtrTy(context);
//     declareFunction("malloc", malloc_param_types, malloc_return_type, mod);

//     std::vector<llvm::Type *> memcpy_param_types;
//     memcpy_param_types.push_back(llvm::Type::getInt8PtrTy(context));
//     memcpy_param_types.push_back(llvm::Type::getInt8PtrTy(context));
//     memcpy_param_types.push_back(llvm::Type::getInt64Ty(context));
//     auto memcpy_return_type = llvm::Type::getInt8PtrTy(context);
//     declareFunction("memcpy", memcpy_param_types, memcpy_return_type, mod);

//     std::vector<llvm::Type *> printf_param_types;
//     printf_param_types.push_back(llvm::Type::getInt8PtrTy(context));
//     auto printf_return_type = llvm::Type::getInt32Ty(context);
//     declareFunction("printf", printf_param_types, printf_return_type, mod);
// }

// void CodeGenerator::declareStringFunctions(llvm::Module *mod)
// {
//     std::vector<llvm::Type *> string_property_types = {llvm::Type::getInt8PtrTy(context), llvm::Type::getInt64Ty(context), llvm::Type::getInt64Ty(context), llvm::Type::getInt64Ty(context)};
//     std::map<std::string, std::string> string_properties = {
//         {"buffer", "i8*"},
//         {"length", "i64"},
//         {"maxLength", "i64"},
//         {"factor", "i64"}};
//     std::vector<std::string> string_property_insetion_order = {"buffer, length, maxLength, factor"};
//     declareStructType(string_property_types, string_properties, string_property_insetion_order, "string");

//     std::vector<llvm::Type *> string_create_default_param_types = {llvmStructTypes["string"]->getPointerTo()};
//     auto string_create_default_return_type = llvm::Type::getVoidTy(context);
//     declareFunction("string_create_default", string_create_default_param_types, string_create_default_return_type, mod);

//     std::vector<llvm::Type *> string_add_char_param_types = {llvmStructTypes["string"]->getPointerTo(), llvm::Type::getInt8Ty(context)};
//     auto string_add_char_return_type = llvm::Type::getVoidTy(context);
//     declareFunction("string_add_char", string_add_char_param_types, string_add_char_return_type, mod);

//     std::vector<llvm::Type *> string_delete_param_types = {llvmStructTypes["string"]->getPointerTo()};
//     auto string_delete_return_type = llvm::Type::getVoidTy(context);
//     declareFunction("string_delete", string_delete_param_types, string_delete_return_type, mod);
// }

// // void CodeGenerator::declareImportedFunctions(Dependency_Tree *tree, fs::path path, llvm::Module *mod)
// // {
// //     for (auto &connection : tree->connections)
// //     {
// //         auto left_path = tree->nodes[connection.first].first;
// //         if (left_path == path)
// //         {
// //             auto funcs_to_declare = tree->nodes[connection.second].second;
// //             for (auto &func : funcs_to_declare)
// //             {
// //                 std::vector<llvm::Type *> param_types;
// //                 for (auto &param_type : func->param_types)
// //                 {
// //                     param_types.push_back(ssTypeToLLVMType(param_type));
// //                 }
// //                 declareFunction(func->name, param_types, ssTypeToLLVMType(func->return_type), mod);
// //             }
// //         }
// //     }
// // }

// void CodeGenerator::codegenNode(const unique_ptr<Node> &node)
// {
//     node->codegen();
// }

// void CodeGenerator::initializeFPM(llvm::Module *mod)
// {
//     fpm = std::make_unique<llvm::legacy::FunctionPassManager>(mod);
//     fpm->add(llvm::createInstructionCombiningPass());
//     fpm->add(llvm::createReassociatePass());
//     fpm->add(llvm::createDeadCodeEliminationPass());
//     fpm->add(llvm::createGVNPass());
//     fpm->add(llvm::createCFGSimplificationPass());
//     fpm->add(llvm::createPromoteMemoryToRegisterPass());

//     fpm->doInitialization();
// }

// llvm::Value *CodeGenerator::codegenFunctionDeclaration(unique_ptr<FunctionDeclaration> fnDeclaration)
// {
//     auto name = fnDeclaration->getName();
//     auto params = fnDeclaration->getParams();
//     auto returnType = fnDeclaration->getReturnType();
//     llvm::Function *f = codeGenFunctionPrototype(params, returnType, name);
//     if (f == 0)
//         return 0;

//     auto entry = llvm::BasicBlock::Create(context, "entry", f);
//     builder->SetInsertPoint(entry);

//     currentFunctionName = name;
//     functions[currentFunctionName] = fnDeclaration.get();

//     createFunctionParamAllocas(f, params);

//     currentlyPreferredType = ssTypeToLLVMType(returnType);

//     fnDeclaration->getThen()->codegen();

//     // free_strings();

//     auto last_bb = builder->GetInsertBlock();
//     if (!last_bb->getTerminator())
//     {
//         builder->CreateRetVoid();
//     }

//     llvm::verifyFunction(*f, &llvm::outs());

//     if (compilerOptions->getOptimize())
//         fpm->run(*f);

//     return 0;
// }

// // void CodeGenerator::FunctionDeclaration::free_strings()
// // {
// //     auto f = module->getFunction("string_delete");
// //     for (auto it = variables.begin(), end = variables.end(); it != end; it++)
// //     {
// //         auto var = it->second;
// //         auto ptr = llvm::getPointerOperand(var);
// //         printV(var);
// //         printV(ptr);
// //         auto ty = var->getType();
// //         while (ty->isPointerTy())
// //             ty = ty->getPointerElementType();
// //         if (ty == llvmStructTypes["string"])
// //         {
// //             std::vector<llvm::Value *> params = {ptr};
// //             builder->CreateCall(f, params);
// //         }
// //     }
// // }

// llvm::Function *CodeGenerator::codeGenFunctionPrototype(std::map<std::string, std::string> params, std::string return_type_str, std::string function_name)
// {
//     std::vector<std::string> param_names;
//     std::vector<llvm::Type *> param_types;
//     auto it = params.begin();
//     while (it != params.end())
//     {
//         param_names.push_back(it->first);
//         auto type = ssTypeToLLVMType(it->second);
//         param_types.push_back(type);
//         it++;
//     }

//     auto return_type = ssTypeToLLVMType(return_type_str);
//     llvm::FunctionType *function_type = llvm::FunctionType::get(return_type, param_types, false);
//     llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, function_name);

//     if (f->getName() != function_name)
//     {
//         f->eraseFromParent();
//         f = module->getFunction(function_name);

//         if (!f->empty())
//             fatalError("Redefinition of function " + function_name);

//         if (f->arg_size() != params.size())
//             fatalError("Redefinition of function with different number of arguments");
//     }

//     unsigned idx = 0;
//     for (llvm::Function::arg_iterator ai = f->arg_begin(); idx != params.size(); ai++, idx++)
//     {
//         ai->setName(param_names[idx]);
//     }

//     return f;
// }

// llvm::Value *CodeGenerator::codegenNumberExpression(unique_ptr<NumberExpression> numExpr)
// {
//     return llvm::ConstantInt::get(currentlyPreferredType, numExpr->getValue());
// }

// llvm::Value *CodeGenerator::codegenStringLiteralExpression(unique_ptr<StringLiteralExpression> stringLitExpr)
// {
//     auto str = builder->CreateGlobalString(stringLitExpr->getValue());
//     return str;
// }

// llvm::Value *CodeGenerator::codegenVariableReferenceExpression(unique_ptr<VariableReferenceExpression> varRefExpr)
// {
//     return functions[currentFunctionName]->getVariable(varRefExpr->getName());
// }

// llvm::Value *CodeGenerator::codegenIndexAccessedExpression(unique_ptr<IndexAccessedExpression> indexAccessExpr)
// {
//     auto var = indexAccessExpr->getExpression()->codegen();
//     auto index_v = indexAccessExpr->getIndex()->codegen();
//     auto gep = builder->CreateGEP(var, index_v);
//     auto loaded = builder->CreateLoad(gep);
//     return loaded;
// }

// llvm::Value *CodeGenerator::codeGenBinopSumDiffProdQuot(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs)
// {
//     auto l = lhs->codegen();
//     auto r = rhs->codegen();
//     auto sum = builder->CreateAdd(l, r);
//     return sum;
// }

// llvm::Value *CodeGenerator::codeGenBinopEq(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs)
// {
//     auto l = lhs->codegen();
//     currentlyPreferredType = l->getType();
//     auto r = rhs->codegen();
//     auto l_ptr = llvm::getPointerOperand(l);
//     auto store = builder->CreateStore(r, l_ptr);
//     return store;
// }

// llvm::Value *CodeGenerator::codeGenBinopArrow(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs)
// {
//     auto lhs_var_reference_expr = dynamic_cast<VariableReferenceExpression *>(lhs.get());
//     if (lhs_var_reference_expr == nullptr)
//         fatalError("Object property assignments must have a variable reference on the left hand side");

//     auto rhs_var_reference_expr = dynamic_cast<VariableReferenceExpression *>(rhs.get());
//     if (rhs_var_reference_expr == nullptr)
//         fatalError("Object property assignments must have a variable reference on the right hand side");

//     auto prop_name = rhs_var_reference_expr->getName();

//     auto l_var = lhs_var_reference_expr->codegen();
//     auto ty = l_var->getType();
//     std::string struct_name;
//     while (ty->isPointerTy())
//         ty = ty->getPointerElementType();
//     struct_name = ty->getStructName().str();

//     int i = 0;
//     int prop_index = -1;
//     auto property_insertion_order = structPropertyInsertionOrders[struct_name];
//     auto properties = structProperties[struct_name];
//     for (auto &prop : property_insertion_order)
//     {
//         if (prop == prop_name)
//             prop_index = i;
//         i++;
//     }

//     if (prop_index == -1)
//         fatalError("Could not find property in struct");
//     auto gep = builder->CreateStructGEP(l_var, prop_index, prop_name);
//     auto loaded = builder->CreateLoad(gep);
//     return loaded;
// }

// llvm::Value *CodeGenerator::codeGenBinopPeriod(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs)
// {
//     auto lhs_var_reference_expr = dynamic_cast<VariableReferenceExpression *>(lhs.get());
//     if (lhs_var_reference_expr == nullptr)
//         fatalError("Object property assignments must have a variable reference on the left hand side");

//     auto rhs_var_reference_expr = dynamic_cast<VariableReferenceExpression *>(rhs.get());
//     if (rhs_var_reference_expr == nullptr)
//         fatalError("Object property assignments must have a variable reference on the right hand side");

//     auto prop_name = rhs_var_reference_expr->getName();

//     auto l_var = lhs_var_reference_expr->codegen();
//     auto ty = l_var->getType();
//     std::string struct_name;
//     while (ty->isPointerTy())
//         ty = ty->getPointerElementType();
//     struct_name = ty->getStructName().str();

//     int i = 0;
//     int prop_index = -1;
//     auto property_insertion_order = structPropertyInsertionOrders[struct_name];
//     auto properties = structProperties[struct_name];
//     for (auto &prop : property_insertion_order)
//     {
//         if (prop == prop_name)
//             prop_index = i;
//         i++;
//     }

//     if (prop_index == -1)
//         fatalError("Could not find property in struct");

//     auto l_var_ptr = llvm::getPointerOperand(l_var);
//     auto gep = builder->CreateStructGEP(l_var_ptr, prop_index, prop_name);
//     auto loaded = builder->CreateLoad(gep);
//     return loaded;
// }

// llvm::Value *CodeGenerator::codeGenBinopCmp(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, TokenType op)
// {
//     auto l = lhs->codegen();
//     auto r = rhs->codegen();

//     switch (op)
//     {
//     case TokenType::tok_compare_eq:
//         return builder->CreateICmpEQ(l, r);
//     case TokenType::tok_compare_ne:
//         return builder->CreateICmpNE(l, r);
//     case TokenType::tok_compare_lt:
//         return builder->CreateICmpULT(l, r);
//     case TokenType::tok_compare_gt:
//         return builder->CreateICmpUGT(l, r);
//     default:
//         break;
//     }
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenBinaryOperationExpression(unique_ptr<BinaryOperationExpression> binopExpr)
// {
//     // auto lhs = binopExpr->getLHS(); //TODO getLHS returns * not unique -- change this
//     // auto rhs = binopExpr->getRHS();
//     // auto op = binopExpr->getOp();
//     // switch (op)
//     // {
//     // case TokenType::tok_plus:
//     //     return codeGenBinopSumDiffProdQuot(lhs, rhs);
//     // case TokenType::tok_minus:
//     //     return codeGenBinopSumDiffProdQuot(lhs, rhs);
//     // case TokenType::tok_asterisk:
//     //     return codeGenBinopSumDiffProdQuot(lhs, rhs);
//     // case TokenType::tok_slash:
//     //     return codeGenBinopSumDiffProdQuot(std::move(lhs), std::move(rhs));
//     // case TokenType::tok_eq:
//     //     return codeGenBinopEq(lhs, rhs);
//     // case TokenType::tok_arrow:
//     //     return codeGenBinopArrow(lhs, rhs);
//     // case TokenType::tok_period:
//     //     return codeGenBinopPeriod(lhs, rhs);
//     // case TokenType::tok_compare_eq:
//     //     return codeGenBinopCmp(lhs, rhs, op);
//     // case TokenType::tok_compare_ne:
//     //     return codeGenBinopCmp(lhs, rhs, op);
//     // default:
//     //     fatalError("Tried to codegen binary operation expression of unimplemented operator");
//     // }
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenNullptrExpression(unique_ptr<NullptrExpression> nullptrExpr)
// {
//     while (currentlyPreferredType->isPointerTy())
//     {
//         currentlyPreferredType = currentlyPreferredType->getPointerElementType();
//     }
//     return llvm::ConstantPointerNull::get(currentlyPreferredType->getPointerTo());
// }

// llvm::Value *CodeGenerator::codegenUnaryPrefixOperationExpression(unique_ptr<UnaryPrefixOperationExpression> unaPrefixOpExpr)
// {
//     auto value = unaPrefixOpExpr->getValue();
//     auto op = unaPrefixOpExpr->getOp();
//     auto val = value->codegen();
//     if (op == TokenType::tok_ampersand)
//     {
//         return llvm::getPointerOperand(val);
//     }
//     else if (op == TokenType::tok_asterisk)
//     {
//         return builder->CreateLoad(val);
//     }
//     fatalError("Tried to codegen unary prefix opepration expression that is unimplemented");
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenCodeBlock(unique_ptr<CodeBlock> codeBlock)
// {
//     auto nodes = codeBlock->getNodes();
//     for (auto &node : nodes)
//     {
//         node->codegen();
//     }
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenVariableDeclaration(unique_ptr<VariableDeclaration> varDec)
// {
//     auto isStruct = varDec->getIsStruct();
//     auto value = varDec->getValue();
//     auto name = varDec->getName();
//     auto type = varDec->getType();
//     if (isStruct)
//     {
//         auto struct_val_expr = dynamic_cast<StructValueExpression *>(value);
//         if (struct_val_expr != nullptr)
//         {
//             return codeGenStructVariableDeclaration(name, type, struct_val_expr);
//         }
//     }
//     else if (type == "string")
//     {
//         return codeGenStringVariableDeclaration(name, value);
//     }

//     auto llvm_type = ssTypeToLLVMType(type);
//     auto ptr = builder->CreateAlloca(llvm_type, 0, name);

//     if (value != nullptr)
//     {
//         currentlyPreferredType = llvm_type;
//         auto val = value->codegen();
//         auto store = builder->CreateStore(val, ptr);
//     }

//     auto loaded = builder->CreateLoad(ptr, 0, name);
//     functions[currentFunctionName]->setVariable(name, std::move(loaded));
//     return loaded;
// }

// llvm::Value *CodeGenerator::codeGenStringVariableDeclaration(std::string name, Expression *value)
// {
//     auto ptr = builder->CreateAlloca(llvmStructTypes["string"]);

//     if (value != nullptr)
//     {
//         // String literals not implemented yet so... this is TODO
//         auto val = value->codegen();
//         auto glob_var = llvm::dyn_cast<llvm::GlobalVariable>(val);
//         if (glob_var)
//         {
//             auto const_data_arr = llvm::dyn_cast<llvm::ConstantDataArray>(glob_var->getInitializer());
//             if (const_data_arr)
//             {
//                 //? This means the value is a string literal, copy the chars into the string struct
//                 auto str_val = const_data_arr->getAsCString();
//                 cout << str_val.str() << endl;
//                 for (auto &c : str_val)
//                 {
//                     auto f = module->getFunction("string_add_char");
//                     std::vector<llvm::Value *> params = {ptr, llvm::ConstantInt::get(llvm::Type::getInt8Ty(context), llvm::APInt(8, c))};
//                     builder->CreateCall(f, params);
//                 }
//             }
//         }
//         // builder->CreateStore(val, ptr);
//     }
//     else
//     {
//         std::vector<llvm::Value *> params = {ptr};
//         auto string_init_fn = module->getFunction("string_create_default");
//         auto init_string = builder->CreateCall(string_init_fn, params);
//     }

//     auto loaded = builder->CreateLoad(ptr);
//     functions[currentFunctionName]->setVariable(name, std::move(loaded));
//     return loaded;
// }

// llvm::Value *CodeGenerator::codeGenStructVariableDeclaration(std::string name, std::string type, StructValueExpression *value)
// {
//     auto ptr = builder->CreateAlloca(llvmStructTypes[type]);
//     auto props = value->getProperties();
//     int i = 0;
//     for (auto &prop_name : value->getPropertyInsertionOrder())
//     {
//         auto struct_property_ptr = builder->CreateStructGEP(ptr, i, prop_name);
//         auto prop_ty = ssTypeToLLVMType(structProperties[type][prop_name]);
//         currentlyPreferredType = prop_ty;
//         auto v = props[prop_name]->codegen();
//         builder->CreateStore(v, struct_property_ptr);
//         i++;
//     }

//     auto loaded = builder->CreateLoad(ptr);

//     functions[currentFunctionName]->setVariable(name, std::move(loaded));

//     return 0;
// }

// void CodeGenerator::declareStructType(std::vector<llvm::Type *> llvm_types, std::map<std::string, std::string> properties, std::vector<std::string> property_insetion_order, std::string name)
// {
//     auto struct_type = llvm::StructType::create(context, llvm_types, name, false);
//     llvmStructTypes[name] = struct_type;
//     structProperties[name] = properties;
//     structPropertyInsertionOrders[name] = property_insetion_order;
// }

// llvm::Value *CodeGenerator::codegenStructTypeExpression(unique_ptr<StructTypeExpression> structTypeExpr)
// {
//     auto name = structTypeExpr->getName();
//     auto properties = structTypeExpr->getProperties();
//     auto propertyInsertionOrder = structTypeExpr->getPropertyInsertionOrder();
//     std::vector<llvm::Type *> llvm_types;
//     for (auto &prop_name : propertyInsertionOrder)
//     {
//         auto ty = ssTypeToLLVMType(properties[prop_name]);
//         llvm_types.push_back(ty);
//     }

//     declareStructType(llvm_types, properties, propertyInsertionOrder, name);

//     return 0;
// }

// llvm::Value *CodeGenerator::codegenStructValueExpression(unique_ptr<StructValueExpression> structValExpr)
// {
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenIfStatement(unique_ptr<IfStatement> ifStatement)
// {
//     auto conditions = ifStatement->getConditions();
//     auto conditionSeparators = ifStatement->getConditionSeparators();
//     auto then = ifStatement->getThen();

//     std::vector<llvm::Value *> cond_vs;
//     for (auto &cond : conditions)
//     {
//         auto v = cond->codegen();
//         cond_vs.push_back(v);
//     }
//     for (auto &sep : conditionSeparators)
//     {
//         fatalError("Condition seperators in if statements not implemented yet");
//         //TODO unimplemented
//     }

//     auto func = builder->GetInsertBlock()->getParent();
//     auto then_bb = llvm::BasicBlock::Create(context, "if.then", func);
//     auto else_bb = llvm::BasicBlock::Create(context, "if.else");
//     auto merge_bb = llvm::BasicBlock::Create(context, "if.merge");

//     auto br = builder->CreateCondBr(cond_vs[0], then_bb, else_bb);
//     builder->SetInsertPoint(then_bb);

//     then->codegen();

//     builder->CreateBr(merge_bb);

//     then_bb = builder->GetInsertBlock();

//     func->getBasicBlockList().push_back(else_bb);
//     builder->SetInsertPoint(else_bb);

//     builder->CreateBr(merge_bb);

//     else_bb = builder->GetInsertBlock();

//     func->getBasicBlockList().push_back(merge_bb);
//     builder->SetInsertPoint(merge_bb);

//     return 0;
// }

// llvm::Value *CodeGenerator::codegenReturnStatement(unique_ptr<ReturnStatement> retStatement)
// {
//     currentlyPreferredType = ssTypeToLLVMType(functions[currentFunctionName]->getReturnType());
//     auto v = retStatement->getValue()->codegen();
//     builder->CreateRet(v);
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenImportStatement(unique_ptr<ImportStatement> importStatement)
// {
//     return 0;
// }

// llvm::Value *CodeGenerator::codegenFunctionCallExpression(unique_ptr<FunctionCallExpression> fnCallExpr)
// {
//     auto name = fnCallExpr->getName();
//     auto params = fnCallExpr->getParams();

//     if (name.substr(0, 1) == "@")
//     {
//         name = name.substr(1, name.size() - 1);
//     }
//     auto callee_function = module->getFunction(name);
//     if (callee_function == 0)
//         fatalError("Unknown function referenced in function call");
//     if (callee_function->arg_size() != params.size())
//         fatalError("Incorrect number of parameters passed to function call");

//     std::vector<llvm::Value *> param_values;
//     std::vector<llvm::Type *> param_types;
//     for (auto it = callee_function->args().begin(), end = callee_function->args().end(); it != end; it++)
//     {
//         param_types.push_back(it->getType());
//     }

//     int i = 0;
//     for (auto &param : params)
//     {
//         currentlyPreferredType = param_types[i];
//         auto v = param->codegen();
//         param_values.push_back(v);
//         i++;
//     }

//     return builder->CreateCall(callee_function, param_values);
// }

// void CodeGenerator::declareFunction(std::string name, std::vector<llvm::Type *> param_types, llvm::Type *return_type, llvm::Module *mod)
// {
//     llvm::FunctionType *function_type = llvm::FunctionType::get(return_type, param_types, false);
//     llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, mod);
// }

// void CodeGenerator::createFunctionParamAllocas(llvm::Function *f, std::map<std::string, std::string> params)
// {

//     llvm::Function::arg_iterator f_it = f->arg_begin();
//     auto param_it = params.begin();
//     while (param_it != params.end())
//     {
//         auto ptr = createEntryBlockAlloca(f, param_it->first, ssTypeToLLVMType(param_it->second));
//         auto store = builder->CreateStore(f_it, ptr);
//         auto loaded = builder->CreateLoad(ptr);
//         functions[currentFunctionName]->setVariable(param_it->first, std::move(loaded));
//         param_it++;
//         f_it++;
//     }
// }

// llvm::Value *CodeGenerator::createEntryBlockAlloca(llvm::Function *function, const std::string &name, llvm::Type *type)
// {
//     llvm::IRBuilder<> tmp_builder(&function->getEntryBlock(), function->getEntryBlock().begin());
//     return tmp_builder.CreateAlloca(type);
// }

// llvm::Type *CodeGenerator::ssTypeToLLVMType(std::string type)
// {
//     std::string base_type = "";
//     for (const char &c : type)
//     {
//         if (c != (const char &)"*" && c != (const char &)"&")
//         {
//             base_type += c;
//         }
//     }

//     auto llvm_type = ssBaseTypeToLLVMType(base_type);

//     std::string rest = type.substr(base_type.size(), type.size() - 1);

//     for (auto &c : rest)
//     {
//         if (c == (char &)"*")
//         {
//             llvm_type = llvm_type->getPointerTo();
//         }
//     }

//     return llvm_type;
// }

// llvm::Type *CodeGenerator::ssBaseTypeToLLVMType(std::string type)
// {
//     if (type == "i64")
//         return llvm::Type::getInt64Ty(context);
//     else if (type == "i32")
//         return llvm::Type::getInt32Ty(context);
//     else if (type == "i16")
//         return llvm::Type::getInt16Ty(context);
//     else if (type == "i8")
//         return llvm::Type::getInt8Ty(context);
//     else if (type == "void")
//         return llvm::Type::getVoidTy(context);
//     else
//     {
//         if (std::find(structTypeNames.begin(), structTypeNames.end(), type) != structTypeNames.end())
//             return llvmStructTypes[type];
//         fatalError("Could not convert base type to llvm type");
//         return nullptr;
//     }
// }

// void CodeGenerator::printT(llvm::Type *ty)
// {
//     ty->print(llvm::outs());
//     llvm::outs() << '\n';
// }

// void CodeGenerator::printV(llvm::Value *v)
// {
//     v->print(llvm::outs());
//     llvm::outs() << '\n';
// }

// void CodeGenerator::printModule(llvm::Module *mod)
// {
//     auto writer = new llvm::AssemblyAnnotationWriter();
//     mod->print(llvm::outs(), writer);
// }

// void CodeGenerator::writeModuleToFile(llvm::Module *mod, std::string path)
// {
//     auto writer = new llvm::AssemblyAnnotationWriter();
//     std::error_code ec;
//     auto f_out = llvm::raw_fd_ostream(path, ec);
//     mod->print(f_out, writer);
// }

// void CodeGenerator::fatalError(std::string msg)
// {
//     cout << msg << endl;
//     exit(1);
// }

// // llvm::Value *NumberExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *StringLiteralExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *NullptrExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *VariableReferenceExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *BinaryOperationExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *IndexAccessedExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *UnaryPrefixOperationExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *CodeBlock::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *FunctionDeclaration::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *VariableDeclaration::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *StructTypeExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *StructValueExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *IfStatement::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *ReturnStatement::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *FunctionCallExpression::codegen(llvm::Module *mod) { return 0; }
// // llvm::Value *ImportStatement::codegen(llvm::Module *mod) { return 0; }
