#ifndef JIT_H
#define JIT_H

#include <llvm/ADT/StringRef.h>
#include "llvm/ExecutionEngine/JITSymbol.h"
#include "llvm/ExecutionEngine/Orc/CompileUtils.h"
#include "llvm/ExecutionEngine/Orc/Core.h"
#include "llvm/ExecutionEngine/Orc/ExecutionUtils.h"
#include "llvm/ExecutionEngine/Orc/IRCompileLayer.h"
#include "llvm/ExecutionEngine/Orc/JITTargetMachineBuilder.h"
#include "llvm/ExecutionEngine/Orc/RTDyldObjectLinkingLayer.h"
#include "llvm/ExecutionEngine/SectionMemoryManager.h"
#include "llvm/IR/DataLayout.h"
#include "llvm/IR/LLVMContext.h"

#include <memory>
#include <iostream>

namespace llvm
{
  namespace orc
  {
    class SSJIT
    {
    private:
      ExecutionSession ES;
      RTDyldObjectLinkingLayer ObjectLayer;
      IRCompileLayer CompileLayer;

      DataLayout DL;
      MangleAndInterner Mangle;
      ThreadSafeContext Ctx;

    public:
      SSJIT(JITTargetMachineBuilder JTMB, DataLayout DL)
          : ObjectLayer(ES,
                        []() { return std::make_unique<SectionMemoryManager>(); }),
            CompileLayer(ES, ObjectLayer, ConcurrentIRCompiler(std::move(JTMB))),
            DL(std::move(DL)), Mangle(ES, this->DL),
            Ctx(std::make_unique<LLVMContext>())
      {
        ES.getMainJITDylib().setGenerator(
            cantFail(DynamicLibrarySearchGenerator::GetForCurrentProcess(DL)));
      };
    }
  } // namespace orc
} // namespace llvm
#endif