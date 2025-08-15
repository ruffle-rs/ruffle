// Compiled with mxmlc

package {
    import flash.display.Sprite;

    public class Test extends Sprite {
        public function Test() {
            super();
        }
    }
}



function test(name, f) {
    try {
        f();
        trace(name + " did not throw");
    } catch(e) {
        trace(name + " did throw " + e);
    }
}

function f1(){
    var a = 1; /* replace with (maxstack=1):
         pushbyte 1
         pushbyte 1
         pop
         pop
         returnvoid
    */
}
function f2(){
    var a = 1; /* replace with (maxstack=1):
         pushbyte 1
         jump asdf
asdf:
         pushbyte 1
         pop
         pop
         returnvoid
    */
}
function f3(){
    var a = 1; /* replace with (maxstack=1):
         pushbyte 1
         jump asdf
         pushbyte 2
asdf:
         pop
         returnvoid
    */
}
function f4(){
    var a = 1;
    try { var b = 1; } catch(e) { var c = 2; }
    /* replace with (maxstack=3):
a:
         pushbyte 1
         pop
b:
         returnvoid
exc:
         pushbyte 1
         pushbyte 1
         pushbyte 1
         pushbyte 1
         pop
         pop
         pop
         pop
         returnvoid
      end ; code
      try from a to b target exc type null name QName(PrivateNamespace("FilePrivateNS:Test"),"e") end
    */
}
function f5(){
    var a = 1;
    try { var b = 1; } catch(e) { var c = 2; }
    /* replace with (maxstack=3):
a:
         pushbyte 1
         increment
         pop
b:
         returnvoid
exc:
         pushbyte 1
         pushbyte 1
         pushbyte 1
         pushbyte 1
         pop
         pop
         pop
         pop
         returnvoid
      end ; code
      try from a to b target exc type null name QName(PrivateNamespace("FilePrivateNS:Test"),"e") end
    */
}
test("Overflow", f1);
test("Overflow after jump", f2);
test("Overflow in unreachable code", f3);
test("Overflow in unreachable exception", f4);
test("Overflow in reachable exception", f5);