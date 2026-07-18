obj_1 = { valueOf:function() {
   trace("OBJ_1");
   return 1;
}};
obj_2 = { valueOf:function() {
   trace("OBJ_2");
   return "4";
}};

// We want to test the `Add` opcode specifically, and we can't just wrap the opcode in a function as we test in swf4 which doesn't have functions

trace("// 'ab' + 'cd'");
@PCode {
	Push "a", "ab", "cd"
	Add
	SetVariable
}
trace(a);
trace("");

trace("// 300 + '150' + true");
@PCode {
	Push "a", 300, 150
	Add
	Push true
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + '150a'");
@PCode {
	Push "a", "300", "150a"
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + '0x96' + '010'");
@PCode {
	Push "a", "300", "0x96"
	Add
	Push "010"
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + undefined");
@PCode {
	Push "a", "300", undefined
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + null");
@PCode {
	Push "a", "300", null
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + NaN");
@PCode {
	Push "a", "300", NaN
	Add
	SetVariable
}
trace(a);
trace("");

trace("// '300' + Infinity");
@PCode {
	Push "a", "300", Infinity
	Add
	SetVariable
}
trace(a);
trace("");

trace("// obj_1 + obj_2");
@PCode {
	Push "a", "obj_1"
	GetVariable
	Push "obj_2"
    GetVariable
	Add
	SetVariable
}
trace(a);

fscommand("quit");
