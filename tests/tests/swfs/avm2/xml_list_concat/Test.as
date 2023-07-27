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

var out:XMLList = xml + list;
trace(out);
trace(out[0].parent());
trace(out[1].parent());

trace("XMLList + XML");
var out:XMLList = list + xml;
trace(out);
trace(out[0].parent());
trace(out[1].parent());

trace("XML + XML");
var xml2: XML = new XML("<c>D</c>");
var out:XMLList = xml2 + xml;
trace(out);
trace(out[0].parent());
trace(out[1].parent());

