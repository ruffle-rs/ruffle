package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.display3D.Context3D;
    import flash.display3D.Context3DProgramType;
    import flash.display3D.Context3DVertexBufferFormat;
    import flash.display3D.IndexBuffer3D;
    import flash.display3D.Program3D;
    import flash.display3D.VertexBuffer3D;
    import flash.events.Event;
    import com.adobe.utils.AGALMiniAssembler; 

    public class Test extends Sprite {
        private var context3D:Context3D;
        private var vertexBuffer:VertexBuffer3D;
        private var indexBuffer:IndexBuffer3D;
        private var program:Program3D;

        public function Test() {
            stage.align = StageAlign.TOP_LEFT;
            stage.scaleMode = StageScaleMode.NO_SCALE;
            
            stage.stage3Ds[0].addEventListener(Event.CONTEXT3D_CREATE, onContextCreated);
            
            stage.stage3Ds[0].requestContext3D();
        }

        private function onContextCreated(event:Event):void {
            context3D = stage.stage3Ds[0].context3D;
            context3D.configureBackBuffer(stage.stageWidth, stage.stageHeight, 0, true);

            var vertexData:Vector.<Number> = Vector.<Number>([
                -0.6, -0.6, 0,  0, 0,
                 0.0,  0.6, 0,  0.5, 1,
                 0.6, -0.6, 0,  1, 0
            ]);
            
            vertexBuffer = context3D.createVertexBuffer(3, 5);
            vertexBuffer.uploadFromVector(vertexData, 0, 3);

            var indexData:Vector.<uint> = Vector.<uint>([0, 1, 2]);
            indexBuffer = context3D.createIndexBuffer(3);
            indexBuffer.uploadFromVector(indexData, 0, 3);

            var vertexShaderAssembler:AGALMiniAssembler = new AGALMiniAssembler();
            vertexShaderAssembler.assemble(Context3DProgramType.VERTEX,
                "mov op, va0\n" + // Pass position (va0) to output
                "mov v0, va1\n"   // Pass UV coordinates (va1) to varying v0
            );

            var fragmentShaderAssembler:AGALMiniAssembler = new AGALMiniAssembler();
            fragmentShaderAssembler.assemble(Context3DProgramType.FRAGMENT,
                "mov ft0, fc0\n" + 
                "tex ft1, v0, fs7 <2d, linear, mipnone>\n" + 
                "mov oc, ft0\n"
            );

            program = context3D.createProgram();
            program.upload(vertexShaderAssembler.agalcode, fragmentShaderAssembler.agalcode);

            // Solid red color
            var data:Vector.<Number> = Vector.<Number>([1, 0, 0, 1]);
            context3D.setProgramConstantsFromVector(Context3DProgramType.FRAGMENT, 0, data);

            context3D.clear(0, 0.4, 0.8, 1);

            // va0 = positions (index 0, 3 floats)
            context3D.setVertexBufferAt(0, vertexBuffer, 0, Context3DVertexBufferFormat.FLOAT_3);
            // va1 = UVs (index 3, 2 floats)
            context3D.setVertexBufferAt(1, vertexBuffer, 3, Context3DVertexBufferFormat.FLOAT_2);
            
            context3D.setProgram(program);

            context3D.drawTriangles(indexBuffer, 0, 1);
            context3D.present();
        }
    }
}
