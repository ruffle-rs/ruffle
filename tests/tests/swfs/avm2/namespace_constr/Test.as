package {
	public class Test {}
}
var n = new Namespace();

trace("new Namespace()", n);

n = new Namespace("prefix", "uri");

trace("new Namespace(\"prefix\", \"uri\")", n);

n = new Namespace("uri");

trace("new Namespace(\"uri\")", n);

var q = new QName("uri", "test");
trace("var q = new QName(\"test\");");

n = new Namespace(q);

trace("new Namespace(q)", n);