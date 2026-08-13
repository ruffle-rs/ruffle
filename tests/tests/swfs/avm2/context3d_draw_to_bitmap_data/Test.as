package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display.BitmapData;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;

    // Context3D.drawToBitmapData copies the current back buffer into the given
    // BitmapData. A red fullscreen quad is drawn over a blue clear, then read
    // back into a green-filled BitmapData. Every read pixel must be red: green
    // would mean the draw never happened, blue would mean the quad didn't cover
    // the pixel.
    public class Test extends Sprite {
        private const VERTEX_SHADER:String =
            "mov op, va0  \n" + // position (already in clip space)
            "mov v0, va1";      // pass colour to fragment shader

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";       // output the interpolated colour

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();

        public function Test() {
            var s3d:Stage3D = stage.stage3Ds[0];
            s3d.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            s3d.requestContext3D(Context3DRenderMode.AUTO);

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1);
        }

        private function onContext(e:Event):void {
            var context:Context3D = Stage3D(e.target).context3D;
            context.configureBackBuffer(50, 50, 0, false);

            // Two triangles covering the whole clip-space viewport.
            var indices:Vector.<uint> = Vector.<uint>([0, 1, 2, 2, 1, 3]);
            var indexBuffer:IndexBuffer3D = context.createIndexBuffer(indices.length);
            indexBuffer.uploadFromVector(indices, 0, indices.length);

            // x, y, z, r, g, b, a - all vertices opaque red.
            var vertexData:Vector.<Number> = Vector.<Number>([
                -1, -1, 0,  1, 0, 0, 1,
                 1, -1, 0,  1, 0, 0, 1,
                -1,  1, 0,  1, 0, 0, 1,
                 1,  1, 0,  1, 0, 0, 1
            ]);
            var vertexBuffer:VertexBuffer3D = context.createVertexBuffer(4, 7);
            vertexBuffer.uploadFromVector(vertexData, 0, 4);
            context.setVertexBufferAt(0, vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);
            context.setVertexBufferAt(1, vertexBuffer, 3, Context3DVertexBufferFormat.FLOAT_4);

            var program:Program3D = context.createProgram();
            program.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            context.setProgram(program);

            // Clear to blue, then draw the red quad on top.
            context.clear(0, 0, 1, 1);
            context.drawTriangles(indexBuffer, 0, 2);

            // Read the back buffer into a green-filled BitmapData.
            var bmd:BitmapData = new BitmapData(50, 50, true, 0xff00ff00);
            context.drawToBitmapData(bmd);

            trace("pixel(0,0):   " + hex(bmd.getPixel32(0, 0)));
            trace("pixel(25,25): " + hex(bmd.getPixel32(25, 25)));
            trace("pixel(49,49): " + hex(bmd.getPixel32(49, 49)));
        }

        private function hex(color:uint):String {
            var s:String = color.toString(16);
            while (s.length < 8) {
                s = "0" + s;
            }
            return s;
        }
    }
}
