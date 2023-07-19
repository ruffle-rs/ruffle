package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

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

for (var i:int = 0; i < list.length(); i++)
{
    var val:XML = list[i];

    trace(val.text[0].toString());
}
