package {
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.events.Event;
    import flash.geom.Matrix3D;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends MovieClip {

        public function Test() {
            super();
            this.stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            this.stage.stage3Ds[0].requestContext3D();
        }

        private function contextCreated(event:Event):void {
            var stage3d:Stage3D = event.target as Stage3D;
            var context:Context3D = stage3d.context3D;
            context.configureBackBuffer(100, 100, 0, false);

            trace("=== Vector Tests ===");

            // Test 1: Vector too small for numRegisters
            try {
                var smallVec:Vector.<Number> = new <Number>[1.0, 2.0, 3.0]; // Only 3 elements, need 4
                context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, smallVec, 1);
                trace("Test 1: Should have thrown");
            } catch (e:Error) {
                trace("Test 1: " + e.errorID);
            }

            // Test 2: Empty vector with numRegisters=1
            try {
                var emptyVec:Vector.<Number> = new Vector.<Number>();
                context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, emptyVec, 1);
                trace("Test 2: Should have thrown");
            } catch (e:Error) {
                trace("Test 2: " + e.errorID);
            }

            // Test 3: Vector too small for 2 registers
            try {
                var vec7:Vector.<Number> = new <Number>[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]; // 7 elements, need 8
                context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, vec7, 2);
                trace("Test 3: Should have thrown");
            } catch (e:Error) {
                trace("Test 3: " + e.errorID);
            }

            // Test 4: Valid vector (should succeed)
            try {
                var validVec:Vector.<Number> = new <Number>[1.0, 2.0, 3.0, 4.0];
                context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, validVec, 1);
                trace("Test 4: Success");
            } catch (e:Error) {
                trace("Test 4: " + e.errorID);
            }

            trace("=== ByteArray Tests ===");

            // Test 5: ByteArray too small
            try {
                var smallBA:ByteArray = new ByteArray();
                smallBA.endian = Endian.LITTLE_ENDIAN;
                smallBA.writeFloat(1.0);
                smallBA.writeFloat(2.0);
                smallBA.writeFloat(3.0);
                // Only 12 bytes, need 16
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, smallBA, 0);
                trace("Test 5: Should have thrown");
            } catch (e:Error) {
                trace("Test 5: " + e.errorID);
            }

            // Test 6: Offset too large
            try {
                var ba16:ByteArray = new ByteArray();
                ba16.endian = Endian.LITTLE_ENDIAN;
                ba16.writeFloat(1.0);
                ba16.writeFloat(2.0);
                ba16.writeFloat(3.0);
                ba16.writeFloat(4.0);
                // 16 bytes, but offset 20 is past end
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, ba16, 20);
                trace("Test 6: Should have thrown");
            } catch (e:Error) {
                trace("Test 6: " + e.errorID);
            }

            // Test 7: Offset at end (0 bytes remaining)
            try {
                var ba16b:ByteArray = new ByteArray();
                ba16b.endian = Endian.LITTLE_ENDIAN;
                ba16b.writeFloat(1.0);
                ba16b.writeFloat(2.0);
                ba16b.writeFloat(3.0);
                ba16b.writeFloat(4.0);
                // 16 bytes, offset 16 leaves 0 bytes
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, ba16b, 16);
                trace("Test 7: Should have thrown");
            } catch (e:Error) {
                trace("Test 7: " + e.errorID);
            }

            // Test 8: Not enough data after offset
            try {
                var ba20:ByteArray = new ByteArray();
                ba20.endian = Endian.LITTLE_ENDIAN;
                ba20.writeFloat(1.0);
                ba20.writeFloat(2.0);
                ba20.writeFloat(3.0);
                ba20.writeFloat(4.0);
                ba20.writeFloat(5.0);
                // 20 bytes, offset 8 leaves 12 bytes, need 16
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, ba20, 8);
                trace("Test 8: Should have thrown");
            } catch (e:Error) {
                trace("Test 8: " + e.errorID);
            }

            // Test 9: Empty ByteArray
            try {
                var emptyBA:ByteArray = new ByteArray();
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, emptyBA, 0);
                trace("Test 9: Should have thrown");
            } catch (e:Error) {
                trace("Test 9: " + e.errorID);
            }

            // Test 10: Valid ByteArray (should succeed)
            try {
                var validBA:ByteArray = new ByteArray();
                validBA.endian = Endian.LITTLE_ENDIAN;
                validBA.writeFloat(1.0);
                validBA.writeFloat(2.0);
                validBA.writeFloat(3.0);
                validBA.writeFloat(4.0);
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 1, validBA, 0);
                trace("Test 10: Success");
            } catch (e:Error) {
                trace("Test 10: " + e.errorID);
            }

            // Test 11: 2 registers with offset
            try {
                var ba32:ByteArray = new ByteArray();
                ba32.endian = Endian.LITTLE_ENDIAN;
                for (var i:int = 0; i < 8; i++) {
                    ba32.writeFloat(i);
                }
                // 32 bytes total, offset 4 leaves 28 bytes, need 32 for 2 registers
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 2, ba32, 4);
                trace("Test 11: Should have thrown");
            } catch (e:Error) {
                trace("Test 11: " + e.errorID);
            }

            // Test 12: Valid 2 registers
            try {
                var ba32valid:ByteArray = new ByteArray();
                ba32valid.endian = Endian.LITTLE_ENDIAN;
                for (var j:int = 0; j < 8; j++) {
                    ba32valid.writeFloat(j);
                }
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, 2, ba32valid, 0);
                trace("Test 12: Success");
            } catch (e:Error) {
                trace("Test 12: " + e.errorID);
            }

            // Test 13: ByteArray with numRegisters=-1 (throws error, unlike Vector)
            try {
                var baNeg1:ByteArray = new ByteArray();
                baNeg1.endian = Endian.LITTLE_ENDIAN;
                for (var k:int = 0; k < 8; k++) {
                    baNeg1.writeFloat(k);
                }
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, -1, baNeg1, 0);
                trace("Test 13: Should have thrown");
            } catch (e:Error) {
                trace("Test 13: " + e.errorID);
            }

            // Test 14: ByteArray with numRegisters=-2
            try {
                var baNeg2:ByteArray = new ByteArray();
                baNeg2.endian = Endian.LITTLE_ENDIAN;
                for (var m:int = 0; m < 8; m++) {
                    baNeg2.writeFloat(m);
                }
                context.setProgramConstantsFromByteArray(Context3DProgramType.VERTEX, 0, -2, baNeg2, 0);
                trace("Test 14: Should have thrown");
            } catch (e:Error) {
                trace("Test 14: " + e.errorID);
            }

            // Test 15: ByteArray with invalid programType
            try {
                var baInvalid:ByteArray = new ByteArray();
                baInvalid.endian = Endian.LITTLE_ENDIAN;
                for (var n:int = 0; n < 4; n++) {
                    baInvalid.writeFloat(n);
                }
                context.setProgramConstantsFromByteArray("invalid", 0, 1, baInvalid, 0);
                trace("Test 15: Should have thrown");
            } catch (e:Error) {
                trace("Test 15: " + e.errorID);
            }

            // Test 16: Vector with invalid programType
            try {
                var vecInvalid:Vector.<Number> = new <Number>[1.0, 2.0, 3.0, 4.0];
                context.setProgramConstantsFromVector("invalid", 0, vecInvalid, 1);
                trace("Test 16: Should have thrown");
            } catch (e:Error) {
                trace("Test 16: " + e.errorID);
            }

            trace("=== Matrix Tests ===");

            // Test 17: Matrix with invalid programType
            try {
                var mat:flash.geom.Matrix3D = new flash.geom.Matrix3D();
                context.setProgramConstantsFromMatrix("invalid", 0, mat, false);
                trace("Test 17: Should have thrown");
            } catch (e:Error) {
                trace("Test 17: " + e.errorID);
            }

            trace("Done");
        }
    }
}
