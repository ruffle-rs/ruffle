package {
    import com.adobe.utils.AGALMiniAssembler;

    import flash.display.Stage3D;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DBlendFactor;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DRenderMode;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import flash.display.MovieClip;
    import flash.display.BitmapData;

    public class Test extends MovieClip {
        public const viewWidth:Number = 500;
        public const viewHeight:Number = 500;

        private var stage3D:Stage3D;
        private var renderContext:Context3D;
        private var indexList:IndexBuffer3D;
        private var vertexes:VertexBuffer3D;

        private const VERTEX_SHADER:String =
            "add op, va0, vc0    \n" +
            "mov v0, va1";

        private const FRAGMENT_SHADER_DXT5:String =
            "tex oc, v0, fs0 <2d,clamp,linear,mipnone,dxt5>";

        private const FRAGMENT_SHADER_FLOAT:String =
            "tex oc, v0, fs0 <2d,clamp,linear,mipnone>";

        private var vertexAssembly:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssemblyDxt5:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var fragmentAssemblyFloat:AGALMiniAssembler = new AGALMiniAssembler(false);
        private var programDxt5:Program3D;
        private var programFloat:Program3D;

        [Embed(source = "circle.png")]
        public var CIRCLE_PNG: Class;

        public function Test() {
            stage3D = this.stage.stage3Ds[0];
            stage3D.addEventListener(Event.CONTEXT3D_CREATE, contextCreated);
            // Request "standard" profile which supports compressedAlpha and rgbaHalfFloat
            stage3D.requestContext3D(Context3DRenderMode.AUTO, "standard");

            vertexAssembly.assemble(Context3DProgramType.VERTEX, VERTEX_SHADER, 2);
            fragmentAssemblyDxt5.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER_DXT5, 2);
            fragmentAssemblyFloat.assemble(Context3DProgramType.FRAGMENT, FRAGMENT_SHADER_FLOAT, 2);
        }

        private function contextCreated(event:Event):void {
            renderContext = Stage3D(event.target).context3D;
            renderContext.enableErrorChecking = true;
            renderContext.configureBackBuffer(viewWidth, viewHeight, 4, true);

            var triangles:Vector.<uint> = Vector.<uint>([0, 1, 2, 0, 2, 3]);
            indexList = renderContext.createIndexBuffer(triangles.length);
            indexList.uploadFromVector(triangles, 0, triangles.length);

            const dataPerVertex:int = 5;
            var vertexData:Vector.<Number> = Vector.<Number>([
                -0.4, -0.4, 0,  0, 1,
                0.4, -0.4, 0,  1, 1,
                0.4, 0.4, 0,  1, 0,
                -0.4, 0.4, 0,  0, 0
            ]);
            vertexes = renderContext.createVertexBuffer(vertexData.length / dataPerVertex, dataPerVertex);
            vertexes.uploadFromVector(vertexData, 0, vertexData.length / dataPerVertex);

            renderContext.setVertexBufferAt(0, vertexes, 0, Context3DVertexBufferFormat.FLOAT_3);
            renderContext.setVertexBufferAt(1, vertexes, 3, Context3DVertexBufferFormat.FLOAT_2);

            var circleBitmap:BitmapData = new CIRCLE_PNG().bitmapData;

            var compressedTexture = renderContext.createTexture(
                512, 512,
                "compressedAlpha",
                false
            );
            compressedTexture.uploadFromBitmapData(circleBitmap);

            var halfFloatTexture = renderContext.createTexture(
                512, 512,
                "rgbaHalfFloat",
                false
            );
            halfFloatTexture.uploadFromBitmapData(circleBitmap);

            programDxt5 = renderContext.createProgram();
            programDxt5.upload(vertexAssembly.agalcode, fragmentAssemblyDxt5.agalcode);

            programFloat = renderContext.createProgram();
            programFloat.upload(vertexAssembly.agalcode, fragmentAssemblyFloat.agalcode);

            renderContext.setBlendFactors(Context3DBlendFactor.SOURCE_ALPHA, Context3DBlendFactor.ONE_MINUS_SOURCE_ALPHA);

            renderContext.clear(.3, .3, .3, 1, 1, 0);

            renderContext.setProgram(programDxt5);
            renderContext.setTextureAt(0, compressedTexture);
            renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([-0.5, 0.0, 0.0, 0.0]));
            renderContext.drawTriangles(indexList, 0, 2);

            renderContext.setProgram(programFloat);
            renderContext.setTextureAt(0, halfFloatTexture);
            renderContext.setProgramConstantsFromVector("vertex", 0, Vector.<Number>([0.5, 0.0, 0.0, 0.0]));
            renderContext.drawTriangles(indexList, 0, 2);

            renderContext.present();
        }
    }
}
