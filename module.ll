%Animal = type { %Animal_VTable_Type* }
%Animal_VTable_Type = type { i32 (%Dog*)* }
%Dog = type { i32, i32*, i32*, i32*, i32, i32 }

@Animal_VTable_Data = global %Animal_VTable_Type { i32 (%Dog*)* @Dog_Hello }

define i32 @Dog_Hello(%Dog* %dog) {
entry:
	ret i32 0
}

define i32 @main() {
entry:
	ret i32 0
}
