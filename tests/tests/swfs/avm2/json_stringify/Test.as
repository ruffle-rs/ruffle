package  {
	
	public class Test {
	}
	
}

class SealedClass {
	public var prop1: String;
	var privateProp: String = "Hidden";
	public var prop2: Boolean;
	public const MY_CONST: String = "Const val";
	
	function SealedClass(prop1: String, prop2: Boolean) {
		this.prop1 = prop1;
		this.prop2 = prop2;
	}

	public function get myGetter():String {
		return "Getter value";
	}
}

dynamic class DynamicClass {
	public var prop1: String;
	var privateProp: String = "Hidden";
	public var prop2: Boolean;
	
	function DynamicClass(prop1: String, prop2: Boolean) {
		this.prop1 = prop1;
		this.prop2 = prop2;
	}	
}

var obj = {toJSON: function () {
		return {e: "test"};
}}

var test = {
	a: "b",
	c: 2,
	d: [1, 2, 3],
	e: {
		f: undefined,
		g: null,
		h: "i"
	},
	j: obj,
	k: 5.3
}

function get_props(str) {
	var parsed = JSON.parse(str);
	with (parsed) {
			trace(a, c, d, e, e.f, e.g, e.h, j, j.e, k);
	}
	trace(str.length);
}

get_props(JSON.stringify(test))

get_props(JSON.stringify(test, function(k, v) {
	if (v == "b" || v == "i") {
		return "replacement";
	}
	return v;
}));



get_props(JSON.stringify(test, null, 1));

get_props(JSON.stringify(test, null, 20));

trace(JSON.stringify(test, null, "custom").length);
trace(JSON.stringify(test, ["a", "e", "f"]).length);

var sealed = new SealedClass("Hello", true);

trace("WARNING: The output.txt file has been hand-edited to match Ruffle's output, since we don't match Flash's serialization order")
trace(JSON.stringify(sealed));

var dynamicObj = new DynamicClass("Dynamic", false);
dynamicObj["dyn1"] = "Dyn prop";
dynamicObj["dyn2"] = 25;

dynamicObj.setPropertyIsEnumerable("dyn1", true);
dynamicObj.setPropertyIsEnumerable("dyn2", false);

trace("WARNING: The output.txt file has been hand-edited to match Ruffle's output, since we don't match Flash's serialization order")
trace(JSON.stringify(dynamicObj));