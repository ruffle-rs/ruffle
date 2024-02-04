package {
    import flash.display.Sprite;

    public class Test extends Sprite {}
}

var a = new XML('<item val="example"><a/></item>');
trace("@val in a");
trace("@val" in a);
trace("");

trace("val in a");
trace("val" in a);
trace("");

trace("item in a");
trace("item" in a);
trace("");

trace("@item in a");
trace("@item" in a);
trace("");

trace("a in a");
trace("a" in a);
trace("");

trace("@a in a");
trace("@a" in a);
trace("");

trace("0 in a");
trace(0 in a);
trace("");

trace("1 in a");
trace(1 in a);
trace("");

trace("propertyIsEnumerable in a");
trace("propertyIsEnumerable" in a);
