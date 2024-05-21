package {
    import flash.display.Sprite;

    public class Test extends Sprite {}
}

var namespace = new Namespace("p", "u");

trace("for in namespace");
for (var name in namespace) {
    trace(" " + name);
}

trace();

trace("for each in namespace");
for each (var value in namespace) {
    trace(" " + value);
}
