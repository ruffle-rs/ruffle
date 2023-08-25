package
{
    import flash.display.Sprite;
    public class Test extends Sprite
    {
    }
}

var main:XML = <root><item>A</item></root>;

var part:XMLList = new XMLList("<root><item>A</item><item>B</item></root>");

trace(main);

main.item[0].@name = "Hello";

trace(main);

main.item[0].@name2 = part;

trace(main);
