package {
	public class Test {}
}

var x = {"key": "value"};

for (var name in x) {
	trace(name);
	trace(x[name]);
}