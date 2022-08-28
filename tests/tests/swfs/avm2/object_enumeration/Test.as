package {
	public class Test {}
}

var x = {"key": "value", "key2": "value2"};

for (var name in x) {
	trace(name);
	trace(x[name]);
}

trace("Delete key2");
delete x["key2"];

for (var name in x) {
	trace(name);
	trace(x[name]);
}