// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {

	var p = { x: 3 };
	p.addProperty("x", function() { trace("get x"); }, function(value) { trace("set x to " + value); });

	var o = { __proto__: p };
	o.watch("x", function(prop, old_val, new_val) { trace("old_val: " + old_val); return new_val; });
	o.x = 4;

    }
}
