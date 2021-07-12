package codegen

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/IbrahimFadel/pi-lang/ast"
	"github.com/IbrahimFadel/pi-lang/utils"
	"github.com/llir/llvm/ir"
	"github.com/llir/llvm/ir/constant"
	"github.com/llir/llvm/ir/types"
	"github.com/llir/llvm/ir/value"
)

type VTable map[string]value.Value // function value

type IRGenerator struct {
	Nodes        []ast.Node
	Module       *ir.Module
	CurBB        *ir.Block
	CurBlockStmt *ast.BlockStmt

	TypedefLLVMTypes map[string]*types.Type

	InterfaceTypeExprs   map[string]*ast.InterfaceTypeExpr
	InterfaceVTableTypes map[string]*types.StructType
	InterfaceVTables     map[string]*constant.Struct
	CurTypeDeclName      string
}

func (gen *IRGenerator) Init() {
	gen.TypedefLLVMTypes = make(map[string]*types.Type)
	gen.InterfaceTypeExprs = make(map[string]*ast.InterfaceTypeExpr)
	gen.InterfaceVTableTypes = make(map[string]*types.StructType)
	gen.InterfaceVTables = make(map[string]*constant.Struct)
}

func (gen *IRGenerator) GenerateIR(ast []ast.Node) {
	gen.Init()
	gen.Module = ir.NewModule()

	for _, node := range ast {
		gen.Node(node)
	}
}

func (gen *IRGenerator) Node(node ast.Node) {
	switch n := node.(type) {
	default:
		utils.FatalError(fmt.Sprintf("could not codegen node of type: %v", n))
	case ast.FuncDecl:
		gen.FuncDecl(n)
	case ast.ReturnStmt:
		gen.ReturnStmt(n)
	case ast.VarDecl:
		gen.VarDecl(n)
	case ast.TypeDecl:
		gen.TypeDecl(n)
	}
}

func (gen *IRGenerator) TypeDecl(typeDecl ast.TypeDecl) {
	gen.CurTypeDeclName = typeDecl.Name // TODO: ew refactor... need a better way to do this
	ty, err := gen.Type(typeDecl.Type)
	if err != nil {
		utils.FatalError(fmt.Sprintf("could not codegen type value in type declaration: %s", err.Error()))
	}

	gen.TypedefLLVMTypes[gen.CurTypeDeclName] = &ty
	gen.Module.NewTypeDef(typeDecl.Name, ty)
}

func (gen *IRGenerator) VarDecl(varDecl ast.VarDecl) {
	ty, err := gen.Type(varDecl.Type)
	if err != nil {
		utils.FatalError(fmt.Sprintf("could not codegen type: %s", err.Error()))
	}

	for i, v := range varDecl.Values {
		ptr := gen.CurBB.NewAlloca(ty)
		val, err := gen.Expr(v)
		if err != nil {
			utils.FatalError(fmt.Sprintf("could not codegen const declaration expression: %s", err.Error()))
		}
		gen.CurBB.NewStore(val, ptr)
		loaded := gen.CurBB.NewLoad(ty, ptr)
		if varDecl.Mut {
			gen.CurBlockStmt.Mutables[varDecl.Names[i]] = loaded
		} else {
			gen.CurBlockStmt.Constants[varDecl.Names[i]] = loaded
		}
	}
}

func (gen *IRGenerator) Expr(expr ast.Expr) (value.Value, error) {
	switch e := expr.(type) {
	default:
		fmt.Println(utils.PrettyPrint(e))
		return constant.NewInt(types.I32, 0), fmt.Errorf("unimplemented expression type")
	case ast.NumberLitExpr:
		return gen.NumberLitExpr(e)
	case ast.NullExpr:
		return gen.NullExpr(e)
	case ast.VarRefExpr:
		return gen.VarRefExpr(e)
	}
}

func (gen *IRGenerator) VarRefExpr(ref ast.VarRefExpr) (value.Value, error) {
	if v, found := gen.CurBlockStmt.Constants[ref.Name]; found {
		return v, nil
	} else if v, found := gen.CurBlockStmt.Mutables[ref.Name]; found {
		return v, nil
	}
	return constant.False, fmt.Errorf("could not find variable '%s'", ref.Name)
}

func (gen *IRGenerator) NullExpr(nullExpr ast.NullExpr) (value.Value, error) {
	ty, err := gen.Type(nullExpr.Type)
	if err != nil {
		return constant.NewNull(types.I32Ptr), fmt.Errorf("could not generate null expression type: %s", err.Error())
	}
	return gen.CurBB.NewLoad(ty, constant.NewNull(&types.PointerType{ElemType: ty})), nil
}

func (gen *IRGenerator) NumberLitExpr(num ast.NumberLitExpr) (value.Value, error) {
	errVal := constant.NewInt(types.I32, 0)
	ty, err := gen.Type(num.Type)
	if err != nil {
		return errVal, fmt.Errorf("could not codegen type: %s", err.Error())
	}

	if ty.Equal(types.Double) {
		doubleTy, ok := ty.(*types.FloatType)
		if !ok {
			return errVal, fmt.Errorf("could not cast type to double type")
		}
		val, err := strconv.ParseFloat(num.Value, 64)
		if err != nil {
			return errVal, fmt.Errorf("could not convert %s to double", num.Value)
		}
		return constant.NewFloat(doubleTy, val), nil
	} else if ty.Equal(types.Float) {
		floatTy, ok := ty.(*types.FloatType)
		if !ok {
			return errVal, fmt.Errorf("could not cast type to float type")
		}
		val, err := strconv.ParseFloat(num.Value, 32)
		if err != nil {
			return errVal, fmt.Errorf("could not convert %s to float", num.Value)
		}
		return constant.NewFloat(floatTy, val), nil
	} else if ty.Equal(types.I64) || ty.Equal(types.I32) || ty.Equal(types.I16) || ty.Equal(types.I8) || ty.Equal(types.I1) {
		intTy, ok := ty.(*types.IntType)
		if !ok {
			return errVal, fmt.Errorf("could not cast type to int type")
		}
		val, err := strconv.Atoi(num.Value)
		if err != nil {
			return errVal, fmt.Errorf("could not convert %s to int", num.Value)
		}
		return constant.NewInt(intTy, int64(val)), nil
	}

	return errVal, fmt.Errorf("number literal of unknown type") // I don't think this can be reached
}

func (gen *IRGenerator) ReturnStmt(ret ast.ReturnStmt) {
	val, err := gen.Expr(ret.Value)
	if err != nil {
		utils.FatalError(fmt.Sprintf("could not codegen expression: %s", err.Error()))
	}
	gen.CurBB.NewRet(val)
}

func (gen *IRGenerator) FuncDecl(fnDecl ast.FuncDecl) {
	retType, err := gen.Type(fnDecl.FuncType.Return)
	if err != nil {
		utils.FatalError(fmt.Sprintf("could not codegen function declaration: %s", err.Error()))
	}

	var params []*ir.Param

	empty := ast.FuncReceiver{}
	hasReceiver := fnDecl.Receiver != empty

	if hasReceiver {
		recvTy, err := gen.Type(fnDecl.Receiver.Type)
		if err != nil {
			utils.FatalError("could not codegen function receiver type")
		}
		params = append(params, ir.NewParam(fnDecl.Receiver.Name, recvTy))
	}

	for _, param := range fnDecl.FuncType.Params.Params {
		paramTy, err := gen.Type(param.Type)
		if err != nil {
			utils.FatalError("could not codegen function receiver type")
		}
		params = append(params, ir.NewParam(param.Name, paramTy))
	}

	fnName := fnDecl.Name
	implementsInterface := false
	var interfaceImplementedName string

	if hasReceiver {
		for interfaceName, knownInterface := range gen.InterfaceTypeExprs {
			if gen.FnImplementsInterface(fnDecl, interfaceName, knownInterface) {
				structName := gen.FindTypeExprName(fnDecl.Receiver.Type)
				fnName = structName + "_" + fnName
				implementsInterface = true
				interfaceImplementedName = interfaceName
				break
			}
		}
	}

	fn := gen.Module.NewFunc(fnName, retType, params...)

	if implementsInterface {
		vTableType := gen.InterfaceVTableTypes[interfaceImplementedName]
		vTableType.Fields = append(vTableType.Fields, fn.Type())

		vTableData := gen.InterfaceVTables[interfaceImplementedName]
		vTableData.Fields = append(vTableData.Fields, fn)
	}

	gen.CurBB = fn.NewBlock(fnDecl.Body.Name)
	gen.CurBlockStmt = &fnDecl.Body
	gen.BlockStmt(fnDecl.Body)

	if gen.CurBB.Term == nil {
		if retType != types.Void {
			utils.FatalError(fmt.Sprintf("missing return statement in function '%s'", fnDecl.Name))
		}
		gen.CurBB.NewRet(nil)
	}
}

func (gen *IRGenerator) FindTypeExprName(ty ast.Expr) string {
	switch t := ty.(type) {
	default:
		utils.FatalError("could not find type expression name")
		return "" // useless but for ide to not complain
	case ast.PointerTypeExpr:
		return gen.FindTypeExprName(t.PointerToType)
	case ast.IdentifierExpr:
		return t.Name
	}
}

func (gen *IRGenerator) FnImplementsInterface(fn ast.FuncDecl, interfaceName string, interfaceTy *ast.InterfaceTypeExpr) bool {
	// .... this shouldnt be this bad... so many loops. find a better way

	fnRetTy, err := gen.Type(fn.FuncType.Return)
	if err != nil {
		utils.FatalError("could not codegen function return type")
	}

	for _, method := range interfaceTy.Methods.Methods {

		if fn.Name != method.Name {
			return false
		}

		methodRetTy, err := gen.Type(method.Return)
		if err != nil {
			utils.FatalError("could not codegen method return type")
		}

		if fnRetTy != methodRetTy {
			return false
		}

		for _, methodParam := range method.Params.Params {
			methodParamTy, err := gen.Type(methodParam.Type)
			if err != nil {
				utils.FatalError("could not codegen method param type")
			}

			for _, fnParam := range fn.FuncType.Params.Params {
				fnParamTy, err := gen.Type(fnParam.Type)
				if err != nil {
					utils.FatalError("could not codegen function param type")
				}

				if methodParamTy != fnParamTy {
					return false
				}
			}
		}
	}

	return true
}

func (gen *IRGenerator) BlockStmt(block ast.BlockStmt) {
	for _, stmt := range block.List {
		gen.Node(stmt)
	}
}

func (gen *IRGenerator) Type(ty ast.Expr) (types.Type, error) {
	switch t := ty.(type) {
	default:
		fmt.Println(utils.PrettyPrint(t))
		return types.Void, fmt.Errorf("could not convert pi type to llvm type")
	case ast.PrimitiveTypeExpr:
		return gen.PrimitiveTypeExpr(t)
	case ast.PointerTypeExpr:
		return gen.PointerTypeExpr(t)
	case ast.VoidExpr:
		return gen.VoidExpr(t)
	case ast.StructTypeExpr:
		return gen.StructTypeExpr(t)
	case ast.InterfaceTypeExpr:
		return gen.InterfaceTypeExpr(t)
	case ast.IdentifierExpr:
		if idType, ok := gen.TypedefLLVMTypes[t.Name]; ok {
			return *idType, nil
		} else if idType, ok := gen.TypedefLLVMTypes[t.Name]; ok {
			return *idType, nil
		} else if idType, ok := gen.TypedefLLVMTypes[t.Name]; ok {
			return *idType, nil
		} else {
			return types.Void, fmt.Errorf("could not convert pi type to llvm type")
		}
	}
}

func (gen *IRGenerator) VoidExpr(_ ast.VoidExpr) (types.Type, error) {
	return types.Void, nil
}

func (gen *IRGenerator) InterfaceTypeExpr(ty ast.InterfaceTypeExpr) (types.Type, error) {
	structTy := types.StructType{}
	gen.Module.NewTypeDef(gen.CurTypeDeclName, &structTy)

	vtableType := types.StructType{}

	gen.Module.NewTypeDef(gen.CurTypeDeclName+"_VTable_Type", &vtableType)
	gen.InterfaceVTableTypes[gen.CurTypeDeclName] = &vtableType

	structTy.Fields = append(structTy.Fields, types.NewPointer(&vtableType))

	vTableData := constant.NewStruct(&vtableType)
	gen.Module.NewGlobalDef(gen.CurTypeDeclName+"_VTable_Data", vTableData)
	gen.InterfaceVTables[gen.CurTypeDeclName] = vTableData
	gen.InterfaceTypeExprs[gen.CurTypeDeclName] = &ty

	return &structTy, nil
}

func (gen *IRGenerator) StructTypeExpr(ty ast.StructTypeExpr) (types.Type, error) {
	var structPropertyTypes []types.Type

	for _, props := range ty.Properties.Properties {
		propTy, err := gen.Type(props.Type)
		if err != nil {
			return types.NewStruct(types.I32), fmt.Errorf("could not codegen '%s''s type: %s", strings.Join(props.Names, ","), err.Error())
		}
		for range props.Names {
			structPropertyTypes = append(structPropertyTypes, propTy)
		}
	}

	structTy := types.StructType{Fields: structPropertyTypes}
	gen.Module.NewTypeDef(gen.CurTypeDeclName, &structTy)

	return &structTy, nil
}

func (gen *IRGenerator) PointerTypeExpr(ty ast.PointerTypeExpr) (types.Type, error) {
	ptrTo, err := gen.Type(ty.PointerToType)
	if err != nil {
		return types.Void, fmt.Errorf("could not convert pointer type to llvmm type: %s", err.Error())
	}
	ptrTy := types.PointerType{ElemType: ptrTo}
	return &ptrTy, nil
}

func (gen *IRGenerator) PrimitiveTypeExpr(ty ast.PrimitiveTypeExpr) (types.Type, error) {
	switch ty.PrimitiveType {
	default:
		return types.Void, fmt.Errorf("could not convert type %d to LLVM type", ty.PrimitiveType)
	case ast.TokenTypeI64, ast.TokenTypeU64:
		return types.NewInt(64), nil
	case ast.TokenTypeI32, ast.TokenTypeU32:
		return types.NewInt(32), nil
	case ast.TokenTypeI16, ast.TokenTypeU16:
		return types.NewInt(16), nil
	case ast.TokenTypeI8, ast.TokenTypeU8:
		return types.NewInt(8), nil
	case ast.TokenTypeF64:
		return &types.FloatType{Kind: types.Double.Kind}, nil
	case ast.TokenTypeF32:
		return &types.FloatType{Kind: types.Float.Kind}, nil
	case ast.TokenTypeBool:
		return types.NewInt(1), nil
	case ast.TokenTypeVoid:
		return types.Void, nil
	}
}
