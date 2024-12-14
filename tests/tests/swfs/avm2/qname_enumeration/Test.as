package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

trace("QName with namespace")
var q = new QName("http://someuri", "foo");
for(var p in q)
{
    trace(p);
}
for each (var v in q) {
    trace(v);
}

trace("QName without namespace")
var q = new QName("bar");
for(var p in q)
{
    trace(p);
}
for each (var v in q) {
    trace(v);
}
