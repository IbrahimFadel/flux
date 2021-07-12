type Animal interface {
	Hello() -> i32
}

type Dog struct {
}

type Cat struct {
}

fn (dog Dog*) Hello() -> i32 {
	return 0
}

fn (cat Cat*) Hello() -> i32 {
	return 0
}

---------------------------------

%Animal = type { %Animal_VTable_Type* }
%Animal_VTable_Type = type { i32 (%Dog*)*, i32 (%Cat*)* }
%Dog = type { }
%Cat = type { }

@Animal_VTable_Data = global %Animal_VTable_Type { i32 (%Dog*)* @Dog_Hello, i32 (%Cat*)* @Cat_Hello }

define i32 @Dog_Hello(%Dog* %dog) {
entry:
	ret i32 0
}

define i32 @Cat_Hello(%Cat* %cat) {
entry:
	ret i32 0
}
