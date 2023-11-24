package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DBlendFactor;
    import flash.display3D.Context3DCompareMode;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DStencilAction;
    import flash.display3D.Context3DTriangleFace;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.Context3DTextureFilter;
    import flash.display3D.Context3DWrapMode;
    import flash.display3D.Context3DTextureFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.events.MouseEvent;
    import flash.events.TimerEvent;
    import flash.geom.Rectangle;
    import flash.text.TextField;
    import flash.text.TextFormat;
    import flash.ui.Keyboard;
    import flash.utils.Timer;
    import flash.display.MovieClip;
    import flash.display.Stage;
    import flash.display.BitmapData;
    import flash.display.Bitmap;
    import flash.utils.ByteArray;

    // Based on example from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display3D/Context3D.html#setStencilActions
    public class Test extends MovieClip {
        public const viewWidth:Number = 500;
        public const viewHeight:Number = 500;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        private const VERTEX_SHADER:String =
            "add op, va0, vc0    \n" + // copy position to output, adding offset
            "sub v0, va0, va0      \n" +
            // Compute cross product using two-component 'va1' input - should be zero-extended
            "crs v0.xyz, va1, vc1";

        private const FRAGMENT_SHADER:String =
            "mov oc, v0";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var programPair:Program3D;

        public function Test() {
            stage3D = this.stage.stage3Ds[0];

            // Add event listener before requesting the context
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.AUTO, "standard");

            // Compile shaders
            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 2);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 2);
        }

        // Note, context3DCreate event can happen at any time, such as when the hardware resources are taken by another process
        private function contextCreated(event:Event):void {
            renderContext = Stage3D(event.target).context3D;

            renderContext.enableErrorChecking = true; // Can slow rendering - only turn on when developing/testing
            renderContext.configureBackBuffer(viewWidth, viewHeight, 4, true);

            // Create vertex index list for the triangles
            var triangles:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 2, 3]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            // Create vertexes
            const dataPerVertex:int = 5;
            var vertexData:Vector.<Number> = Vector.<Number>(
                    [
                        // x, y, z   u, v
                        0, 0, 0, 0, 1,
                        0.5, 0, 0, 1, 1,
                        0.5, 0.5, 0, 1, 0,
                        0, 0.5, 0, 0.5, 0.1,
                    ]);
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            // Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3); // va0 is position
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_2); // va1 is texture uv coords

            renderContext.setBlendFactors(Context3DBlendFactor.ONE, Context3DBlendFactor.ZERO);
            // Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            renderContext.setProgram(programPair);

            // Clear, setting stencil to 0
            renderContext.clear(.3, .3, .3, 1, 1, 0);
            renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([-0.7, 0.5, 0.0, 0.0,
                            1.0, 1.0, 1.0, 1.0]));
            renderContext.drawTriangles(indexList, 0, 2);
            renderContext.present();
        }
    }
}
