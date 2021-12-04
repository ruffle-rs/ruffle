package  {
	
	public class Test {
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

trace(JSON.stringify(test).length)

trace(JSON.stringify(test, function(k, v) {
	if (v == "b" || v == "i") {
		return "replacement";
	}
	return v;
}).length);

trace(JSON.stringify(test, ["a", "e", "f"]).length);

trace(JSON.stringify(test, null, 1).length);

trace(JSON.stringify(test, null, 20).length);

trace(JSON.stringify(test, null, "custom").length);