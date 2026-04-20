// NOTE: The script initializer is p-code edited to contain the following
// before the compiler-generated initializer code:
/*
    pushbyte 4
    pushscope
    findpropstrict QName(PackageNamespace(""),"trace")
    getlex QName(PackageNamespace(""),"toExponential")
    callpropvoid QName(PackageNamespace(""),"trace"), 1
    popscope
*/

package {
    import flash.display.MovieClip;

    public class Test extends MovieClip {
        public function Test() {
            super();

            try {
                trace(toString);
                trace("no error");
            }
            catch(e:Error) {
                trace("error with id " + e.errorID);
            }
            try {
                trace(slage);
                trace("no error");
            }
            catch(e:Error) {
                trace("error with id " + e.errorID);
            }

            Object.prototype.slage = 99;

            try {
                trace(slage);
                trace("no error");
            }
            catch(e:Error) {
                trace("error with id " + e.errorID);
            }
        }
    }
}
