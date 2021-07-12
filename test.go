package main

import (
	"fmt"

	"github.com/llir/llvm/ir"
	"github.com/llir/llvm/ir/constant"
	"github.com/llir/llvm/ir/types"
)

func main() {
	// mod := ir.NewModule()

	// structName := "Foo"

	// structTy := types.StructType{TypeName: structName}

	// structTy.Fields = append(structTy.Fields, types.I32)
	// structTy.Fields = append(structTy.Fields, types.I32)
	// structTy.Fields = append(structTy.Fields, types.I32)

	// someTypeName := mod.NewTypeDef("SomeTypeName", types.I32)

	// fn := mod.NewFunc("main", types.I32)
	// bb := fn.NewBlock("entry")

	// bb.NewAlloca(someTypeName)
	// bb.NewAlloca(&structTy)

	// bb.NewRet(constant.NewInt(types.I32, 0))

	zero := constant.NewInt(types.I32, 0)
	m := ir.NewModule()

	m.NewTypeDef("SomeTypeName", types.I32)
	foo := m.NewTypeDef("foo", types.NewStruct(types.I32))

	main := m.NewFunc("main", types.I32)
	b := main.NewBlock("")
	fooInstance := b.NewAlloca(foo)
	fieldX := b.NewGetElementPtr(foo, fooInstance, zero, zero)
	// now `fieldX` is a pointer to field `x` of `foo`.
	b.NewStore(constant.NewInt(types.I32, 10), fieldX)
	b.NewLoad(types.I32, fieldX)
	b.NewRet(zero)

	fmt.Println(m)
}
