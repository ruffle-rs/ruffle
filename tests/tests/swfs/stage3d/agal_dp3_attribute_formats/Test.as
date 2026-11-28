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
            "mov op, va0 \n" +
            "dp3 vt0.x, va1, vc0 \n" +
            "mul v0, vc2, vt0.xxxx";

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";

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

            var indices:Vector.<uint> = new Vector.<uint>();
            for (var q:int = 0; q < 4; q++) {
                var b:int = q * 4;
                indices.push(b, b + 1, b + 2, b + 2, b + 1, b + 3);
            }
            var indexBuffer:IndexBuffer3D = context.createIndexBuffer(indices.length);
            indexBuffer.uploadFromVector(indices, 0, indices.length);

            // Per vertex: x, y, z, then four attribute floats.
            var attrs:Vector.<Vector.<Number>> = new <Vector.<Number>>[
                new <Number>[1, 9, 9, 9],
                new <Number>[1, 0, 9, 9],
                new <Number>[0, 0, -1, 9],
                new <Number>[1, 0, 0, -5]
            ];
            var vertexData:Vector.<Number> = new Vector.<Number>();
            for (q = 0; q < 4; q++) {
                var x0:Number = -1 + q * 0.5;
                var x1:Number = x0 + 0.5;
                var a:Vector.<Number> = attrs[q];
                vertexData.push(x0, -1, 0, a[0], a[1], a[2], a[3]);
                vertexData.push(x1, -1, 0, a[0], a[1], a[2], a[3]);
                vertexData.push(x0,  1, 0, a[0], a[1], a[2], a[3]);
                vertexData.push(x1,  1, 0, a[0], a[1], a[2], a[3]);
            }
            var vertexBuffer:VertexBuffer3D = context.createVertexBuffer(16, 7);
            vertexBuffer.uploadFromVector(vertexData, 0, 16);
            context.setVertexBufferAt(0, vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);

            context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0, new <Number>[1, -1, -1, 1]);

            var program:Program3D = context.createProgram();
            program.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            context.setProgram(program);

            context.clear(0, 0, 0, 1);

            var formats:Vector.<String> = new <String>[
                Context3DVertexBufferFormat.FLOAT_1,
                Context3DVertexBufferFormat.FLOAT_2,
                Context3DVertexBufferFormat.FLOAT_3,
                Context3DVertexBufferFormat.FLOAT_4
            ];
            var colors:Vector.<Vector.<Number>> = new <Vector.<Number>>[
                new <Number>[1, 0, 0, 1], // red
                new <Number>[0, 1, 0, 1], // green
                new <Number>[0, 0, 1, 1], // blue
                new <Number>[1, 1, 0, 1]  // yellow
            ];
            for (q = 0; q < 4; q++) {
                context.setVertexBufferAt(1, vertexBuffer, 3, formats[q]);
                context.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 2, colors[q]);
                context.drawTriangles(indexBuffer, q * 6, 2);
            }

            context.present();
        }
    }
}
