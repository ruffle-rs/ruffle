package {
	public class Test {}
}
import flash.utils.Dictionary;

namespace test = "NS::test";
namespace test2 = "NS::test2";

class TestDictionary extends Dictionary {
	test var test1 = "";
	
	test2 var test1 = "";
}

trace("///var dict = new TestDictionary()");
var dict = new TestDictionary();

trace("///var test1 = {\"toString\": function() { return \"test1\"; }};");
var test1 = {"toString": function(){
	return "test1";
}};

trace("///dict.test::[test1] = \"Test Object from test ns\"");
dict.test::[test1] = "Test Object from test ns";
trace("///dict.test2::[test1] = \"Test Object from test2 ns\"");
dict.test2::[test1] = "Test Object from test2 ns";

trace("///dict[test1]");
trace(dict[test1]);
trace("///dict.test::[test1]");
trace(dict.test::[test1]);
trace("///dict.test2::[test1]");
trace(dict.test2::[test1]);

trace("///dict.test::[\"test1\"]");
trace(dict.test::["test1"]);
trace("///dict.test2::[\"test1\"]");
trace(dict.test2::["test1"]);

trace("///dict[test1] = \"Test Object from public ns\"");
dict[test1] = "Test Object from public ns";

trace("///dict[test1]");
trace(dict[test1]);
trace("///dict.test::[test1]");
trace(dict.test::[test1]);
trace("///dict.test2::[test1]");
trace(dict.test2::[test1]);

trace("///dict.test::[\"test1\"]");
trace(dict.test::["test1"]);
trace("///dict.test2::[\"test1\"]");
trace(dict.test2::["test1"]);

trace("///delete dict.test::[test1]");
trace(delete dict.test::[test1]);

trace("///dict[test1]");
trace(dict[test1]);
trace("///dict.test::[test1]");
trace(dict.test::[test1]);
trace("///dict.test2::[test1]");
trace(dict.test2::[test1]);

trace("///dict.test::[\"test1\"]");
trace(dict.test::["test1"]);
trace("///dict.test2::[\"test1\"]");
trace(dict.test2::["test1"]);
