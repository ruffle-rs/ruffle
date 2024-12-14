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

    // Based on example from https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display3D/Context3D.html#setStencilActions
    public class Test extends MovieClip {
        public const viewWidth:Number = 500;
        public const viewHeight:Number = 600;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        private const VERTEX_SHADER:String =
            "add op, va0, vc0    \n" + // copy position to output with offset vector
            "mov v0, va1"; // copy uv to varying variable v0

        private const FRAGMENT_SHADER:String =
            "tex oc, v0, fs0 <2d,clamp,linear,mipnone,ignoresampler>";
		
		
        private const FRAGMENT_SHADER_REPEAT:String =
            "tex oc, v0, fs0 <2d,repeat,linear,mipnone>";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
		private var fragmentAssemblyRepeat:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var programPair:Program3D;

        public function Test() {
            stage3D = this.stage.stage3Ds[0];

            // Add event listener before requesting the context
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            stage3D.requestContext3D(Context3DRenderMode.AUTO, "standard");

            // Compile shaders
            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 2);
            fragmentAssembly.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER, 2);
			fragmentAssemblyRepeat.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER_REPEAT, 2);
        }

        // Note, context3DCreate event can happen at any time, such as when the hardware resources are taken by another process
        private function contextCreated(event:Event):void {
            renderContext = Stage3D(event.target).context3D;

            renderContext.enableErrorChecking = true; // Can slow rendering - only turn on when developing/testing
            renderContext.configureBackBuffer(viewWidth, viewHeight, 4, false);

            // Create vertex index list for the triangles
            var triangles:Vector.<uint> = Vector.<uint>([0, 3, 2,
                    0, 1, 3,
                ]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            // Create vertexes
            const dataPerVertex:int = 5;
            var vertexData:Vector.<Number> = Vector.<Number>(
                [
                    // x, y, z   u, v
                    -.1, .1, 0, 0, 2,
                    .1, .1, 0, 2, 2,
                    -.1, -.1, 0, 0, 0,
                    .1, -.1, 0, 2, 0
                ]);
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            // Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3); // va0 is position
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_2); // va1 is texture uv coords

            const size = 4;
            var redGreen = new BitmapData(size, size, true, 0x0);
            redGreen.fillRect(new Rectangle(0, 0, size / 2, size / 2), 0xFFFF0000);
            redGreen.fillRect(new Rectangle(size / 2, 0, size / 2, size / 2), 0xFF00FF00);
            redGreen.fillRect(new Rectangle(0, size / 2, size / 2, size / 2), 0xFF0000FF);
            redGreen.fillRect(new Rectangle(size / 2, size / 2, size / 2, size / 2), 0xFFFF00FF);

            var redGreenTexture = renderContext.createTexture(size, size, "bgra", false);
            redGreenTexture.uploadFromBitmapData(redGreen);

            // This modification is done after 'redGreenTexture.uploadFromBitmapData(redGreen)',
            // so it should have no effect.
            redGreen.fillRect(new Rectangle(0, 0, size, size), 0);

            // Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
			
			var repeatProgram = renderContext.createProgram();
			repeatProgram.upload(vertexAssembly.agalcode, fragmentAssemblyRepeat.agalcode);

            // Clear, setting stencil to 0
            renderContext.clear(.3, .3, .3, 1, 1, 0);

            var offsetVec = Vector.<Number>([-0.7, 0.7, 0, 0]);
            // FIXME - implement and test anisotropic filters
			
			renderContext.setTextureAt(0, redGreenTexture);
			
			for each (var mode in [0, 1, 2, 3, 4, 5]) {
				for each (var textureFilter in [Context3DTextureFilter.NEAREST, Context3DTextureFilter.LINEAR]) {
					for each (var wrapMode in [Context3DWrapMode.CLAMP, Context3DWrapMode.CLAMP_U_REPEAT_V, Context3DWrapMode.REPEAT, Context3DWrapMode.REPEAT_U_CLAMP_V]) {
						if (mode == 0) {
							renderContext.setProgram(programPair);	
						} else if (mode == 1) {
							renderContext.setSamplerStateAt(0, wrapMode, textureFilter, "mipnone");
							renderContext.setProgram(repeatProgram);
							renderContext.setProgram(programPair);	
						} else if (mode == 2) {
							renderContext.setSamplerStateAt(0, wrapMode, textureFilter, "mipnone");
							renderContext.setProgram(programPair);	
						} else if (mode == 3) {
							renderContext.setSamplerStateAt(0, wrapMode, textureFilter, "mipnone");
							renderContext.setProgram(repeatProgram);
						} else if (mode == 4) {
							renderContext.setSamplerStateAt(0, wrapMode, textureFilter, "mipnone");
							renderContext.setProgram(programPair);
						} else if (mode == 5) {
							renderContext.setProgram(repeatProgram);	
							renderContext.setSamplerStateAt(0, wrapMode, textureFilter, "mipnone");
						}
						renderContext.setProgramConstantsFromVector("vertex", 0, offsetVec);
						renderContext.drawTriangles(indexList, 0, 2);

						offsetVec[0] += 0.3;
					}
				}
				offsetVec[1] += -0.3;
				offsetVec[0] = -0.7;
			}

            renderContext.present();
        }
    }
}
