package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            var a:XML = <outer attrib="value"><inner>innerText</inner></outer>;
            var b:XML = new XML("justText");
            trace(a.toXMLString());
            a.setName("newOuterName");
            trace(a.toXMLString());
            a.setName(new QName(null,"someOuterName"));
            trace(a.toXMLString());
            a.setName(new QName("","otherOuterName"));
            trace(a.toXMLString());
            a.@attrib.setName("newattribname");
            trace(a.toXMLString());
            trace(b.toXMLString());
            b.setName("noeffect");
            trace(b.toXMLString());
        }
    }
}

