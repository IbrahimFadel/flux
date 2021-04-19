#include "context.h"

using namespace ssc;

void CodegenContext::error(std::string msg)
{
    std::cerr << "\033[1;31m"
              << "Codegen Error: "
              << "\033[0m" << msg << std::endl;
    exit(1);
}

void CodegenContext::warning(std::string msg)
{
    if (compilerOptions->getWError())
    {
        error(msg);
    }

    std::cerr << "\033[1;33m"
              << "Codegen Warning: "
              << "\033[0m" << msg << std::endl;
}

llvm::Type *CodegenContext::ssTypeToLLVMType(std::string type)
{
    std::string baseType = "";
    for (const char &c : type)
    {
        if (c != (const char &)"*" && c != (const char &)"&")
        {
            baseType += c;
        }
    }

    auto llvmType = ssBaseTypeToLLVMType(baseType);

    std::string rest = type.substr(baseType.size(), type.size() - 1);

    for (auto &c : rest)
    {
        if (c == (char &)"*")
        {
            llvmType = llvmType->getPointerTo();
        }
        else
        {
            error("Unimplemented character '" + std::string(1, c) + "' converting ss type to LLVM type");
        }
    }

    return llvmType;
}

llvm::Type *CodegenContext::ssBaseTypeToLLVMType(std::string type)
{
    if (type == "i64")
        return llvm::Type::getInt64Ty(ctx);
    else if (type == "u64")
        return llvm::Type::getInt64Ty(ctx);
    else if (type == "i32")
        return llvm::Type::getInt32Ty(ctx);
    else if (type == "u32")
        return llvm::Type::getInt32Ty(ctx);
    else if (type == "i16")
        return llvm::Type::getInt16Ty(ctx);
    else if (type == "u16")
        return llvm::Type::getInt16Ty(ctx);
    else if (type == "i8")
        return llvm::Type::getInt8Ty(ctx);
    else if (type == "u8")
        return llvm::Type::getInt8Ty(ctx);
    else if (type == "f64")
        return llvm::Type::getDoubleTy(ctx);
    else if (type == "f32")
        return llvm::Type::getFloatTy(ctx);
    else if (type == "bool")
        return llvm::Type::getInt1Ty(ctx);
    else if (type == "void")
        return llvm::Type::getVoidTy(ctx);
    else
    {
        if (isClassType(type))
            return structTypes[type];
        error("Could not convert base type: " + type + " to llvm type");
        return nullptr;
    }
}

bool CodegenContext::isClassType(std::string type)
{
    // std::string baseType = "";
    // for (const char &c : type)
    // {
    //     if (c != (const char &)"*" && c != (const char &)"&")
    //     {
    //         baseType += c;
    //     }
    // }

    return (structTypes.find(type) != structTypes.end());
}

bool CodegenContext::isTypeSigned(std::string type)
{
    if (type == "u64")
        return false;
    else if (type == "u32")
        return false;
    else if (type == "u16")
        return false;
    else if (type == "u8")
        return false;

    return true;
}

llvm::Value *CodegenContext::implicityTypecastExpression(llvm::Value *v, std::string currentType, llvm::Type *targetType)
{
    auto vType = v->getType();
    if (vType->isIntegerTy())
    {
        if (targetType->isIntegerTy())
        {
            return builder.CreateIntCast(v, targetType, isTypeSigned(currentType));
        }
        else if (targetType->isFloatTy() || targetType->isDoubleTy())
        {
            if (isTypeSigned(currentType))
            {
                return builder.CreateSIToFP(v, targetType);
            }
            else
            {
                return builder.CreateUIToFP(v, targetType);
            }
        }
    }
    else if (vType->isFloatTy() || vType->isDoubleTy())
    {
        if (targetType->isIntegerTy())
        {
            if (isTypeSigned(currentType))
            {
                return builder.CreateFPToSI(v, targetType);
            }
            else
            {
                return builder.CreateFPToUI(v, targetType);
            }
        }
        else if (targetType->isFloatTy() || targetType->isDoubleTy())
        {
            return builder.CreateFPCast(v, targetType);
        }
    }
    else
    {
        error("Could not implicitly typecast");
    }
    return 0;
}

void CodegenContext::initializeFPM()
{
    fpm = std::make_unique<llvm::legacy::FunctionPassManager>(mod);
    fpm->add(llvm::createInstructionCombiningPass());
    fpm->add(llvm::createReassociatePass());
    fpm->add(llvm::createDeadCodeEliminationPass());
    fpm->add(llvm::createGVNPass());
    fpm->add(llvm::createCFGSimplificationPass());
    fpm->add(llvm::createPromoteMemoryToRegisterPass());
    fpm->doInitialization();
}

void CodegenContext::runFPM(llvm::Function *f)
{
    fpm->run(*f);
}

void CodegenContext::defineCFunctions()
{
    std::vector<llvm::Type *> paramTypes = {llvm::Type::getInt32Ty(ctx)};
    llvm::Type *returnType = llvm::Type::getInt8PtrTy(ctx);
    llvm::FunctionType *functionType = llvm::FunctionType::get(returnType, paramTypes, false);
    mallocFunction = llvm::Function::Create(functionType, llvm::Function::ExternalLinkage, "malloc", mod);

    paramTypes = {llvm::Type::getInt8PtrTy(ctx)};
    returnType = llvm::Type::getVoidTy(ctx);
    functionType = llvm::FunctionType::get(returnType, paramTypes, false);
    freeFunction = llvm::Function::Create(functionType, llvm::Function::ExternalLinkage, "free", mod);
}