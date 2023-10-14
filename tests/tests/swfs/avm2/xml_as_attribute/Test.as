package
{
    import flash.display.Sprite;
    public class Test extends Sprite
    {
    }
}

var main:XML = <root><item>A</item></root>;

var part:XML = <hello><a>B</a><b>C</b></hello>;

trace(main);

main.item[0].@name = "Hello";

trace(main);

main.item[0].@name2 = part;

trace(main);
