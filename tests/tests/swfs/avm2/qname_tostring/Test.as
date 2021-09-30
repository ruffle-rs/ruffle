package {
	public class Test {}
}

trace("var qname_public = new QName(\"name\");");
var qname_public = new QName("name");

trace("///qname_public.toString()");
trace(qname_public.toString());

trace("///Object.prototype.toString.call(qname_public)");
trace(Object.prototype.toString.call(qname_public));

trace("var qname_scoped = new QName(\"https://ruffle.rs/AS3/tests/qname\", \"name\");");
var qname_scoped = new QName("https://ruffle.rs/AS3/tests/qname", "name");

trace("///qname_scoped.toString()");
trace(qname_scoped.toString());

trace("///Object.prototype.toString.call(qname_scoped)");
trace(Object.prototype.toString.call(qname_scoped));

trace("var qname_rescoped = new QName(\"https://ruffle.rs/AS3/tests/qname/2\", qname_scoped);");
var qname_rescoped = new QName("https://ruffle.rs/AS3/tests/qname/2", qname_scoped);

trace("///qname_rescoped.toString()");
trace(qname_rescoped.toString());

trace("///Object.prototype.toString.call(qname_rescoped)");
trace(Object.prototype.toString.call(qname_rescoped));

trace("var qname_clone = new QName(qname_scoped);");
var qname_clone = new QName(qname_scoped);

trace("///qname_clone.toString()");
trace(qname_clone.toString());

trace("///Object.prototype.toString.call(qname_clone)");
trace(Object.prototype.toString.call(qname_clone));

trace("var qname_null = new QName(null, \"name\");");
var qname_null = new QName(null, "name");

trace("///qname_null.toString()");
trace(qname_null.toString());

trace("///Object.prototype.toString.call(qname_null)");
trace(Object.prototype.toString.call(qname_null));