#include "codegen.h"

using namespace ssc;

void ssc::codegenNodes(Nodes nodes, unique_ptr<CodegenContext> &codegenContext)
{
    for (auto &node : nodes)
    {
        node->codegen(codegenContext);
    }
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

    auto llvmFnRetType = codegenContext->ssTypeToLLVMType(fnRetType);
    auto retValueType = retValue->getType();

    if (llvmFnRetType->isIntegerTy())
    {
        if (retValueType->isFloatTy() || retValueType->isDoubleTy())
        {
            if (codegenContext->isTypeSigned(fnRetType))
            {
                retValue = builder->CreateFPToSI(retValue, llvmFnRetType);
            }
            else
            {
                retValue = builder->CreateFPToUI(retValue, llvmFnRetType);
            }
        }
        else if (retValueType->isIntegerTy())
        {
            retValue = builder->CreateIntCast(retValue, llvmFnRetType, codegenContext->isTypeSigned(fnRetType));
        }
    }
    else if (llvmFnRetType->isFloatTy() || llvmFnRetType->isDoubleTy())
    {
        if (retValueType->isFloatTy() || retValueType->isDoubleTy())
        {
            retValue = builder->CreateFPCast(retValue, llvmFnRetType);
        }
        else if (retValueType->isIntegerTy())
        {
            auto var = codegenContext->getRecentlyReferencedVar();
            if (codegenContext->isTypeSigned(var->getType()))
            {
                retValue = builder->CreateSIToFP(retValue, llvmFnRetType);
            }
            else
            {
                retValue = builder->CreateUIToFP(retValue, llvmFnRetType);
            }
        }
    }

    retValueType = retValue->getType();

    if (llvmFnRetType != retValueType)
    {
        std::string typeStr;
        llvm::raw_string_ostream typeStrStream(typeStr);
        retValueType->print(typeStrStream);
        codegenContext->error("Expected return type of " + fnRetType + " but got " + typeStr);
    }

    auto ret = builder->CreateRet(retValue);
    return ret;
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

    auto isSigned = codegenContext->isTypeSigned(type);

    auto val = value->codegen(codegenContext);

    if (ptr->getType()->getPointerElementType() != val->getType())
    {
        codegenContext->error("Types do not match in " + name + "'s variable initialization");
    }

    builder->CreateStore(val, ptr);
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

    return 0;
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

    auto lVarRefExpr = static_cast<ASTVariableReferenceExpression *>(lhs.get());
    if (!lVarRefExpr)
    {
        codegenContext->error("Expected variable reference expression on left hand side of '=' operator");
    }

    if (!lVarRefExpr->getIsMut())
    {
        codegenContext->error("Cannot reassign non-mutable variable " + lVarRefExpr->getName());
    }

    auto lVal = lhs->codegen(codegenContext);
    auto rVal = rhs->codegen(codegenContext);

    if (lVal->getType() != rVal->getType())
    {
        codegenContext->error("Types do not match in " + lVarRefExpr->getName() + " variable reassignment");
    }

    auto builder = codegenContext->getBuilder();
    return builder->CreateStore(rVal, llvm::getPointerOperand(lVal));
}

llvm::Value *ASTBinaryOperationExpression::codegenBinopSumDiffProdQuot(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op)
{
    auto builder = codegenContext->getBuilder();
    auto lVal = lhs->codegen(codegenContext);
    auto rVal = rhs->codegen(codegenContext);
    auto llvmTy = codegenContext->ssTypeToLLVMType(type);

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

    auto foundInConstants = false;
    auto foundInMutables = false;
    for (auto &constant : f->getConstants())
    {
        if (constant.first == name)
        {
            foundInConstants = true;
        }
    }
    for (auto &mut : f->getMutables())
    {
        if (mut.first == name)
        {
            foundInMutables = true;
        }
    }

    if (!foundInMutables && !foundInConstants)
    {
        codegenContext->error("Unknown variable " + name + " referenced");
    }

    if (foundInConstants)
    {
        auto c = f->getConstant(name);

        auto var = codegenContext->getFunction(codegenContext->getCurrentFunctionName())->getVariable(name);
        codegenContext->setRecentlyReferencedVar(var);

        return c;
    }
    return f->getMutable(name);
}