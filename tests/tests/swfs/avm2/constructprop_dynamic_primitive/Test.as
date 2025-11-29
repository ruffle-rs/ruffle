package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            super();

            Number.prototype.abc = 99;
            Number.prototype.def = new Function();
            Number.prototype.ghi = null;

            var n:Number = 19;

            try {
                new (n.abc)();
                trace("no error");
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }

            try {
                new (n.def)();
                trace("no error");
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }

            try {
                new (n.ghi)();
                trace("no error");
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }

            try {
                new (n.jkl)();
                trace("no error");
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }
        }
    }
}
