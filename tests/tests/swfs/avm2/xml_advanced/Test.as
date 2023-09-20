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

var list:XMLList = new XMLList();

trace("Assignment without target");
list.a = <item>ASdasdasd</item>;
trace(list);

trace("Assignment list without target");
list.a = <new_item>abcdefg</new_item>;
trace(list);

var list:XMLList = new XMLList(<root></root>);

trace("Assignment to init list without target");
list.a = <item>ASdasdasd</item>;
trace(list);

trace("Assignment to init list without target");
list.* = <new_item>abcdefg</new_item>;
trace(list);

// used to just create list with a target object.
var list:XMLList = list.children();

trace("Assignment to list with target");
trace(list);
list.a = <item>def</item>;
trace(list);

trace("Assignment to list with target");
list.a = <new_item>abc</new_item>;
trace(list);