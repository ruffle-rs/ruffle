package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            // 1. Prototype own properties & enumerability (were false before PR)
            trace("XML.prototype.hasOwnProperty('hasOwnProperty'): " + XML.prototype.hasOwnProperty("hasOwnProperty"));
            trace("XML.prototype.hasOwnProperty('propertyIsEnumerable'): " + XML.prototype.hasOwnProperty("propertyIsEnumerable"));
            trace("XML.prototype.hasOwnProperty('valueOf'): " + XML.prototype.hasOwnProperty("valueOf"));
            trace("XML.prototype.propertyIsEnumerable('hasOwnProperty'): " + XML.prototype.propertyIsEnumerable("hasOwnProperty"));
            trace("XML.prototype.propertyIsEnumerable('propertyIsEnumerable'): " + XML.prototype.propertyIsEnumerable("propertyIsEnumerable"));
            trace("XML.prototype.propertyIsEnumerable('valueOf'): " + XML.prototype.propertyIsEnumerable("valueOf"));

            trace("XMLList.prototype.hasOwnProperty('hasOwnProperty'): " + XMLList.prototype.hasOwnProperty("hasOwnProperty"));
            trace("XMLList.prototype.hasOwnProperty('propertyIsEnumerable'): " + XMLList.prototype.hasOwnProperty("propertyIsEnumerable"));
            trace("XMLList.prototype.hasOwnProperty('valueOf'): " + XMLList.prototype.hasOwnProperty("valueOf"));
            trace("XMLList.prototype.propertyIsEnumerable('hasOwnProperty'): " + XMLList.prototype.propertyIsEnumerable("hasOwnProperty"));
            trace("XMLList.prototype.propertyIsEnumerable('propertyIsEnumerable'): " + XMLList.prototype.propertyIsEnumerable("propertyIsEnumerable"));
            trace("XMLList.prototype.propertyIsEnumerable('valueOf'): " + XMLList.prototype.propertyIsEnumerable("valueOf"));

            // Ensure prototype methods are not enumerated by for..in
            var enumKeys:Array = [];
            for (var k:String in XML.prototype) {
                enumKeys.push(k);
            }
            trace("XML.prototype enumKeys.length: " + enumKeys.length);

            var listEnumKeys:Array = [];
            for (var lk:String in XMLList.prototype) {
                listEnumKeys.push(lk);
            }
            trace("XMLList.prototype enumKeys.length: " + listEnumKeys.length);

            // 2. XML AS3:: and prototype calls
            var xml:XML = <root attr="val"><child>text</child></root>;
            trace("xml.AS3::hasOwnProperty('@attr'): " + xml.AS3::hasOwnProperty("@attr"));
            trace("xml.AS3::hasOwnProperty('child'): " + xml.AS3::hasOwnProperty("child"));
            trace("xml.AS3::hasOwnProperty('toXMLString'): " + xml.AS3::hasOwnProperty("toXMLString"));
            trace("xml.AS3::propertyIsEnumerable('@attr'): " + xml.AS3::propertyIsEnumerable("@attr"));
            trace("xml.AS3::propertyIsEnumerable('child'): " + xml.AS3::propertyIsEnumerable("child"));
            trace("xml.AS3::propertyIsEnumerable(0): " + xml.AS3::propertyIsEnumerable(0));
            trace("xml.AS3::propertyIsEnumerable('0'): " + xml.AS3::propertyIsEnumerable("0"));

            trace("XML.prototype.hasOwnProperty.call(xml, 'child'): " + XML.prototype.hasOwnProperty.call(xml, "child"));
            trace("XML.prototype.propertyIsEnumerable.call(xml, 0): " + XML.prototype.propertyIsEnumerable.call(xml, 0));
            trace("XML.prototype.valueOf.call(xml) === xml: " + (XML.prototype.valueOf.call(xml) === xml));

            // 3. XMLList AS3:: and prototype calls
            var list:XMLList = xml.children();
            trace("list.AS3::hasOwnProperty(0): " + list.AS3::hasOwnProperty(0));
            trace("list.AS3::hasOwnProperty('0'): " + list.AS3::hasOwnProperty("0"));
            trace("list.AS3::hasOwnProperty(1): " + list.AS3::hasOwnProperty(1));
            trace("list.AS3::propertyIsEnumerable(0): " + list.AS3::propertyIsEnumerable(0));
            trace("list.AS3::propertyIsEnumerable('0'): " + list.AS3::propertyIsEnumerable("0"));
            trace("list.AS3::propertyIsEnumerable(1): " + list.AS3::propertyIsEnumerable(1));

            trace("XMLList.prototype.hasOwnProperty.call(list, 0): " + XMLList.prototype.hasOwnProperty.call(list, 0));
            trace("XMLList.prototype.propertyIsEnumerable.call(list, 0): " + XMLList.prototype.propertyIsEnumerable.call(list, 0));
            trace("XMLList.prototype.valueOf.call(list) === list: " + (XMLList.prototype.valueOf.call(list) === list));
        }
    }
}
