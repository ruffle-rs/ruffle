package {
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends MovieClip {
        private var context:Context3D;
        private var indices:IndexBuffer3D;
        private var vertices:VertexBuffer3D;
        private var program:Program3D;

        public function Test() {
            var stage3D:Stage3D = this.stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.SOFTWARE, "baseline");
        }

        private function contextCreated(event:Event):void {
            context = Stage3D(event.target).context3D;
            context.configureBackBuffer(64, 64, 0, false);

            indices = context.createIndexBuffer(3);
            indices.uploadFromVector(Vector.<uint>([0, 1, 2]), 0, 3);

            // A triangle covering the whole viewport, with a color per vertex.
            vertices = context.createVertexBuffer(3, 6);
            vertices.uploadFromVector(Vector.<Number>([
                -1, -1, 1, 0, 0, 1,
                -1, 3, 1, 0, 0, 1,
                3, -1, 1, 0, 0, 1
            ]), 0, 3);

            // mov op, va0
            // mov v0, va1
            var vertexCode:ByteArray = agal(0, [
                [0x00, 0, 3, 0, 0, -1, -1],
                [0x00, 0, 4, 1, 0, -1, -1]
            ]);
            // add oc, v0, fc0
            var fragmentCode:ByteArray = agal(1, [
                [0x01, 0, 3, 0, 4, 0, 1]
            ]);

            program = context.createProgram();
            program.upload(vertexCode, fragmentCode);

            runTest(false);
            runTest(true);

            trace("Done");
        }

        private function runTest(errorChecking:Boolean):void {
            trace("/// enableErrorChecking = " + errorChecking);
            context.enableErrorChecking = errorChecking;

            runCase("all streams set", function():void {
                context.setVertexBufferAt(0, vertices, 0, Context3DVertexBufferFormat.FLOAT_2);
                context.setVertexBufferAt(1, vertices, 2, Context3DVertexBufferFormat.FLOAT_4);
            });
            runCase("stream 1 never set", function():void {
                context.setVertexBufferAt(0, vertices, 0, Context3DVertexBufferFormat.FLOAT_2);
            });
            runCase("stream 1 cleared", function():void {
                context.setVertexBufferAt(0, vertices, 0, Context3DVertexBufferFormat.FLOAT_2);
                context.setVertexBufferAt(1, vertices, 2, Context3DVertexBufferFormat.FLOAT_4);
                context.setVertexBufferAt(1, null);
            });
            runCase("all streams set again", function():void {
                context.setVertexBufferAt(0, vertices, 0, Context3DVertexBufferFormat.FLOAT_2);
                context.setVertexBufferAt(1, vertices, 2, Context3DVertexBufferFormat.FLOAT_4);
            });
        }

        private function runCase(name:String, setup:Function):void {
            trace("// " + name);
            context.setVertexBufferAt(0, null);
            context.setVertexBufferAt(1, null);
            setup();
            context.setProgram(program);
            context.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 0, Vector.<Number>([0.25, 0, 0, 0]));
            context.clear(0, 0, 0, 1);

            try {
                context.drawTriangles(indices);
                trace("drawTriangles did not throw");
            } catch (e:Error) {
                trace(Object(e).constructor);
                trace(e.getStackTrace());
            }

            var bitmap:BitmapData = new BitmapData(64, 64, true, 0);
            context.drawToBitmapData(bitmap);
            trace("pixel: " + bitmap.getPixel32(32, 32).toString(16));
            context.present();
        }

        // Each instruction is [opcode, destReg, destType, src1Reg, src1Type, src2Reg, src2Type].
        // A source with type -1 is unused.
        private static function agal(programType:int, instructions:Array):ByteArray {
            var code:ByteArray = new ByteArray();
            code.endian = Endian.LITTLE_ENDIAN;
            code.writeByte(0xA0);
            code.writeUnsignedInt(1);
            code.writeByte(0xA1);
            code.writeByte(programType);
            for each (var insn:Array in instructions) {
                code.writeUnsignedInt(insn[0]);
                code.writeShort(insn[1]);
                code.writeByte(0x0F);
                code.writeByte(insn[2]);
                writeSource(code, insn[3], insn[4]);
                writeSource(code, insn[5], insn[6]);
            }
            return code;
        }

        private static function writeSource(code:ByteArray, reg:int, type:int):void {
            if (type < 0) {
                code.writeUnsignedInt(0);
                code.writeUnsignedInt(0);
                return;
            }
            code.writeShort(reg);
            code.writeByte(0);
            code.writeByte(0xE4);
            code.writeByte(type);
            code.writeByte(0);
            code.writeShort(0);
        }
    }
}
