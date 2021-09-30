package {
	public class Test {}
}

trace("var qname_public = new QName(\"name\");");
var qname_public = new QName("name");

trace("///qname_public.valueOf()");
trace(qname_public.valueOf());

trace("///qname_public.valueOf().localName");
trace(qname_public.valueOf().localName);

trace("///Object.prototype.valueOf.call(qname_public)");
trace(Object.prototype.valueOf.call(qname_public));

trace("///Object.prototype.valueOf.call(qname_public).localName");
trace(Object.prototype.valueOf.call(qname_public).localName);

trace("var qname_scoped = new QName(\"https://ruffle.rs/AS3/tests/qname\", \"name\");");
var qname_scoped = new QName("https://ruffle.rs/AS3/tests/qname", "name");

trace("///qname_scoped.valueOf()");
trace(qname_scoped.valueOf());

trace("///Object.prototype.valueOf.call(qname_scoped)");
trace(Object.prototype.valueOf.call(qname_scoped));

trace("var qname_rescoped = new QName(\"https://ruffle.rs/AS3/tests/qname/2\", qname_scoped);");
var qname_rescoped = new QName("https://ruffle.rs/AS3/tests/qname/2", qname_scoped);

trace("///qname_rescoped.valueOf()");
trace(qname_rescoped.valueOf());

trace("///Object.prototype.valueOf.call(qname_rescoped)");
trace(Object.prototype.valueOf.call(qname_rescoped));

trace("var qname_clone = new QName(qname_scoped);");
var qname_clone = new QName(qname_scoped);

trace("///qname_clone.valueOf()");
trace(qname_clone.valueOf());

trace("///Object.prototype.valueOf.call(qname_clone)");
trace(Object.prototype.valueOf.call(qname_clone));

trace("var qname_null = new QName(null, \"name\");");
var qname_null = new QName(null, "name");

trace("///qname_null.valueOf()");
trace(qname_null.valueOf());

trace("///Object.prototype.valueOf.call(qname_null)");
trace(Object.prototype.valueOf.call(qname_null));