package codegen

import (
	"fmt"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/utils"
	"github.com/llir/llvm/ir"
	"github.com/llir/llvm/ir/constant"
	"github.com/llir/llvm/ir/types"
)

type IRGenerator struct {
	Nodes  []ast.Node
	Module *ir.Module
}

func (gen *IRGenerator) GenerateIR(ast []ast.Node) {
	gen.Module = ir.NewModule()

	for _, node := range ast {
		gen.CodegenNode(node)
	}

	fmt.Println(gen.Module)
}

func (gen *IRGenerator) CodegenNode(node ast.Node) {
	switch node.Type() {
	default:
		utils.FatalError(fmt.Sprintf("could not codegen node of type: %d", node.Type()))
	case ast.NodeTypeFuncDecl:
		gen.CodegenFuncDecl(node)
	}
}

func (gen *IRGenerator) CodegenFuncDecl(decl ast.Decl) {
	fnDecl, ok := decl.(ast.FuncDecl)
	if !ok {
		utils.FatalError("could not cast ast.Decl to ast.FuncDecl")
	}

	fmt.Println(fnDecl.Name)

	retType, err := gen.StringTypeToLLVMType(fnDecl.FuncType.Return)
	if err != nil {
		utils.FatalError(fmt.Sprintf("could not codegen function declaration: %s", err.Error()))
	}
	fn := gen.Module.NewFunc(fnDecl.Name, retType)
	entry := fn.NewBlock("")
	entry.NewRet(constant.NewInt(types.I32, 0))

}

// TODO: finish implementing
func (gen *IRGenerator) StringTypeToLLVMType(ty string) (types.Type, error) {
	baseType := ""
	for _, c := range ty {
		baseType += string(c)
	}

	llvmType, err := gen.BaseTypeToLLVMType(baseType)
	if err != nil {
		return llvmType, fmt.Errorf("could not convert base type '%s' to LLVM type: %s", baseType, err.Error())
	}

	// rest := ty[len(baseType) : len(ty)-1]

	// for _, c := range rest {
	// 	if c == '*' {
	// 		llvmType = llvmType.getPointerT()
	// 	}
	// }
	return llvmType, nil
}

func (gen *IRGenerator) BaseTypeToLLVMType(baseType string) (types.Type, error) {
	switch baseType {
	default:
		return types.I32Ptr.ElemType, fmt.Errorf("unknown type: %s", baseType)
	case "i32":
		return types.I32Ptr.ElemType, nil
	}
}
