// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {

		trace("// var str = hello world");
		var str = "hello world";
		trace("// str.substr(0, -1)");
		trace(str.substr(0,-1));
		trace("// str.substr(0, -4)");
		trace(str.substr(0,-4));
		trace("// str.substr(8, -4)");
		trace(str.substr(8,-4));
		trace("// str.substr(3, -3)");
		trace(str.substr(3,-3));
		trace("// str.substr(3, -4)");
		trace(str.substr(3,-4));
		trace("// str.substr(3, -5)");
		trace(str.substr(3,-5));
		trace("// str.substr(4, -4)");
		trace(str.substr(4,-4));
		trace("// str.substr(4, -5)");
		trace(str.substr(4,-5));
		trace("// str.substr(null, -1)");
		trace(str.substr(null,-1));
		trace("// str.substr(undefined, -1)");
		trace(str.substr(undefined,-1));

		var text = "abcd";
		for (var i = -5; i < 5; i += 1) {
			trace("// text.substr(" + i + ")");
			trace(text.substr(i));
			for (var j = -5; j < 5; j += 1) {
				trace("// text.substr(" + i + "," + j + ")");
				trace(text.substr(i, j));
			}
		}
    }
}