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
            "mov v0, va1"; // copy uv to varying variable v0

        private const FRAGMENT_SHADER:String =
            "tex oc, v0, fs0 <2d,clamp,linear,mipnone>";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var programPair:Program3D;
		
		[Embed(source = "ruffle_logo.png")]
		public var RUFFLE_LOGO: Class;
		
		[Embed(source = "circle.atf", mimeType = "application/octet-stream")]
		public var CIRCLE_ATF: Class;

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
                    0, 0, 0,  0, 1,
                    0.5, 0, 0,  1, 1,
                    0.5, 0.5, 0,  1, 0,
                    0, 0.5, 0,  0, 0
                ]);
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            // Identify vertex data inputs for vertex program
            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3); // va0 is position
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_2); // va1 is texture uv coords

			var logo: BitmapData = new RUFFLE_LOGO().bitmapData;
			
			var bgraPixels = new ByteArray();
			// Test that we can skip over these first 4 bytse using byteArrayOffset
			// during uploading to the texture
			bgraPixels.writeInt(0xFFFFFFFF);
			var temp = logo.getPixels(logo.rect);
			bgraPixels.writeBytes(temp);
			
			// Convert from big endian to little endian
			for (var i = 4; i < bgraPixels.length; i += 4) {
				var first = bgraPixels[i];
				var second = bgraPixels[i + 1];
				var third = bgraPixels[i + 2];
				var fourth = bgraPixels[i + 3];
				bgraPixels[i] = fourth;
				bgraPixels[i + 1] = third;
				bgraPixels[i + 2] = second;
				bgraPixels[i + 3] = first;
			}

			renderContext.setBlendFactors(Context3DBlendFactor.SOURCE_ALPHA, Context3DBlendFactor.ONE_MINUS_SOURCE_ALPHA);
            // Upload programs to render context
            programPair = renderContext.createProgram();
            programPair.upload(vertexAssembly.agalcode, fragmentAssembly.agalcode);
            renderContext.setProgram(programPair);
		
			var bitmapDataTexture = renderContext.createRectangleTexture(logo.width, logo.height, "bgra", false);
			bitmapDataTexture.uploadFromBitmapData(logo);
		
			var byteArrayTexture = renderContext.createRectangleTexture(logo.width, logo.height, "bgra", false);
			byteArrayTexture.uploadFromByteArray(bgraPixels, 4);
		
			var circleATF: ByteArray = new CIRCLE_ATF();
			var atfTexture = renderContext.createTexture(512, 512, "bgra", false);
			atfTexture.uploadCompressedTextureFromByteArray(circleATF, 0);


            // Clear, setting stencil to 0
            renderContext.clear(.3, .3, .3, 1, 1, 0);
		
				
            renderContext.setTextureAt(0, bitmapDataTexture);
			renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([-0.7, 0.5, 0.0, 0.0]));
			renderContext.drawTriangles(indexList, 0, 2);
			
			renderContext.setTextureAt(0, byteArrayTexture);
			renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([-0.7, -0.6, 0.0, 0.0]));
			renderContext.drawTriangles(indexList, 0, 2);
			
			renderContext.setTextureAt(0, atfTexture);
			renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([0.0, 0.5, 0.0, 0.0]));
			renderContext.drawTriangles(indexList, 0, 2);
		

            renderContext.present();

            // this.addChild(new Bitmap(redGreen));
        }
    }
}
