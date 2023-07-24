package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

function f():void{};

f.prototype = null;

trace(f.prototype); // undefined

var x = new f();

trace(f.prototype); // [object Object]
trace(x.prototype); // undefined

trace(x.valueOf()); // [object Object]
trace(x.toString()); // [object Object]

trace(x); // [object Object]
trace(x.asdf); // undefined