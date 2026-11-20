package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.display3D.textures.CubeTexture;
    import flash.events.Event;

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

            var vertexShader:AGALMiniAssembler = new AGALMiniAssembler();
            vertexShader.assemble(Context3DProgramType.VERTEX, "mov op, va0\nmov v0, va1");
            var fragmentShader:AGALMiniAssembler = new AGALMiniAssembler();
            fragmentShader.assemble(Context3DProgramType.FRAGMENT, "tex oc, v0, fs0 <cube,clamp,linear,mipnone,dxt5>");
            var program:Program3D = context.createProgram();
            program.upload(vertexShader.agalcode, fragmentShader.agalcode);

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
    }
}
