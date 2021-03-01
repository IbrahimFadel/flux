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
        // if (std::find(struct_types.begin(), struct_types.end(), type) != struct_types.end())
        //     return llvm_struct_types[type];
        error("Could not convert base type: " + type + " to llvm type");
        return nullptr;
    }
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