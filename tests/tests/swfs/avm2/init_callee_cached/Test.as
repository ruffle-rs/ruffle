package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        var count:int;
        var savedCallee:Function;

        public function Test() {
            trace("// Entered instance init");
            trace("this.count ++;");
            this.count ++;
            trace("// this.count is " + this.count);
            trace("// arguments.callee is " + arguments.callee);
            trace("// this.savedCallee is " + this.savedCallee);
            trace("// arguments.callee == this.savedCallee? " + (arguments.callee === this.savedCallee));
            trace("this.count = arguments.callee;");
            this.savedCallee = arguments.callee;

            if (this.count >= 3) {
                trace("// this.count >= 3, stopping");
            } else {
                trace("this.callInitAgain();");
                this.callInitAgain();
            }
        }

        public function callInitAgain():void {
            // Written in p-code, uses the callStatic op to invoke the instance
            // init method
        }
    }
}
