package {
    import flash.display.Sprite;
    import flash.utils.ByteArray;
    
    public class Tests extends Sprite {
        public function Tests() {
            testDelete(0);
            testDelete(1);
            testDelete(2);
            testDelete(3);
            testDelete(4);
            testDelete("0");
            testDelete("1");
            testDelete("2");
            testDelete("3");
            testDelete("4");
            testDelete("abcd");
            testDelete("0defg");
            testDelete("");
            testDelete(-1);
            testDelete(3.75);
        }

        internal function testDelete(index:*) : void {
            var b:ByteArray = new ByteArray();
            b[0] = 1;
            b[1] = 2;
            b[2] = 3;
            trace(delete b[index]);
            trace(b.length);

            var v:Vector.<uint> = new Vector.<uint>();
            v.push(1);
            v.push(2);
            v.push(3);
            trace(delete v[index]);
            trace(v);
            trace(v.length);
        }
    }
}

