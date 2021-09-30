package {
	public class Test {}
}

trace("var qname_public = new QName(\"name\");");
var qname_public = new QName("name");

trace("///qname_public.valueOf()");
trace(qname_public.valueOf());

trace("var qname_scoped = new QName(\"https://ruffle.rs/AS3/tests/qname\", \"name\");");
var qname_scoped = new QName("https://ruffle.rs/AS3/tests/qname", "name");

trace("///qname_scoped.valueOf()");
trace(qname_scoped.valueOf());

trace("var qname_rescoped = new QName(\"https://ruffle.rs/AS3/tests/qname/2\", qname_scoped);");
var qname_rescoped = new QName("https://ruffle.rs/AS3/tests/qname/2", qname_scoped);

trace("///qname_rescoped.valueOf()");
trace(qname_rescoped.valueOf());

trace("var qname_clone = new QName(qname_scoped);");
var qname_clone = new QName(qname_scoped);

trace("///qname_clone.valueOf()");
trace(qname_clone.valueOf());

trace("var qname_null = new QName(null, \"name\");");
var qname_null = new QName(null, "name");

trace("///qname_null.valueOf()");
trace(qname_null.valueOf());