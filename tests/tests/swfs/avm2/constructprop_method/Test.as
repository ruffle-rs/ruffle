package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            super();

            try {
                new (this.abc)();
            } catch(e:Error) {
                trace(Object.prototype.toString.call(e));
                trace(e.errorID);
            }
        }

        public function abc():void {
        }
    }
}
