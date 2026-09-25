package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;

    // drawToBitmapData copies the back buffer into a BitmapData 1:1 from the
    // top-left, read here into a same-size, larger and smaller destination.
    [SWF(width="100", height="50", backgroundColor="#000000")]
    public class Test extends Sprite {
        private const VERTEX_SHADER:String =
            "mov op, va0  \n" + // position (already in clip space)
            "mov v0, va1";      // pass colour to fragment shader

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";       // output the interpolated colour

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();

        public function Test() {
            stage.scaleMode = StageScaleMode.NO_SCALE;
            stage.align = StageAlign.TOP_LEFT;

            var s3d:Stage3D = stage.stage3Ds[0];
            s3d.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            s3d.requestContext3D(Context3DRenderMode.AUTO);

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1);
        }

        private function onContext(e:Event):void {
            var context:Context3D = Stage3D(e.target).context3D;
            context.configureBackBuffer(50, 50, 0, false);

            var indices:Vector.<uint> = Vector.<uint>([0, 1, 2, 2, 1, 3]);
            var indexBuffer:IndexBuffer3D = context.createIndexBuffer(indices.length);
            indexBuffer.uploadFromVector(indices, 0, indices.length);

            // Red quad, left half of clip space.
            var vertexData:Vector.<Number> = Vector.<Number>([
                -1, -1, 0,  1, 0, 0, 1,
                 0, -1, 0,  1, 0, 0, 1,
                -1,  1, 0,  1, 0, 0, 1,
                 0,  1, 0,  1, 0, 0, 1
            ]);
            var vertexBuffer:VertexBuffer3D = context.createVertexBuffer(4, 7);
            vertexBuffer.uploadFromVector(vertexData, 0, 4);
            context.setVertexBufferAt(0, vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);
            context.setVertexBufferAt(1, vertexBuffer, 3, Context3DVertexBufferFormat.FLOAT_4);

            var program:Program3D = context.createProgram();
            program.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            context.setProgram(program);

            // Back buffer: red left, blue right.
            context.clear(0, 0, 1, 1);
            context.drawTriangles(indexBuffer, 0, 2);

            // Same size.
            var same:BitmapData = new BitmapData(50, 50, true, 0xff00ff00);
            context.drawToBitmapData(same);
            trace("same(5,5):   " + hex(same.getPixel32(5, 5)));
            trace("same(45,5):  " + hex(same.getPixel32(45, 5)));

            // Larger: extra pixels keep their fill.
            var big:BitmapData = new BitmapData(80, 60, true, 0xff808080);
            context.drawToBitmapData(big);
            trace("big(5,5):    " + hex(big.getPixel32(5, 5)));
            trace("big(40,5):   " + hex(big.getPixel32(40, 5)));
            trace("big(60,5):   " + hex(big.getPixel32(60, 5)));
            trace("big(5,55):   " + hex(big.getPixel32(5, 55)));

            // Smaller: clipped, not scaled.
            var small:BitmapData = new BitmapData(30, 30, true, 0xff808080);
            context.drawToBitmapData(small);
            trace("small(5,5):  " + hex(small.getPixel32(5, 5)));
            trace("small(20,5): " + hex(small.getPixel32(20, 5)));
            trace("small(29,5): " + hex(small.getPixel32(29, 5)));

            // Null destination throws #2007.
            try {
                context.drawToBitmapData(null);
                trace("null: no error");
            } catch (e:Error) {
                trace("null: " + e.getStackTrace());
            }

            // After present(), a read with no clear throws #3692.
            context.present();
            try {
                context.drawToBitmapData(new BitmapData(50, 50, true, 0xff000000));
                trace("after present: no error");
            } catch (e:Error) {
                trace("after present: " + e.getStackTrace());
            }

            // Show the larger capture: red|blue back buffer, then grey fill.
            addChild(new Bitmap(big));
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
