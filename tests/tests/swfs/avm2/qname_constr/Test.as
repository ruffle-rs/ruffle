package {
	public class Test {}
}

trace("var qname_public = new QName(\"name\");");
var qname_public = new QName("name");

trace("///qname_public.localName");
trace(qname_public.localName);

trace("///qname_public.uri");
trace(qname_public.uri);

trace("var qname_scoped = new QName(\"https://ruffle.rs/AS3/tests/qname\", \"name\");");
var qname_scoped = new QName("https://ruffle.rs/AS3/tests/qname", "name");

trace("///qname_scoped.localName");
trace(qname_scoped.localName);

trace("///qname_scoped.uri");
trace(qname_scoped.uri);

trace("var qname_rescoped = new QName(\"https://ruffle.rs/AS3/tests/qname/2\", qname_scoped);");
var qname_rescoped = new QName("https://ruffle.rs/AS3/tests/qname/2", qname_scoped);

trace("///qname_rescoped.localName");
trace(qname_rescoped.localName);

trace("///qname_rescoped.uri");
trace(qname_rescoped.uri);

trace("var qname_clone = new QName(qname_scoped);");
var qname_clone = new QName(qname_scoped);

trace("///qname_clone.localName");
trace(qname_clone.localName);

trace("///qname_clone.uri");
trace(qname_clone.uri);

trace("var qname_null = new QName(null, \"name\");");
var qname_null = new QName(null, "name");

trace("///qname_null.localName");
trace(qname_null.localName);

trace("///qname_null.uri");
trace(qname_null.uri);