// compiled with mxmlc

package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }

    }

}

import flash.utils.getQualifiedClassName;

trace("// uint doesn't exist")
trace(getQualifiedClassName(1 as uint));
trace((1 as uint) is uint);
trace(getQualifiedClassName(new uint()));
trace("// int overflow => Number")
trace(getQualifiedClassName(268435454));
trace(getQualifiedClassName(268435454 + 1));
trace(getQualifiedClassName(268435454 + 2));
trace("// int underflow => Number")
trace(getQualifiedClassName(-268435454));
trace(getQualifiedClassName(-268435454 - 1));
trace(getQualifiedClassName(-268435454 - 2));
trace(getQualifiedClassName(-268435454 - 3));
