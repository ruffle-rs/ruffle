package {
    import flash.display.Sprite;
    import flash.geom.*;
    import flash.utils.ByteArray;

    public class Test extends Sprite {
        public function Test() {
            testBasic();
            testExponentialEdges();
            testExponent308();
            testSweepNormal();
            testSweepSubnormal();
            testEdgeCasesNormal();
        }

        function testBasic() {
            trace("testBasic");
            test(0.1);
            test(0.5);
            test(0.2);
            test(1.5);
            test(2.0);
            test(10.0);
            test(100.25);
            test(0.25);
            test(0.125);
            test(1.0 / 3.0);
            test(2.0 / 3.0);
            test(3.14159265358979);
            test(123456789.0);
            test(1e10);
            test(1e-3);
            test(1000000.0);
            test(0.0001);
            test(Math.sqrt(2));
            test(-0.0);
            test(2.220446049250313e-16);
        }

        function testExponentialEdges() {
            trace("testExponentialEdges");
            var numbers = [
                0.000001,
                0.0000009999999999999997,
                0.0000009999999999999996,
                0.0000009999999999999993,
                0.0000009999999999999991,
                0.000000999999999999999,
                0.0000009999999999999987,
                0.0000009999999999999984,
                0.0000009999999999999982,
                0.000000999999999999998,
                999999999999999900000.0,
                1000000000000000000000.0,
                1000000000000000100000.0,
                1000000000000000200000.0,
                1000000000000009800000.0,
                1000000000000010000000.0,
                1000000000000099900000.0,
                1000000000000100000000.0,
                1000000000000100100000.0,
                1000000000000999800000.0,
                1000000000001000000000.0,
                1000000000001000100000.0,
                1000000000001009900000.0,
                1000000000001010000000.0,
            ];

            for each (var n in numbers) {
                test(n);
            }

            for each (var n in numbers) {
                test(-n);
            }
        }

        function testExponent308() {
            test(fromBits(0x7fcfffff, 0xfffffffc));
            test(fromBits(0x7fcfffff, 0xfffffffd));
            test(fromBits(0x7fcfffff, 0xfffffffe));
            test(fromBits(0x7fcfffff, 0xffffffff));
            test(fromBits(0x7fd00000, 0x00000000));
            test(fromBits(0x7fd00000, 0x00000001));
            test(fromBits(0x7fd00000, 0x00000002));
            test(fromBits(0x7fd00000, 0x00000003));
            test(fromBits(0x7fd00000, 0x00000004));

            test(fromBits(0x7fdfffff, 0xfffffffc));
            test(fromBits(0x7fdfffff, 0xfffffffd));
            test(fromBits(0x7fdfffff, 0xfffffffe));
            test(fromBits(0x7fdfffff, 0xffffffff));
            test(fromBits(0x7fe00000, 0x00000000));
            test(fromBits(0x7fe00000, 0x00000001));
            test(fromBits(0x7fe00000, 0x00000002));
            test(fromBits(0x7fe00000, 0x00000003));
            test(fromBits(0x7fe00000, 0x00000004));

            test(-fromBits(0x7fcfffff, 0xfffffffc));
            test(-fromBits(0x7fcfffff, 0xfffffffd));
            test(-fromBits(0x7fcfffff, 0xfffffffe));
            test(-fromBits(0x7fcfffff, 0xffffffff));
            test(-fromBits(0x7fd00000, 0x00000000));
            test(-fromBits(0x7fd00000, 0x00000001));
            test(-fromBits(0x7fd00000, 0x00000002));
            test(-fromBits(0x7fd00000, 0x00000003));
            test(-fromBits(0x7fd00000, 0x00000004));

            test(-fromBits(0x7fdfffff, 0xfffffffc));
            test(-fromBits(0x7fdfffff, 0xfffffffd));
            test(-fromBits(0x7fdfffff, 0xfffffffe));
            test(-fromBits(0x7fdfffff, 0xffffffff));
            test(-fromBits(0x7fe00000, 0x00000000));
            test(-fromBits(0x7fe00000, 0x00000001));
            test(-fromBits(0x7fe00000, 0x00000002));
            test(-fromBits(0x7fe00000, 0x00000003));
            test(-fromBits(0x7fe00000, 0x00000004));
        }

        function testSweepNormal() {
            trace("testSweepNormal");
            test(fromBits(0x0015c85c, 0x97cb3127));
            test(fromBits(0x001779b9, 0x7f4a7c15));
            test(fromBits(0x0175c85c, 0x97cb3127));
            test(fromBits(0x017779b9, 0x7f4a7c15));
            test(fromBits(0x07b5c85c, 0x97cb3127));
            test(fromBits(0x07b779b9, 0x7f4a7c15));
            test(fromBits(0x0df5c85c, 0x97cb3127));
            test(fromBits(0x0df779b9, 0x7f4a7c15));
            test(fromBits(0x1435c85c, 0x97cb3127));
            test(fromBits(0x143779b9, 0x7f4a7c15));
            test(fromBits(0x1a75c85c, 0x97cb3127));
            test(fromBits(0x1a7779b9, 0x7f4a7c15));
            test(fromBits(0x20b5c85c, 0x97cb3127));
            test(fromBits(0x20b779b9, 0x7f4a7c15));
            test(fromBits(0x26f5c85c, 0x97cb3127));
            test(fromBits(0x26f779b9, 0x7f4a7c15));
            test(fromBits(0x2d35c85c, 0x97cb3127));
            test(fromBits(0x2d3779b9, 0x7f4a7c15));
            test(fromBits(0x3055c85c, 0x97cb3127));
            test(fromBits(0x305779b9, 0x7f4a7c15));
            test(fromBits(0x3375c85c, 0x97cb3127));
            test(fromBits(0x337779b9, 0x7f4a7c15));
            test(fromBits(0x3695c85c, 0x97cb3127));
            test(fromBits(0x369779b9, 0x7f4a7c15));
            test(fromBits(0x39b5c85c, 0x97cb3127));
            test(fromBits(0x39b779b9, 0x7f4a7c15));
            test(fromBits(0x3b95c85c, 0x97cb3127));
            test(fromBits(0x3b9779b9, 0x7f4a7c15));
            test(fromBits(0x3cd5c85c, 0x97cb3127));
            test(fromBits(0x3cd779b9, 0x7f4a7c15));
            test(fromBits(0x3e15c85c, 0x97cb3127));
            test(fromBits(0x3e1779b9, 0x7f4a7c15));
            test(fromBits(0x3eb5c85c, 0x97cb3127));
            test(fromBits(0x3eb779b9, 0x7f4a7c15));
            test(fromBits(0x3f05c85c, 0x97cb3127));
            test(fromBits(0x3f0779b9, 0x7f4a7c15));
            test(fromBits(0x3f55c85c, 0x97cb3127));
            test(fromBits(0x3f5779b9, 0x7f4a7c15));
            test(fromBits(0x3f85c85c, 0x97cb3127));
            test(fromBits(0x3f8779b9, 0x7f4a7c15));
            test(fromBits(0x3f95c85c, 0x97cb3127));
            test(fromBits(0x3f9779b9, 0x7f4a7c15));
            test(fromBits(0x3fa5c85c, 0x97cb3127));
            test(fromBits(0x3fa779b9, 0x7f4a7c15));
            test(fromBits(0x3fb5c85c, 0x97cb3127));
            test(fromBits(0x3fb779b9, 0x7f4a7c15));
            test(fromBits(0x3fc5c85c, 0x97cb3127));
            test(fromBits(0x3fc779b9, 0x7f4a7c15));
            test(fromBits(0x3fd5c85c, 0x97cb3127));
            test(fromBits(0x3fd779b9, 0x7f4a7c15));
            test(fromBits(0x3fe5c85c, 0x97cb3127));
            test(fromBits(0x3fe779b9, 0x7f4a7c15));
            test(fromBits(0x3ff5c85c, 0x97cb3127));
            test(fromBits(0x3ff779b9, 0x7f4a7c15));
            test(fromBits(0x4005c85c, 0x97cb3127));
            test(fromBits(0x400779b9, 0x7f4a7c15));
            test(fromBits(0x4015c85c, 0x97cb3127));
            test(fromBits(0x401779b9, 0x7f4a7c15));
            test(fromBits(0x4025c85c, 0x97cb3127));
            test(fromBits(0x402779b9, 0x7f4a7c15));
            test(fromBits(0x4035c85c, 0x97cb3127));
            test(fromBits(0x403779b9, 0x7f4a7c15));
            test(fromBits(0x4045c85c, 0x97cb3127));
            test(fromBits(0x404779b9, 0x7f4a7c15));
            test(fromBits(0x4055c85c, 0x97cb3127));
            test(fromBits(0x405779b9, 0x7f4a7c15));
            test(fromBits(0x4065c85c, 0x97cb3127));
            test(fromBits(0x406779b9, 0x7f4a7c15));
            test(fromBits(0x4095c85c, 0x97cb3127));
            test(fromBits(0x409779b9, 0x7f4a7c15));
            test(fromBits(0x40e5c85c, 0x97cb3127));
            test(fromBits(0x40e779b9, 0x7f4a7c15));
            test(fromBits(0x4135c85c, 0x97cb3127));
            test(fromBits(0x413779b9, 0x7f4a7c15));
            test(fromBits(0x4145c85c, 0x97cb3127));
            test(fromBits(0x414779b9, 0x7f4a7c15));
            test(fromBits(0x4155c85c, 0x97cb3127));
            test(fromBits(0x415779b9, 0x7f4a7c15));
            test(fromBits(0x4185c85c, 0x97cb3127));
            test(fromBits(0x418779b9, 0x7f4a7c15));
            test(fromBits(0x41d5c85c, 0x97cb3127));
            test(fromBits(0x41d779b9, 0x7f4a7c15));
            test(fromBits(0x4315c85c, 0x97cb3127));
            test(fromBits(0x431779b9, 0x7f4a7c15));
            test(fromBits(0x4455c85c, 0x97cb3127));
            test(fromBits(0x445779b9, 0x7f4a7c15));
            test(fromBits(0x4635c85c, 0x97cb3127));
            test(fromBits(0x463779b9, 0x7f4a7c15));
            test(fromBits(0x4955c85c, 0x97cb3127));
            test(fromBits(0x495779b9, 0x7f4a7c15));
            test(fromBits(0x4c75c85c, 0x97cb3127));
            test(fromBits(0x4c7779b9, 0x7f4a7c15));
            test(fromBits(0x4f95c85c, 0x97cb3127));
            test(fromBits(0x4f9779b9, 0x7f4a7c15));
            test(fromBits(0x52b5c85c, 0x97cb3127));
            test(fromBits(0x52b779b9, 0x7f4a7c15));
            test(fromBits(0x58f5c85c, 0x97cb3127));
            test(fromBits(0x58f779b9, 0x7f4a7c15));
            test(fromBits(0x5f35c85c, 0x97cb3127));
            test(fromBits(0x5f3779b9, 0x7f4a7c15));
            test(fromBits(0x6575c85c, 0x97cb3127));
            test(fromBits(0x657779b9, 0x7f4a7c15));
            test(fromBits(0x6bb5c85c, 0x97cb3127));
            test(fromBits(0x6bb779b9, 0x7f4a7c15));
            test(fromBits(0x71f5c85c, 0x97cb3127));
            test(fromBits(0x71f779b9, 0x7f4a7c15));
            test(fromBits(0x7835c85c, 0x97cb3127));
            test(fromBits(0x783779b9, 0x7f4a7c15));
            test(fromBits(0x7e75c85c, 0x97cb3127));
            test(fromBits(0x7e7779b9, 0x7f4a7c15));
            test(fromBits(0x7fe5c85c, 0x97cb3127));
            test(fromBits(0x7fe779b9, 0x7f4a7c15));
        }

        function testSweepSubnormal() {
            trace("testSweepSubnormal");
            test(fromBits(0x000dc85c, 0x97cb3127));
            test(fromBits(0x000f79b9, 0x7f4a7c15));
            test(fromBits(0x0005c85c, 0x97cb3127));
            test(fromBits(0x000779b9, 0x7f4a7c15));
            test(fromBits(0x0003c85c, 0x97cb3127));
            test(fromBits(0x000379b9, 0x7f4a7c15));
            test(fromBits(0x0001c85c, 0x97cb3127));
            test(fromBits(0x000179b9, 0x7f4a7c15));
            test(fromBits(0x0000c85c, 0x97cb3127));
            test(fromBits(0x0000f9b9, 0x7f4a7c15));
            test(fromBits(0x0000485c, 0x97cb3127));
            test(fromBits(0x000079b9, 0x7f4a7c15));
            test(fromBits(0x0000285c, 0x97cb3127));
            test(fromBits(0x000039b9, 0x7f4a7c15));
            test(fromBits(0x0000185c, 0x97cb3127));
            test(fromBits(0x000019b9, 0x7f4a7c15));
            test(fromBits(0x0000085c, 0x97cb3127));
            test(fromBits(0x000009b9, 0x7f4a7c15));
            test(fromBits(0x0000045c, 0x97cb3127));
            test(fromBits(0x000005b9, 0x7f4a7c15));
            test(fromBits(0x0000025c, 0x97cb3127));
            test(fromBits(0x000003b9, 0x7f4a7c15));
            test(fromBits(0x0000015c, 0x97cb3127));
            test(fromBits(0x000001b9, 0x7f4a7c15));
            test(fromBits(0x000000dc, 0x97cb3127));
            test(fromBits(0x000000b9, 0x7f4a7c15));
            test(fromBits(0x0000005c, 0x97cb3127));
            test(fromBits(0x00000079, 0x7f4a7c15));
            test(fromBits(0x0000003c, 0x97cb3127));
            test(fromBits(0x00000039, 0x7f4a7c15));
            test(fromBits(0x0000001c, 0x97cb3127));
            test(fromBits(0x00000019, 0x7f4a7c15));
            test(fromBits(0x0000000c, 0x97cb3127));
            test(fromBits(0x00000009, 0x7f4a7c15));
            test(fromBits(0x00000004, 0x97cb3127));
            test(fromBits(0x00000005, 0x7f4a7c15));
            test(fromBits(0x00000002, 0x97cb3127));
            test(fromBits(0x00000003, 0x7f4a7c15));
            test(fromBits(0x00000001, 0x97cb3127));
            test(fromBits(0x00000001, 0x7f4a7c15));
            test(fromBits(0x00000000, 0x97cb3127));
            test(fromBits(0x00000000, 0xff4a7c15));
            test(fromBits(0x00000000, 0x57cb3127));
            test(fromBits(0x00000000, 0x7f4a7c15));
            test(fromBits(0x00000000, 0x37cb3127));
            test(fromBits(0x00000000, 0x3f4a7c15));
            test(fromBits(0x00000000, 0x17cb3127));
            test(fromBits(0x00000000, 0x1f4a7c15));
            test(fromBits(0x00000000, 0x0fcb3127));
            test(fromBits(0x00000000, 0x0f4a7c15));
            test(fromBits(0x00000000, 0x07cb3127));
            test(fromBits(0x00000000, 0x074a7c15));
            test(fromBits(0x00000000, 0x03cb3127));
            test(fromBits(0x00000000, 0x034a7c15));
            test(fromBits(0x00000000, 0x01cb3127));
            test(fromBits(0x00000000, 0x014a7c15));
            test(fromBits(0x00000000, 0x00cb3127));
            test(fromBits(0x00000000, 0x00ca7c15));
            test(fromBits(0x00000000, 0x004b3127));
            test(fromBits(0x00000000, 0x004a7c15));
            test(fromBits(0x00000000, 0x002b3127));
            test(fromBits(0x00000000, 0x002a7c15));
            test(fromBits(0x00000000, 0x001b3127));
            test(fromBits(0x00000000, 0x001a7c15));
            test(fromBits(0x00000000, 0x000b3127));
            test(fromBits(0x00000000, 0x000a7c15));
            test(fromBits(0x00000000, 0x00073127));
            test(fromBits(0x00000000, 0x00067c15));
            test(fromBits(0x00000000, 0x00033127));
            test(fromBits(0x00000000, 0x00027c15));
            test(fromBits(0x00000000, 0x00013127));
            test(fromBits(0x00000000, 0x00017c15));
            test(fromBits(0x00000000, 0x0000b127));
            test(fromBits(0x00000000, 0x0000fc15));
            test(fromBits(0x00000000, 0x00007127));
            test(fromBits(0x00000000, 0x00007c15));
            test(fromBits(0x00000000, 0x00003127));
            test(fromBits(0x00000000, 0x00003c15));
            test(fromBits(0x00000000, 0x00001127));
            test(fromBits(0x00000000, 0x00001c15));
            test(fromBits(0x00000000, 0x00000927));
            test(fromBits(0x00000000, 0x00000c15));
            test(fromBits(0x00000000, 0x00000527));
            test(fromBits(0x00000000, 0x00000415));
            test(fromBits(0x00000000, 0x00000327));
            test(fromBits(0x00000000, 0x00000215));
            test(fromBits(0x00000000, 0x00000127));
            test(fromBits(0x00000000, 0x00000115));
            test(fromBits(0x00000000, 0x000000a7));
            test(fromBits(0x00000000, 0x00000095));
            test(fromBits(0x00000000, 0x00000067));
            test(fromBits(0x00000000, 0x00000055));
            test(fromBits(0x00000000, 0x00000027));
            test(fromBits(0x00000000, 0x00000035));
            test(fromBits(0x00000000, 0x00000017));
            test(fromBits(0x00000000, 0x00000015));
            test(fromBits(0x00000000, 0x0000000f));
            test(fromBits(0x00000000, 0x00000007));
            test(fromBits(0x00000000, 0x00000003));
            test(fromBits(0x00000000, 0x00000001));

            test(fromBits(0x000fffff, 0xffffffff));
            test(fromBits(0x00080000, 0x00000000));
            test(fromBits(0x00002000, 0x00000000));
            test(fromBits(0x00000100, 0x00000000));
            test(fromBits(0x00000000, 0x40000000));
            test(fromBits(0x00000000, 0x00100000));
            test(fromBits(0x00000000, 0x00000400));
            test(fromBits(0x00000000, 0x00000020));
            test(fromBits(0x00000000, 0x00000002));
            test(fromBits(0x00000000, 0x00000001));
        }

        function testEdgeCasesNormal() {
            trace("testEdgeCasesNormal");

            // Smallest normal double and neighbors (the subnormal/normal boundary).
            test(fromBits(0x00100000, 0x00000000));
            test(fromBits(0x00100000, 0x00000001));
            test(fromBits(0x00100000, 0x00000002));
            test(fromBits(0x00100000, 0x00000003));

            // Largest finite double and neighbors.
            test(fromBits(0x7fefffff, 0xffffffff));
            test(fromBits(0x7fefffff, 0xfffffffe));
            test(fromBits(0x7fefffff, 0xfffffffd));

            test(fromBits(0x01700000, 0x00000000)); // 2^-1000
            test(fromBits(0x20b00000, 0x00000000)); // 2^-500
            test(fromBits(0x39b00000, 0x00000000)); // 2^-100
            test(fromBits(0x3cb00000, 0x00000000)); // 2^-52
            test(fromBits(0x3f500000, 0x00000000)); // 2^-10
            test(fromBits(0x3fe00000, 0x00000000)); // 2^-1
            test(fromBits(0x3ff00000, 0x00000000)); // 2^0
            test(fromBits(0x40000000, 0x00000000)); // 2^1
            test(fromBits(0x40900000, 0x00000000)); // 2^10
            test(fromBits(0x43300000, 0x00000000)); // 2^52
            test(fromBits(0x43400000, 0x00000000)); // 2^53
            test(fromBits(0x46300000, 0x00000000)); // 2^100
            test(fromBits(0x5f300000, 0x00000000)); // 2^500
            test(fromBits(0x7e700000, 0x00000000)); // 2^1000
            test(fromBits(0x7fe00000, 0x00000000)); // 2^1023 (largest normal power of two)
        }

        function test(n:Number) {
            trace(hex(n) + " -> " + n.toString());
        }

        function hex(n:Number):String {
            var bytes:ByteArray = new ByteArray();
            bytes.writeDouble(n);

            var result:String = "";
            for (var i:int = 0; i < bytes.length; i++) {
                var byteHex:String = (bytes[i] & 0xFF).toString(16);
                if (byteHex.length < 2) {
                    byteHex = "0" + byteHex;
                }
                result += byteHex;
            }
            return result;
        }

        function fromBits(hi:uint, lo:uint):Number {
            var bytes:ByteArray = new ByteArray();
            bytes.writeUnsignedInt(hi);
            bytes.writeUnsignedInt(lo);
            bytes.position = 0;
            return bytes.readDouble();
        }
    }
}
