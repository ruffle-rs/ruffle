var testCls = function() {
	trace("constructor called!");
};

var storedPrototype = testCls.prototype;
storedPrototype.name = "stored name";
var newPrototype = { name: "name by getter" };

testCls.addProperty("prototype", function() {
	trace("prototype getter called!");
	return newPrototype;
}, null);

trace("// Test.prototype.name");
trace(testCls.prototype.name);

trace("// (new Test()).name");
var obj = new testCls();
trace(obj.name);

fscommand("quit");
