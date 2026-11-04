package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;

    [SWF(width="40", height="40", backgroundColor="#000000")]
    public class Test extends Sprite {

        private const VERTEX_SHADER:String =
            "mov op, va0  \n" +
            "neg op.y, va0 \n" +
            "mov v0, va1";

        private const FRAGMENT_SHADER:String =
            "mov ft0, v0 \n" +
            "mul ft0.z, v0, v0 \n" +
            "mov oc, ft0";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);

        public function Test() {
            stage.align = StageAlign.TOP_LEFT;

            var stage3D:Stage3D = stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, onContext);
            stage3D.requestContext3D(Context3DRenderMode.AUTO);

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1);
        }

        private function onContext(e:Event):void {
            var context:Context3D = Stage3D(e.target).context3D;
            context.configureBackBuffer(40, 40, 0, false);

            var indices:Vector.<uint> = new <uint>[0, 1, 2, 2, 1, 3];
            var indexBuffer:IndexBuffer3D = context.createIndexBuffer(indices.length);
            indexBuffer.uploadFromVector(indices, 0, indices.length);
            // Cyan quad in the TOP-left of clip space.
            // The vertex shader negates y, so it should land BOTTOM-left.
            var vertexData:Vector.<Number> = new <Number>[
                -1, 0, 0,  0, 1, 1, 1,
                 0, 0, 0,  0, 1, 1, 1,
                -1, 1, 0,  0, 1, 1, 1,
                 0, 1, 0,  0, 1, 1, 1
            ];
            var vertexBuffer:VertexBuffer3D = context.createVertexBuffer(4, 7);
            vertexBuffer.uploadFromVector(vertexData, 0, 4);
            context.setVertexBufferAt(0, vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);
            context.setVertexBufferAt(1, vertexBuffer, 3, Context3DVertexBufferFormat.FLOAT_4);

            var program:Program3D = context.createProgram();
            program.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            context.setProgram(program);

            context.clear(0, 0, 0, 1);
            context.drawTriangles(indexBuffer, 0, 2);
            context.present();
        }
    }
}
