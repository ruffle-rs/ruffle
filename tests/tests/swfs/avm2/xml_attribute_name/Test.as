package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            var xml = new XML('<foo bar="attribute"><bar>element</bar></foo>');

            // Test attribute name
            var name = xml.@bar.name();
            trace("name = xml.@bar.name: " + name);
            trace("xml[name]: " + xml[name]);
            xml[name] = "new attribute";
            trace("xml[name]: " + xml[name]);
            trace('xml["bar"]: ' + xml["bar"]);
            trace('xml["@bar"]: ' + xml["@bar"]);

            // Test element name
            var name2 = xml.bar.name();
            trace("name2 = xml.bar.name(): " + name2);
            trace("xml[name2]: " + xml[name2]);
            xml[name2] = "new element";
            trace("xml[name2]: " + xml[name2]);
            trace('xml["bar"]: ' + xml["bar"]);
            trace('xml["@bar"]: ' + xml["@bar"]);

            // Test creating attribute
            xml["@foo"] = "My new attr";
            trace('xml["@foo"]: ' + xml["@foo"]);
            trace(xml.toXMLString());

            trace("Has property: " + xml.hasOwnProperty("@foo"));

            delete xml["@foo"];
            trace('After delete: xml["@foo"] = ' + xml["@foo"]);
            trace(xml.toXMLString());

            var list = new XMLList(xml + xml);
            trace("List: " + list);
            trace('list["@bar"] = ' + list["@bar"]);
            trace(list);
            delete list["@bar"];
            trace("After delete:");
            trace('list["@bar"] = ' + list["@bar"]);
            trace(list);

        }
    }
}
