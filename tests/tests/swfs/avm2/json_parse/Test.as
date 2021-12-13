package  {
	
	public class Test {

	}
	
}

var INPUT = '{"test": "value", "another": [1, 2, 3], "example": {"recursive": "test"}}';
var parsed = JSON.parse(INPUT);
trace(parsed.test, parsed.another, parsed.example, parsed.example.recursive);

trace("// Parse with reviver")
var parsed = JSON.parse(INPUT, function(k, v) {
		trace(k, v);
		return v;
	});

trace(parsed.test, parsed.another, parsed.example, parsed.example.recursive);

trace("// Parse with custom reviver")
var parsed = JSON.parse(INPUT, function(k, v) {
		trace(k, v);
		if (v is int) {
			return "custom";
		}
		return v;
	});

trace(parsed.test, parsed.another, parsed.example, parsed.example.recursive);