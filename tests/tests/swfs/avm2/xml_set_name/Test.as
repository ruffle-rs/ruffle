package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            var a:XML = <outer attrib="value"><inner>innerText</inner></outer>;
            var b:XML = new XML("justText");
            trace(a.toXMLString());

            trace('// "newOuterName"');
            a.setName("newOuterName");
            trace(a.toXMLString());

            trace('// new QName(null,"someOuterName")');
            a.setName(new QName(null,"someOuterName"));
            trace(a.toXMLString());

            trace('// new QName("","otherOuterName")');
            a.setName(new QName("","otherOuterName"));
            trace(a.toXMLString());

            trace('// new QName("http://example.org", "nameWithNs")');
            a.setName(new QName("http://example.org", "nameWithNs"));
            trace(a.toXMLString());

            trace('// "simpleName"');
            a.setName("simpleName");
            trace(a.toXMLString());

            trace('// "newattribname" ');
            a.@attrib.setName("newattribname");
            trace(a.toXMLString());

            trace('// new QName("http://foo.bar", "otherattribname")');
            a.@newattribname.setName(new QName("http://foo.bar", "otherattribname"));
            trace(a.toXMLString());

            trace("// ")
            trace(b.toXMLString());
            b.setName("noeffect");
            trace(b.toXMLString());
        }
    }
}

