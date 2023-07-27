package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {

        public function Test() {
            XML.prettyPrinting = false;
            var xml:XML = <a><child1/><child2/></a>;
            trace(xml);
            var xmls:XMLList = (<root><child3/><child4/></root>).children();
            var appended:XML = xml.appendChild(xmls);
            trace(appended);
            trace(xml);
            trace(xml == appended);
            xml.child3 = "4";
            trace(xmls);
            xml.appendChild("qwerty");
            trace(xml);
            trace(xml.appendChild({"key":"value"}));
            var xml2:XML = <b>abcd</b>;
            trace(xml2.appendChild("text1").toXMLString());
            xml2 = <b></b>;
            trace(xml2.appendChild("text2").toXMLString());
        }
    }
}

