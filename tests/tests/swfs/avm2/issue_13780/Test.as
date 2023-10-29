package {
    import flash.display.Sprite;

    public class Test extends Sprite {}
}

// Issue 13780: Replace does not work with attribute nodes.

var xml = new XML('<val attr="val"/>');
var attr = xml.@attr[0]; // Fooled by XMLList again...
trace(attr);
trace(attr.nodeKind());

var target = new XML('<root><child1/><child2/><child3/></root>');

trace(target);
target.replace(1, attr);
trace(target);