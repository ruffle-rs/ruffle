package {
    public class Test {
        public function Test() {
            super();
            trace("//var a = new Array(5,\"abc\")");
            var a:Array = new Array(5,"abc");

            trace("//a.forEach(function (val) { ... }, a);");
            a.forEach(function(val:*, index:*, array:*):* {
                trace("//(in callback) this === a;");
                trace(this === a);
                trace("//val");
                trace(val);
                trace("//index");
                trace(index);
                trace("//array === a");
                trace(array === a);
            },a);

            var b:Array = new Array(1);
            trace("//b.forEach(function (...) { trace(this); }, [various values])");
            b.forEach(function(item:*, index:int, array:Array):* {
                trace("this = undefined");
                trace(this);
            },undefined);
            b.forEach(function(item:*, index:int, array:Array):* {
                trace("this = \"\"");
                trace(this);
            },"");
            b.forEach(function(item:*, index:int, array:Array):* {
                trace("this = null");
                trace(this);
            },null);
            b.forEach(function(item:*, index:int, array:Array):* {
                trace("this = this");
                trace(this);
            },this);
            b.forEach(function(item:*, index:int, array:Array):* {
                trace("this = 300");
                trace(this);
            },300);
        }
    }
}
