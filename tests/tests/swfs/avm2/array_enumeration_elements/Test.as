package {
	public class Test {
	}
}

var a = [1, 2, 3, 4, 5];
a.elem = "test";
for each(var i in a) {
    trace(i)
}

trace(a.propertyIsEnumerable("elem"));
trace(a.propertyIsEnumerable("another"));
trace(a.propertyIsEnumerable("random"));
trace(a.propertyIsEnumerable("3"));
trace(a.propertyIsEnumerable("7"));