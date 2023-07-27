package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

XML.prettyPrinting = false;

trace("XMLList + XMLList");

var test:XML = <root>
                <list1>
                    <item>
                        <text>Hello</text>
                    </item>
                    <item>
                        <text>World</text>
                    </item>
                </list1>
                <list2>
                    <item>
                        <text>from</text>
                    </item>
                    <item>
                        <text>Ruffle!</text>
                    </item>
                </list2>
               </root>;

var list:XMLList = test.list1.item + test.list2.item;

trace(list);

var xml: XML = new XML("<a><b>A</b></a>");
var list: XMLList = new XMLList("<c><d>B</d></c>");

trace("XML + XMLList");
trace(xml + list);

trace("XMLList + XML");
trace(list + xml);

trace("XML + XML");
var xml2: XML = new XML("<c>D</c>");

trace(xml2 + xml);
