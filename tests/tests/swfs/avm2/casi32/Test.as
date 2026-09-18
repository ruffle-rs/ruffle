package {
    import flash.display.Sprite;

    import avm2.intrinsics.memory.casi32;
    import flash.system.ApplicationDomain;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            var memory:ByteArray = new ByteArray();
            memory.length = 2048;
            ApplicationDomain.currentDomain.domainMemory = memory;

            tryCasi32(1, 0, 0);
            tryCasi32(2, 0, 0);
            tryCasi32(3, 0, 0);
            tryCasi32(13, 0, 0);
            tryCasi32(-50, 0, 0);
            tryCasi32(-1, 0, 0);
            tryCasi32(2048, 0, 0);
            tryCasi32(2047, 0, 0);
            tryCasi32(2046, 0, 0);
            tryCasi32(2045, 0, 0);
            tryCasi32(2044, 0, 0);

            tryCasi32(0, 0, 0);
            tryCasi32(4, 0, 5);
            tryCasi32(0, 0, 3);
            tryCasi32(4, 2, 6);
            tryCasi32(4, 5, 777);
            tryCasi32(0, 3, 122);
            tryCasi32(0, 1, 5);
            tryCasi32(0, 0, 0);
        }

        public function dumpMemory():void {
            trace("Domain memory: ");
            var arr:ByteArray = ApplicationDomain.currentDomain.domainMemory;
            for (var i:int = 0; i < 8; i ++) {
                trace("  " + i + " : " + arr[i]);
            }
        }
        
        public function tryCasi32(address:int, expected:int, update:int):void {
            var result:String = "casi32(" + address + ", " + expected + ", " + update + ") = ";
            var opResult:* = 0;
            try {
                opResult = casi32(address, expected, update);
            } catch(e:Error) {
                opResult = e.getStackTrace();
            }
            result += opResult;
            trace(result);

            if (address <= 4) {
                dumpMemory();
            }
        }
    }
}
