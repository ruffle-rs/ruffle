package {
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DBlendFactor;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.display.MovieClip;
    import flash.display.BitmapData;
    import flash.utils.ByteArray;
    import flash.utils.Endian;

    public class Test extends MovieClip {
        public const viewWidth:Number = 500;
        public const viewHeight:Number = 500;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        // add op, va0, vc0
        // mov v0, va1
        private const VERTEX_SHADER:ByteArray = agal(2, 0, [
            [ADD, [OUTPUT, 0], [ATTRIBUTE, 0], [CONSTANT, 0]],
            [MOV, [VARYING, 0], [ATTRIBUTE, 1], null]
        ]);

        // tex oc, v0, fs0 <2d,clamp,linear,mipnone,dxt5>
        private const FRAGMENT_SHADER_DXT5:ByteArray = agal(2, 1, [
            [TEX, [OUTPUT, 0], [VARYING, 0], [SAMPLER, 0, DXT5, TEXTURE_2D, LINEAR]]
        ]);

        // tex oc, v0, fs0 <2d,clamp,linear,mipnone>
        private const FRAGMENT_SHADER_FLOAT:ByteArray = agal(2, 1, [
            [TEX, [OUTPUT, 0], [VARYING, 0], [SAMPLER, 0, RGBA, TEXTURE_2D, LINEAR]]
        ]);

        private var programDxt5:Program3D;
        private var programFloat:Program3D;

        public function Test() {
            stage3D = this.stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            // Request "standard" profile which supports compressedAlpha and rgbaHalfFloat
            stage3D.requestContext3D(Context3DRenderMode.AUTO, "standard");
        }

        private function contextCreated(event:Event):void {
            renderContext = Stage3D(event.target).context3D;
            renderContext.enableErrorChecking = true;
            renderContext.configureBackBuffer(viewWidth, viewHeight, 0, true);

            var triangles:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 2, 3]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            const dataPerVertex:int = 5;
            var vertexData:Vector.<Number> = Vector.<Number>([
                -0.4, -0.4, 0,  0, 1,
                0.4, -0.4, 0,  1, 1,
                0.4, 0.4, 0,  1, 0,
                -0.4, 0.4, 0,  0, 0
            ]);
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3);
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_2);

            var circleBitmap:BitmapData = new CirclePng();

            var compressedTexture = renderContext.createTexture(
                512, 512,
                "compressedAlpha",
                false
            );
            compressedTexture.uploadFromBitmapData(circleBitmap);

            var halfFloatTexture = renderContext.createTexture(
                512, 512,
                "rgbaHalfFloat",
                false
            );
            halfFloatTexture.uploadFromBitmapData(circleBitmap);

            programDxt5 = renderContext.createProgram();
            programDxt5.upload(VERTEX_SHADER, FRAGMENT_SHADER_DXT5);

            programFloat = renderContext.createProgram();
            programFloat.upload(VERTEX_SHADER, FRAGMENT_SHADER_FLOAT);

            renderContext.setBlendFactors(Context3DBlendFactor.SOURCE_ALPHA, Context3DBlendFactor.ONE_MINUS_SOURCE_ALPHA);

            renderContext.clear(.3, .3, .3, 1, 1, 0);

            renderContext.setProgram(programDxt5);
            renderContext.setTextureAt(0, compressedTexture);
            renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([-0.5, 0.0, 0.0, 0.0]));
            renderContext.drawTriangles(indexList, 0, 2);

            renderContext.setProgram(programFloat);
            renderContext.setTextureAt(0, halfFloatTexture);
            renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([0.5, 0.0, 0.0, 0.0]));
            renderContext.drawTriangles(indexList, 0, 2);

            renderContext.present();
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
