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