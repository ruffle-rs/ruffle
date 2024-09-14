/*
   Compiled with:
   java -jar utils/asc.jar -import playerglobal.abc -swf NetClasses,100,100,2 test/swfs/flash_net_classes.as

   This template is for writing SWFs using pure AS3. It allows for testing UI events, screen shots
   and trace log using the Shumway test harness.


*/

package {
    import flash.display.MovieClip;

    import flash.net.getClassByAlias;
    import flash.net.registerClassAlias;
    import flash.utils.ByteArray;


    public class NetClasses extends MovieClip {
        public function NetClasses() {
          addFrameScript(0, test);
        }

        private function test(): void {
          simpleTestNoReg();
          simpleTest();
          arrayTest();
        }

        private function simpleTestNoReg(): void {
            trace('Simple test (no registration)');

            var test: Test1 = new Test1();
            test.a = 5;

            var ba: ByteArray = new ByteArray();
            ba.writeObject(test);

            dumpHex(ba);

            ba.position = 0;
            var test2: Object = ba.readObject();
            trace('is Test1: ' + (test2 is Test1));
            trace('a: ' + test2.a);

            var thrown:Boolean = false;
            try {
              var c: Class = getClassByAlias('aliastest');
            } catch (ex: *) { thrown = true; }
            trace('aliastest thrown: ' + thrown);
        }

        private function dumpHex(ba: ByteArray):void {
            var dump: String = '';
            for (var i: int = 0; i < ba.length; i++) {
              dump += (ba[i] + 256).toString(16).substr(1);
            }
            trace('dump: ' + dump);
        }


        private function simpleTest(): void {
            trace('Simple test');

            var test: Test1 = new Test1();
            test.a = 5;

            registerClassAlias('aliastest', Test1);

            var ba: ByteArray = new ByteArray();
            ba.writeObject(test);

            dumpHex(ba);

            ba.position = 0;

            var test2: Object = ba.readObject();
            trace('is Test1: ' + (test2 is Test1));
            trace('a: ' + test2.a);

            var c: Class = getClassByAlias('aliastest');
            trace('aliastest: ' + (c === Test1));
        }

        private function arrayTest() : void {
            trace('Test with Test1/Test2 array');

            var test: Test1 = new Test1();
            test.a = 5;
            var test1: Test1 = new Test1();
            test1.a = 5;
            test1.b = 7;
            var test2: Test2 = new Test2();
            test2.b = 6;

            registerClassAlias('aliastest2', Test2);

            var ba: ByteArray = new ByteArray();
            ba.writeObject([test, test1, test, test2, test1, test2]);

            dumpHex(ba);

            ba.position = 0;
            var result: Object = ba.readObject();
            trace('length: ' + result.length);
            trace('[0] == [2]: ' + (result[0] === result[2]));
            trace('[0] != [1]: ' + (result[0] === result[1]));
            trace('[1] == [4]: ' + (result[1] === result[4]));
            trace('[3] == [5]: ' + (result[3] === result[5]));
            trace('[0] is Test1: ' + (result[0] is Test1));
            trace('[5] is Test2: ' + (result[5] is Test2));
            trace('[2].a: ' + result[2].a);
            trace('[4].b: ' + result[4].b);
            trace('[5].b: ' + result[5].b);
        }
    }
}

class Test2 {
  public var b: int;
}

dynamic class Test1 {
  public var a: int;
}

