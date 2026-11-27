package {
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.display3D.textures.CubeTexture;
    import flash.events.Event;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    // Creates a cube texture in the compressedAlpha format, uploads a solid
    // color to all of its sides and draws it.
    public class Test extends MovieClip {
        private var stage3D:Stage3D;

        public function Test() {
            stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D("auto", "standard");
        }

        private function contextCreated(event:Event):void {
            var context:Context3D = stage3D.context3D;
            context.enableErrorChecking = true;
            context.configureBackBuffer(64, 64, 0, false);

            var texture:CubeTexture = context.createCubeTexture(16, "compressedAlpha", false);
            // The color is exactly representable in BC3 (as RGB565), so the
            // result doesn't depend on the compressor.
            // Flash Player requires all mip levels of a cube texture.
            for (var side:uint = 0; side < 6; side++) {
                for (var level:uint = 0, size:uint = 16; size >= 1; level++, size /= 2) {
                    texture.uploadFromBitmapData(new BitmapData(size, size, true, 0xff3165ce), side, level);
                }
            }
            trace("uploaded");

            // mov op, va0
            // mov v0, va1
            var vertexShader:ByteArray = agal(1, 0, [
                [MOV, [OUTPUT, 0], [ATTRIBUTE, 0], null],
                [MOV, [VARYING, 0], [ATTRIBUTE, 1], null]
            ]);
            // tex oc, v0, fs0 <cube,clamp,linear,mipnone,dxt5>
            var fragmentShader:ByteArray = agal(1, 1, [
                [TEX, [OUTPUT, 0], [VARYING, 0], [SAMPLER, 0, DXT5, TEXTURE_CUBE, LINEAR]]
            ]);
            var program:Program3D = context.createProgram();
            program.upload(vertexShader, fragmentShader);

            // A quad covering the viewport, looking along +z.
            var vertices:VertexBuffer3D = context.createVertexBuffer(4, 6);
            vertices.uploadFromVector(Vector.<Number>([
                -1, -1, 0,  0, 0, 1,
                 1, -1, 0,  0, 0, 1,
                 1,  1, 0,  0, 0, 1,
                -1,  1, 0,  0, 0, 1
            ]), 0, 4);
            var indices:IndexBuffer3D = context.createIndexBuffer(6);
            indices.uploadFromVector(Vector.<uint>([0, 1, 2, 0, 2, 3]), 0, 6);

            context.setVertexBufferAt(0, vertices, 0, Context3DVertexBufferFormat.FLOAT_3);
            context.setVertexBufferAt(1, vertices, 3, Context3DVertexBufferFormat.FLOAT_3);
            context.setProgram(program);
            context.setTextureAt(0, texture);
            context.clear(0, 0, 0, 1);
            context.drawTriangles(indices);

            var result:BitmapData = new BitmapData(64, 64, false, 0);
            context.drawToBitmapData(result);
            trace("center: " + result.getPixel(32, 32).toString(16));
        }

        private static const MOV:uint = 0x00;
        private static const ADD:uint = 0x01;
        private static const TEX:uint = 0x28;

        private static const ATTRIBUTE:uint = 0;
        private static const CONSTANT:uint = 1;
        private static const OUTPUT:uint = 3;
        private static const VARYING:uint = 4;
        private static const SAMPLER:uint = 5;

        private static const RGBA:uint = 0;
        private static const DXT5:uint = 2;
        private static const TEXTURE_2D:uint = 0;
        private static const TEXTURE_CUBE:uint = 1;
        private static const LINEAR:uint = 1;

        // Assembles AGAL bytecode, so the test doesn't need an assembler. Each
        // instruction is [opcode, destination, source1, source2], where the
        // destination and sources are [registerType, register], source2 can be
        // null, and a texture sampler is [SAMPLER, register, format, dimension,
        // filter], with clamping and without mipmaps.
        private static function agal(version:uint, programType:uint, instructions:Array):ByteArray {
            var code:ByteArray = new ByteArray();
            code.endian = Endian.LITTLE_ENDIAN;
            code.writeByte(0xA0);
            code.writeUnsignedInt(version);
            code.writeByte(0xA1);
            code.writeByte(programType);

            for each (var instruction:Array in instructions) {
                code.writeUnsignedInt(instruction[0]);
                code.writeShort(instruction[1][1]);
                code.writeByte(0x0F); // Write mask: xyzw
                code.writeByte(instruction[1][0]);
                writeSource(code, instruction[2]);
                writeSource(code, instruction[3]);
            }

            return code;
        }

        private static function writeSource(code:ByteArray, source:Array):void {
            if (source == null) {
                code.writeUnsignedInt(0);
                code.writeUnsignedInt(0);
            } else if (source[0] == SAMPLER) {
                code.writeShort(source[1]);
                code.writeShort(0); // LOD bias
                code.writeByte(SAMPLER);
                code.writeByte(source[2] | (source[3] << 4));
                code.writeByte(0); // No special flags, clamping
                code.writeByte(source[4] << 4); // No mipmaps
            } else {
                code.writeShort(source[1]);
                code.writeByte(0); // Indirect offset
                code.writeByte(0xE4); // Swizzle: xyzw
                code.writeByte(source[0]);
                code.writeByte(0);
                code.writeShort(0);
            }
        }
    }
}
