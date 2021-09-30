package {
	public class Test {}
}

trace("namespace ns_public = \"\"");
namespace ns_public = "";

trace("var qname_public = new QName(ns_public, \"name\");");
var qname_public = new QName(ns_public, "name");

trace("///qname_public.localName");
trace(qname_public.localName);

trace("///qname_public.uri");
trace(qname_public.uri);

trace("namespace ns_ruffle = \"https://ruffle.rs/AS3/tests/qname\";");
namespace ns_ruffle = "https://ruffle.rs/AS3/tests/qname";

trace("var qname_scoped = new QName(ns_ruffle, \"name\");");
var qname_scoped = new QName(ns_ruffle, "name");

trace("///qname_scoped.localName");
trace(qname_scoped.localName);

trace("///qname_scoped.uri");
trace(qname_scoped.uri);

trace("var qname_rescoped = new QName(ns_ruffle, qname_public);");
var qname_rescoped = new QName(ns_ruffle, qname_public);

trace("///qname_rescoped.localName");
trace(qname_rescoped.localName);

trace("///qname_rescoped.uri");
trace(qname_rescoped.uri);