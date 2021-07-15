package {
	public class Test {}
}

function testfunc(v1, v2, v3 = "Default string") {
	trace(v1);
	trace(v2);
	trace(v3);
}

function testfunc2(v1 = "Default string 0", v2 = "Default string 1", v3 = "Default string 2") {
	trace(v1);
	trace(v2);
	trace(v3);
}

testfunc("arg1", "arg2");

testfunc2("arg_again_1");