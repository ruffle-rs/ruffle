package {
import flash.display.Sprite;

public class Test extends Sprite {}
}

var main:XML = <root><hello/></root>;
trace(main);

trace("Set XML as child");
var xmlChild:XML = <item><stuff>Hello</stuff><more><stuff/></more></item>;
main.p = xmlChild;
trace(main);

trace("Set XMLList as child");
var listChild:XMLList = new XMLList("<list><item>A</item><item>B</item><item>C</item></list>");
main.x = listChild;
trace(main);