package {
	public class Test {}
}

dynamic class ES4Class {
	public var variable = "TEST FAIL: Trait properties are NOT enumerable! (var)";
	public const constant = "TEST FAIL: Trait properties are NOT enumerable! (const)";
}

var x = new ES4Class();
x.dynamic_variable = "variable value";
ES4Class.prototype.dynamic_prototype_variable = "prototype value";

for (var name in x) {
	trace(name);
	trace(x[name]);
}