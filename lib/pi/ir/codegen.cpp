#include "codegen.h"

using namespace ssc;

void ssc::codegenNodes(Nodes nodes, unique_ptr<CodegenContext> &codegenContext)
{
    for (auto &node : nodes)
    {
        node->codegen(codegenContext);
    }
}

llvm::Value *ASTUnaryPrefixOperationExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    switch (op)
    {
    case TokenType::tokNew:
        return codegenNew(codegenContext);
    default:
        codegenContext->error("Unimplemented unary prefix operator");
        break;
    }
    return 0;
}

llvm::Value *ASTUnaryPrefixOperationExpression::codegenNew(const unique_ptr<CodegenContext> &codegenContext)
{
    auto v = value->codegen(codegenContext);
    codegenContext->print(v);

    return llvm::getPointerOperand(v);
}

llvm::Value *ASTClassConstructionExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto builder = codegenContext->getBuilder();

    auto ty = codegenContext->getStructType(name);
    auto ptr = builder->CreateAlloca(ty);

    auto constructor = codegenContext->getModule()->getFunction(codegenContext->getASTClassType(name)->getConstructor()->getName());

    if (!constructor)
    {
        codegenContext->error("Could not construct undefined class '" + name + "'");
    }

    if (constructor->arg_size() != parameters.size() + 1) //? parameters.size() + 1 because "this" isn't supplied by user
    {
        codegenContext->error("Incorrect number of parameters supplied to '" + name + "' constructor");
    }

    std::vector<llvm::Value *> paramValues = {ptr};
    for (auto const &param : parameters)
    {
        paramValues.push_back(param->codegen(codegenContext));
    }

    auto call = builder->CreateCall(constructor, paramValues);
    return builder->CreateLoad(ptr);
}

llvm::Value *ASTClassDeclaration::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    std::vector<llvm::Type *> classPropertyTypes;
    std::vector<std::string> propertyNames;
    for (auto const &prop : properties)
    {
        auto ty = prop->getType();
        auto name = prop->getName();
        classPropertyTypes.push_back(codegenContext->ssTypeToLLVMType(ty));
        propertyNames.push_back(name);
    }
    auto llvmClassType = llvm::StructType::create(classPropertyTypes, name);

    codegenContext->setStructType(name, llvmClassType);
    codegenContext->setClassProperties(name, propertyNames);

    if (constructor != nullptr)
    {
        constructor->codegen(codegenContext);
    }
    else
    {
        //TODO Implement default constructor
        codegenContext->warning("Default constructors have not yet been implemented");
    }

    for (auto &method : methods)
    {
        method->codegen(codegenContext);
    }

    codegenContext->setASTClassType(name, this);

    return 0;
}

llvm::Value *ASTFunctionCallExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    llvm::Function *calleeF = codegenContext->getModule()->getFunction(name);
    if (!calleeF)
    {
        codegenContext->error("Function call to undefined function '" + name + "'");
    }

    if (calleeF->arg_size() != params.size())
    {
        codegenContext->error("Incorrect number of parameters supplied in function call to '" + name + "'");
    }

    auto functionParamsIT = calleeF->args().begin();
    std::vector<llvm::Value *> paramValues;
    for (int i = 0; i < params.size(); i++)
    {
        auto v = params[i]->codegen(codegenContext);
        auto functionParamType = functionParamsIT->getType();

        if (functionParamType != v->getType())
        {
            v = codegenContext->implicityTypecastExpression(v, params[i]->getType(), functionParamType);
        }

        paramValues.push_back(v);
        functionParamsIT++;
    }

    return codegenContext->getBuilder()->CreateCall(calleeF, paramValues);
}

llvm::Value *ASTForLoop::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto builder = codegenContext->getBuilder();

    auto f = builder->GetInsertBlock()->getParent();
    auto preBB = builder->GetInsertBlock();
    auto loopBB = llvm::BasicBlock::Create(*codegenContext->getCtx(), "for.loop", f);
    auto mergeBB = llvm::BasicBlock::Create(*codegenContext->getCtx(), "for.merge", f);

    llvm::Value *var;
    if (initialClauseExpression)
    {
        codegenContext->error("error unimplemented method in for loop");
        var = initialClauseExpression->codegen(codegenContext);
    }
    else
    {
        var = initialClauseVarDec->codegen(codegenContext);
    }

    builder->CreateBr(loopBB);
    builder->SetInsertPoint(loopBB);

    var = builder->CreateLoad(llvm::getPointerOperand(var));
    codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setMutable(initialClauseVarDec->getName(), var);

    for (auto &node : then)
    {
        node->codegen(codegenContext);
    }

    auto actionV = action->codegen(codegenContext);

    auto cond = condition->codegen(codegenContext);

    builder->CreateCondBr(cond, loopBB, mergeBB);
    builder->SetInsertPoint(mergeBB);

    return 0;
}

llvm::Value *ASTIfStatement::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto cond = condition->codegen(codegenContext);

    auto builder = codegenContext->getBuilder();

    auto f = builder->GetInsertBlock()->getParent();
    auto thenBB = llvm::BasicBlock::Create(*codegenContext->getCtx(), "if.then", f);
    auto elseBB = llvm::BasicBlock::Create(*codegenContext->getCtx(), "if.else", f);
    auto mergeBB = llvm::BasicBlock::Create(*codegenContext->getCtx(), "if.merge", f);

    builder->CreateCondBr(cond, thenBB, elseBB);

    builder->SetInsertPoint(thenBB);

    for (auto &node : then)
    {
        node->codegen(codegenContext);
    }

    builder->CreateBr(mergeBB);
    builder->SetInsertPoint(elseBB);
    builder->CreateBr(mergeBB);
    builder->SetInsertPoint(mergeBB);

    return 0;
}

llvm::Value *ASTReturnStatement::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto fnRetType = codegenContext->getFunction(codegenContext->getCurrentFunctionName())->getReturnType();
    auto builder = codegenContext->getBuilder();
    auto retValue = value->codegen(codegenContext);

    if (fnRetType != value->getType())
    {
        codegenContext->error("Return value type " + value->getType() + " does not match function return type " + fnRetType);
    }

    return builder->CreateRet(retValue);
}

llvm::Value *ASTFunctionDeclaration::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto f = codegenPrototype(codegenContext);
    if (f == 0)
        codegenContext->error("Could not codegen" + name + "'s function prototype");

    auto entry = llvm::BasicBlock::Create(*codegenContext->getCtx(), "entry", f);
    codegenContext->getBuilder()->SetInsertPoint(entry);
    codegenContext->setCurrentFunctionName(name);
    codegenContext->setFunction(name, this);

    createFunctionParamAllocas(codegenContext, f);

    for (auto &node : then)
    {
        node->codegen(codegenContext);
    }

    auto builder = codegenContext->getBuilder();
    if (!builder->GetInsertBlock()->getTerminator())
    {
        if (returnType != "void")
        {
            codegenContext->error("Implicit return void in function that returns " + returnType);
        }
        builder->CreateRetVoid();
    }

    if (codegenContext->getCompilerOptions()->getOptimize())
    {
        codegenContext->runFPM(f);
    }

    llvm::verifyFunction(*f, &llvm::outs());

    return 0;
}

void ASTFunctionDeclaration::createFunctionParamAllocas(const unique_ptr<CodegenContext> &codegenContext, llvm::Function *f)
{
    auto builder = codegenContext->getBuilder();
    auto functionParamIt = f->arg_begin();
    for (auto &param : parameters)
    {
        auto ty = codegenContext->ssTypeToLLVMType(param.type);
        auto ptr = builder->CreateAlloca(ty, nullptr, param.name);
        auto store = builder->CreateStore(functionParamIt, ptr);
        auto loaded = builder->CreateLoad(ptr);
        if (param.mut)
        {
            codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setMutable(param.name, loaded);
        }
        else
        {
            codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setConstant(param.name, loaded);
        }

        functionParamIt++;
    }
}

llvm::Function *ASTFunctionDeclaration::codegenPrototype(const unique_ptr<CodegenContext> &codegenContext)
{
    std::vector<llvm::Type *> paramTypes;
    for (auto &param : parameters)
    {
        auto ty = codegenContext->ssTypeToLLVMType(param.type);
        paramTypes.push_back(ty);
    }

    auto llvmReturnType = codegenContext->ssTypeToLLVMType(returnType);
    llvm::FunctionType *functionType = llvm::FunctionType::get(llvmReturnType, paramTypes, false);
    // auto linkage = pub ? llvm::Function::ExternalLinkage : llvm::Function::PrivateLinkage;
    auto linkage = llvm::Function::ExternalLinkage;
    llvm::Function *f = llvm::Function::Create(functionType, linkage, name, codegenContext->getModule());

    if (f->getName() != name)
    {
        f->eraseFromParent();
        f = codegenContext->getModule()->getFunction(name);

        if (!f->empty())
            codegenContext->error("Redefinition of function " + name);

        if (f->arg_size() != parameters.size())
            codegenContext->error("Redefinition of function with different number of arguments");
    }

    unsigned idx = 0;
    for (llvm::Function::arg_iterator ai = f->arg_begin(); idx != parameters.size(); ai++, idx++)
    {
        ai->setName(parameters[idx].name);
    }

    return f;
}

llvm::Value *ASTVariableDeclaration::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto builder = codegenContext->getBuilder();

    auto ty = codegenContext->ssTypeToLLVMType(type);
    auto ptr = builder->CreateAlloca(ty);

    codegenContext->print(ptr);

    auto isSigned = codegenContext->isTypeSigned(type);

    if (value)
    {
        auto val = value->codegen(codegenContext);

        if (ptr->getType()->getPointerElementType() != val->getType())
        {
            codegenContext->error("Types do not match in " + name + "'s variable initialization");
        }

        builder->CreateStore(val, ptr);
    }

    auto loaded = builder->CreateLoad(ptr, name);

    if (mut)
    {
        codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setMutable(name, loaded);
    }
    else
    {
        codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setConstant(name, loaded);
    }

    codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setVariable(name, this);

    return loaded;
}

llvm::Value *ASTTypecastExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto expr = value->codegen(codegenContext);
    auto originalType = value->getType();

    if (originalType == type)
    {
        if (codegenContext->getCompilerOptions()->getCodegenWarnings()[CodegenWarnings::UnnecessaryTypecast])
        {
            codegenContext->warning("Unnecessary typecast from " + originalType + " to " + type);
        }
        return expr;
    }

    auto ogLLVMType = codegenContext->ssTypeToLLVMType(originalType);
    auto targetLLVMType = codegenContext->ssTypeToLLVMType(type);
    auto builder = codegenContext->getBuilder();

    if (targetLLVMType->isFloatTy() || targetLLVMType->isDoubleTy())
    {
        if (ogLLVMType->isFloatTy() || ogLLVMType->isDoubleTy())
        {
            return builder->CreateFPCast(expr, targetLLVMType);
        }
        else if (ogLLVMType->isIntegerTy())
        {
            if (codegenContext->isTypeSigned(originalType))
            {
                return builder->CreateSIToFP(expr, targetLLVMType);
            }
            return builder->CreateUIToFP(expr, targetLLVMType);
        }
    }
    else if (targetLLVMType->isIntegerTy())
    {
        if (ogLLVMType->isFloatTy() || ogLLVMType->isDoubleTy())
        {
            if (codegenContext->isTypeSigned(type))
            {
                return builder->CreateFPToSI(expr, targetLLVMType);
            }
            return builder->CreateFPToUI(expr, targetLLVMType);
        }
        else if (ogLLVMType->isIntegerTy())
        {
            return builder->CreateIntCast(expr, targetLLVMType, codegenContext->isTypeSigned(originalType));
        }
    }

    codegenContext->error("Could not typecast " + originalType + " to " + type);

    return 0;
}

llvm::Value *ASTBinaryOperationExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    switch (op)
    {
    case TokenType::tokPlus:
        return codegenBinopSumDiffProdQuot(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokMinus:
        return codegenBinopSumDiffProdQuot(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokAsterisk:
        return codegenBinopSumDiffProdQuot(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokSlash:
        return codegenBinopSumDiffProdQuot(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokArrow:
        return codegenBinopArrow(codegenContext, std::move(lhs), std::move(rhs));
    case TokenType::tokEq:
        return codegenBinopEq(codegenContext, std::move(lhs), std::move(rhs));
    case TokenType::tokCompareLt:
        return codegenBinopComp(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokCompareEq:
        return codegenBinopComp(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokOr:
        return codegenBinopComp(codegenContext, std::move(lhs), std::move(rhs), op);
    case TokenType::tokAnd:
        return codegenBinopComp(codegenContext, std::move(lhs), std::move(rhs), op);
    default:
        codegenContext->error("Found unimplemented operator in binary operation expression");
    }
    return 0;
}

llvm::Value *ASTBinaryOperationExpression::codegenBinopArrow(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs)
{
    auto lhsVarRef = dynamic_cast<ASTVariableReferenceExpression *>(lhs.get());
    if (lhsVarRef == nullptr)
        codegenContext->error("Object property access must have a variable refernce expression on the left hand side");
    auto rhsVarRef = dynamic_cast<ASTVariableReferenceExpression *>(rhs.get());
    if (rhsVarRef == nullptr)
        codegenContext->error("Object property access must have a variable refernce expression on the right hand side");

    auto propName = rhsVarRef->getName();

    auto lVar = lhsVarRef->codegen(codegenContext);
    auto ty = lVar->getType();
    while (ty->isPointerTy())
        ty = ty->getPointerElementType();
    std::string className = ty->getStructName().str();

    auto props = codegenContext->getClassProperties(className);
    int propIndex = 0;
    for (auto const &prop : props)
    {
        if (prop == propName)
            break;
        propIndex++;
    }

    if (propIndex == props.size())
        codegenContext->error("Could not find property '" + propName + "' in class");

    auto builder = codegenContext->getBuilder();
    auto gep = builder->CreateStructGEP(lVar, propIndex);
    auto loaded = builder->CreateLoad(gep);
    return loaded;
}

llvm::Value *ASTBinaryOperationExpression::codegenBinopComp(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op)
{
    auto lVal = lhs->codegen(codegenContext);
    auto rVal = rhs->codegen(codegenContext);

    auto builder = codegenContext->getBuilder();

    auto lTy = lVal->getType();
    auto rTy = rVal->getType();

    if (lTy != rTy)
    {
        codegenContext->error("Cannot compare values of different types");
    }

    bool isFloatingPoint = false;
    if (lTy->isFloatTy() || lTy->isDoubleTy())
    {
        isFloatingPoint = true;
    }

    // TODO Find out what the fuck U vs O is in FCmp
    switch (op)
    {
    case TokenType::tokCompareEq:
    {
        if (isFloatingPoint)
        {
            return builder->CreateFCmpUEQ(lVal, rVal);
        }
        return builder->CreateICmpEQ(lVal, rVal);
    }
    case TokenType::tokCompareLt:
    {
        if (isFloatingPoint)
        {
            return builder->CreateFCmpULT(lVal, rVal);
        }
        if (codegenContext->isTypeSigned(lhs->getType()))
        {
            return builder->CreateICmpSLT(lVal, rVal);
        }
        return builder->CreateICmpULT(lVal, rVal);
    }
    case TokenType::tokOr:
        return builder->CreateOr(lVal, rVal);
    case TokenType::tokAnd:
        return builder->CreateAnd(lVal, rVal);
    default:
        codegenContext->error("Found unimplemented operator in comparison binary operation expression");
        break;
    }

    return 0;
}

llvm::Value *ASTBinaryOperationExpression::codegenBinopEq(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs)
{
    bool isVarRef = true, isBinOp = true;
    auto lVarRefExpr = dynamic_cast<ASTVariableReferenceExpression *>(lhs.get());
    if (lVarRefExpr == nullptr)
    {
        isVarRef = false;
    }
    auto lBinOp = dynamic_cast<ASTBinaryOperationExpression *>(lhs.get());
    if (lBinOp == nullptr)
    {
        isBinOp = false;
    }

    if (!isVarRef && !isBinOp)
    {
        codegenContext->error("Expected variable reference expression or binary operation expression on left hand side of '=' operator");
    }

    // TODO check if bin op is mut
    if (isVarRef)
    {
        if (!lVarRefExpr->getIsMut())
        {
            codegenContext->error("Cannot reassign non-mutable variable " + lVarRefExpr->getName());
        }
    }

    auto lVal = lhs->codegen(codegenContext);
    auto rVal = rhs->codegen(codegenContext);

    // if (lVal->getType() != rVal->getType())
    // {
    //     codegenContext->error("Types do not match in " + lVarRefExpr->getName() + " variable reassignment");
    // }

    auto builder = codegenContext->getBuilder();
    auto lPtr = llvm::getPointerOperand(lVal);
    auto store = builder->CreateStore(rVal, lPtr);

    // ? We've already checked if it's mutable so dw about it here
    // codegenContext->getFunction(codegenContext->getCurrentFunctionName())->setMutable(lVarRefExpr->getName(), builder->CreateLoad(lPtr));

    return store;
}

llvm::Value *ASTBinaryOperationExpression::codegenBinopSumDiffProdQuot(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op)
{
    auto builder = codegenContext->getBuilder();
    auto lVal = lhs->codegen(codegenContext);
    auto rVal = rhs->codegen(codegenContext);
    auto llvmTy = codegenContext->ssTypeToLLVMType(type);

    if (lhs->getType() != rhs->getType())
    {
        codegenContext->error("LHS and RHS of binary operation expression have different types: " + lhs->getType() + " and " + rhs->getType());
    }

    if (llvmTy->isFloatTy() || llvmTy->isDoubleTy())
    {
        switch (op)
        {
        case TokenType::tokPlus:
            return builder->CreateFAdd(lVal, rVal);
        case TokenType::tokMinus:
            return builder->CreateFSub(lVal, rVal);
        case TokenType::tokAsterisk:
            return builder->CreateFMul(lVal, rVal);
        case TokenType::tokSlash:
            return builder->CreateFDiv(lVal, rVal);
        default:
            codegenContext->error("Found unimplemented operator in binary operation expression");
            break;
        }
    }

    switch (op)
    {
    case TokenType::tokPlus:
        return builder->CreateAdd(lVal, rVal);
    case TokenType::tokMinus:
        return builder->CreateSub(lVal, rVal);
    case TokenType::tokAsterisk:
        return builder->CreateMul(lVal, rVal);
    case TokenType::tokSlash:
    {
        if (codegenContext->isTypeSigned(type))
            return builder->CreateSDiv(lVal, rVal);
        return builder->CreateUDiv(lVal, rVal);
    }
    default:
        codegenContext->error("Found unimplemented operator in binary operation expression");
        break;
    }

    return 0;
}

llvm::Value *ASTNumberExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto llvmPreferredType = codegenContext->ssTypeToLLVMType(type);
    if (llvmPreferredType->isFloatTy() || llvmPreferredType->isDoubleTy())
    {
        return llvm::ConstantFP::get(llvmPreferredType, value);
    }
    return llvm::ConstantInt::get(llvmPreferredType, value);
}

llvm::Value *ASTVariableReferenceExpression::codegen(const unique_ptr<CodegenContext> &codegenContext)
{
    auto f = codegenContext->getFunction(codegenContext->getCurrentFunctionName());

    auto foundInConstants = f->getConstants().count(name);
    auto foundInMutables = f->getMutables().count(name);

    if (!foundInMutables && !foundInConstants)
    {
        codegenContext->error("Unknown variable " + name + " referenced");
    }

    auto builder = codegenContext->getBuilder();

    if (foundInConstants)
    {
        auto c = f->getConstant(name);

        auto var = codegenContext->getFunction(codegenContext->getCurrentFunctionName())->getVariable(name);
        return c;
    }
    return f->getMutable(name);
}