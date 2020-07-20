package {
	public class Test {
	}
}

function Test2() {
	trace("Instance constructor");
}

Test2.prototype.method = function() {
	trace("Instance method");
}

Test2.prototype.method2 = function() {
	trace("Instance method 2");
}

function Test3() {
	trace("Child instance constructor pre-super");
	Test2.call(this);
	trace("Child instance constructor post-super");
}

Test3.prototype = new Test2();

Test3.prototype.method = function() {
	trace("Child instance method pre-super");
	Test2.prototype.method.call(this);
	trace("Child instance method post-super");
}

Test3.prototype.method3 = function() {
	trace("Child instance method3 pre-super");
	Test2.prototype.method.call(this);
	trace("Child instance method3 post-super");
}

function Test4() {
	trace("Grandchild instance constructor pre-super");
	Test3.call(this);
	trace("Grandchild instance constructor post-super");
}

Test4.prototype = new Test3();

Test4.prototype.method2 = function () {
	trace("Grandchild instance method2 pre-super");
	Test3.prototype.method2.call(this);
	trace("Grandchild instance method2 post-super");
}

Test4.prototype.method3 = function () {
	trace("Grandchild instance method3 pre-super");
	Test3.prototype.method3.call(this);
	trace("Grandchild instance method3 post-super");
}

trace("Script initializer");
var x = new Test3();
x.method();
x.method2();
x.method3();

var y = new Test4();
y.method();
y.method2();
y.method3();