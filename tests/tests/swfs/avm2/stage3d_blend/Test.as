package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Sprite;
    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DBlendFactor;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.ErrorEvent;
    import flash.events.Event;
    import flash.events.KeyboardEvent;
    import flash.ui.Keyboard;
    import flash.display.Stage;
    import flash.geom.Matrix3D;

    // Based on the example from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display3D/Context3D.html#setBlendFactors()

    public class Test {
        public const viewWidth:Number = 1280;
        public const viewHeight:Number = 800;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        private const VERTEX_SHADER:String =
            "m44 op, va0, vc0    \n" + // Apply the matrix transformation and write to output (starting matrix register is vc0)
            "mov v2, va0         \n" + // Do some dummy writes to varying registers, to ensure that we handle writes in any order
            "mov v3, va0         \n" +
            "mov v0, va1"; // copy color to varying variable v0

        // Due to a WGPU limitation, we need to read all of the outputs from the fragment shader, or we get a validation error.
        // FIXME - remove the dummy reads when https://github.com/gfx-rs/wgpu/issues/3748 is fixed
        private const FRAGMENT_SHADER:String =
            "mov ft0, v3  \n" +
            "mov ft1, v2  \n" +
            "mov oc, v0"; // Set the output color to the value interpolated from the three triangle vertices

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler();
        private var programPair:Program3D;

        // private var sourceFactor:int = 6;
        // private var destinationFactor:int = 4;
        private var blendFactors:Array = [Context3DBlendFactor.DESTINATION_ALPHA,
                Context3DBlendFactor.DESTINATION_COLOR,
                Context3DBlendFactor.ONE,
                Context3DBlendFactor.ONE_MINUS_DESTINATION_ALPHA,
                Context3DBlendFactor.ONE_MINUS_SOURCE_ALPHA,
                Context3DBlendFactor.ONE_MINUS_SOURCE_COLOR,
                Context3DBlendFactor.SOURCE_ALPHA,
                Context3DBlendFactor.SOURCE_COLOR,
                Context3DBlendFactor.ZERO];

        public function Test(stage:Stage) {
            stage.addEventListener(KeyboardEvent.KEY_DOWN, keyHandler);
            // stage.addEventListener("enterFrame", function() { render() });

            stage3D = stage.stage3Ds[0];
            stage3D.x = 10;
            stage3D.y = 10;

            // Add event listener before requesting the context
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.addEventListener(ErrorEvent.ERROR, contextError);
            stage3D.requestContext3D(Context3DRenderMode.AUTO);

            // Compile shaders
            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 1);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 1);
        }

        // Note, context3DCreate event can happen at any time, such as when the hardware resources are taken by another process
        private function contextCreated(event:Event):void {
            renderContext = Stage3D(event.target).context3D;

            renderContext.enableErrorChecking = true; // Can slow rendering - only turn on when developing/testing
            renderContext.configureBackBuffer(viewWidth, viewHeight, 4, false);

            // Create vertex index list for the triangles
            var triangles:Vector.<uint> = Vector.<uint>([0, 3, 2,
                    0, 1, 3,
                    6, 4, 5,
                    5, 7, 6,
                    10, 8, 9,
                    9, 11, 10,
                    12, 15, 14,
                    12, 13, 15,
                    16, 17, 19,
                    16, 19, 18
                ]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            // Create vertexes
            const dataPerVertex:int = 7;
            var vertexData:Vector.<Number> = Vector.<Number>(
                [
                    // x, y, z    r, g, b, a format
                    -.1, .1, 0, 1, 1, 1, .5,
                    0, .1, 0, 1, 1, 1, .5,
                    -.1, 0, 0, 1, 1, 1, .5,
                    0, 0, 0, 1, 1, 1, .5,

                    0, .1, 0, .8, .8, .8, .6,
                    .1, .1, 0, .8, .8, .8, .6,
                    0, 0, 0, .8, .8, .8, .6,
                    .1, 0, 0, .8, .8, .8, .6,

                    -.1, 0, 0, 1, 0, 0, .5,
                    0, 0, 0, 0, 1, 0, .5,
                    -.1, -.1, 0, 0, 0, 1, .5,
                    0, -.1, 0, 1, 0, 1, .5,

                    0, 0, 0, 0, 0, 0, .5,
                    .1, 0, 0, 0, 0, 0, .5,
                    0, -.1, 0, 0, 0, 0, .5,
                    .1, -.1, 0, 0, 0, 0, .5,

                    -.08, .08, 0, .6, .4, .2, .4,
                    .08, .08, 0, .6, .4, .2, .4,
                    -.08, -.08, 0, .6, .4, .2, .4,
                    .08, -.08, 0, .6, .4, .2, .4
                ]
                );
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            // Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3); // va0 is position
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_4); // va1 is color

            // Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            renderContext.setProgram(programPair);

            render();
        }

        private function render():void {
            // Clear required before first drawTriangles() call
            renderContext.clear(1, 1, 1, 1);

            for (var sourceIdx = 0; sourceIdx < blendFactors.length; sourceIdx++) {
                for (var destIdx = 0; destIdx < blendFactors.length; destIdx++) {
                    var currentSourceFactor = blendFactors[sourceIdx];
                    var currentDestFactor = blendFactors[destIdx];

                    trace("Blending with: source=" + currentSourceFactor + " dest=" + currentDestFactor);

                    var mat = new Matrix3D();
                    mat.appendScale(0.7, 0.7, 1);
                    mat.appendTranslation(-.9 + (0.2 * sourceIdx), -.75 + (0.2 * destIdx), 0);

                    renderContext.setProgramConstantsFromMatrix("vertex", 0, mat, true);

                    // Draw the back triangles
                    renderContext.setBlendFactors(Context3DBlendFactor.ONE, Context3DBlendFactor.ZERO); // No blending
                    renderContext.drawTriangles(indexList, 0, 8);

                    // Set blend
                    renderContext.setBlendFactors(currentSourceFactor, currentDestFactor);

                    // Draw the front triangles
                    renderContext.drawTriangles(indexList, 24, 2);
                }

            }

            // Show the frame
            renderContext.present();
        }

        private function contextError(error:ErrorEvent):void {
            trace(error.errorID + ": " + error.text);
        }

        private function keyHandler(event:KeyboardEvent):void {
            /*switch ( event.keyCode )
            {
                case Keyboard.NUMBER_1:
                    if( --sourceFactor < 0 ) sourceFactor = blendFactors.length - 1; 
                    break;
                case Keyboard.NUMBER_2:
                    if( ++sourceFactor > blendFactors.length - 1) sourceFactor = 0;
                    break;
                case Keyboard.NUMBER_3:
                    if( --destinationFactor < 0 ) destinationFactor = blendFactors.length - 1; 
                    break;
                case Keyboard.NUMBER_4:
                    if( ++destinationFactor > blendFactors.length - 1) destinationFactor = 0;
                    break;
            }
            trace( "Source blend factor: " + blendFactors[sourceFactor] + ", destination blend factor: " + blendFactors[destinationFactor] );
            render();*/
        }
    }
}