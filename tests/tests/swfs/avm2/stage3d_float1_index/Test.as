package
{
    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;

    // Test for indirect addressing with Float1 vertex attribute as index.
    // This tests the fix for using vc[va1.x + offset] where va1 is Float1.
    public class Test extends Sprite
    {
        public const viewWidth:Number = 320;
        public const viewHeight:Number = 200;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        // Vertex shader uses indirect addressing: vc[va1.x + 0]
        // va0 = position (Float4)
        // va1 = index into constant registers (Float1)
        // vc0, vc1, vc2 = colors to choose from
        private const VERTEX_SHADER:String =
            "mov op, va0        \n" +  // copy position to output
            "mov v0, vc[va1.x]";       // load color from vc[index] using Float1 as index

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";              // output interpolated color

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var programPair:Program3D;

        public function Test()
        {
            addEventListener(Event.ADDED_TO_STAGE, onAddedToStage);
        }

        private function onAddedToStage(event:Event):void
        {
            removeEventListener(Event.ADDED_TO_STAGE, onAddedToStage);

            stage3D = stage.stage3Ds[0];

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1, false);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1, false);

            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.AUTO);
        }

        private function contextCreated(event:Event):void
        {
            renderContext = Stage3D(event.target).context3D;
            renderContext.configureBackBuffer(viewWidth, viewHeight, 4, false);

            // Triangle indices
            var triangles:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 3, 4]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            // Vertex data: position (4 floats) + index (1 float)
            // Each vertex has a Float1 index that points to a color in constant registers
            const dataPerVertex:int = 5;
            var vertexData:Vector.<Number> = Vector.<Number>(
                [
                    // x,  y, z, w,  index (into vc array)
                       0,  0, 0, 1,  0,    // center vertex uses vc0 (red)
                      -1,  1, 0, 1,  1,    // top-left uses vc1 (green)
                       1,  1, 0, 1,  2,    // top-right uses vc2 (blue)
                       1, -1, 0, 1,  1,    // bottom-right uses vc1 (green)
                      -1, -1, 0, 1,  2     // bottom-left uses vc2 (blue)
                ]
            );
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            // Set vertex attributes
            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_4); // va0 = position
            renderContext.setVertexBufferAt(1, vertexes, 4, Context3DVertexBufferFormat.FLOAT_1); // va1 = index (Float1!)

            // Set constant registers with colors
            // vc0 = red, vc1 = green, vc2 = blue
            renderContext.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 0,
                Vector.<Number>([1, 0, 0, 1]));  // vc0 = red
            renderContext.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 1,
                Vector.<Number>([0, 1, 0, 1]));  // vc1 = green
            renderContext.setProgramConstantsFromVector(Context3DProgramType.VERTEX, 2,
                Vector.<Number>([0, 0, 1, 1]));  // vc2 = blue

            // Upload and set program
            programPair = renderContext.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            renderContext.setProgram(programPair);

            // Render
            renderContext.clear(0.3, 0.3, 0.3);
            renderContext.drawTriangles(indexList, 0, 2);
            renderContext.present();
        }
    }
}
