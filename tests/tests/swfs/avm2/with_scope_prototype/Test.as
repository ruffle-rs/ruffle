package {
    import flash.display.Sprite;
    public class Test extends Sprite {}
}

class C {
    {
        prototype.prototypeProperty = "prototype property";
        prototype.prototypeMethod = function():String { return "prototype method"; };
    }
    public var ownProperty:String = "own property";
}

var c:C = new C();
with (c) {
    trace(ownProperty);
    trace(prototypeProperty);
    trace(prototypeMethod());
}
