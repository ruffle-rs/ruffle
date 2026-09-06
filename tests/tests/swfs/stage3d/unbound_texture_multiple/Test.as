package {
    import flash.display.Sprite;
    import flash.display.BitmapData;
    import flash.display.StageAlign;
    import flash.display.StageScaleMode;
    import flash.display3D.*;
    import flash.display3D.textures.Texture;
    import flash.events.Event;
    import com.adobe.utils.AGALMiniAssembler; 

    public class Test extends Sprite {
        private var context3D:Context3D;
        
        // Triangle 1: Proper texture bound (left side)
        private var vbNormal:VertexBuffer3D;
        private var ibNormal:IndexBuffer3D;
        private var textureNormal:Texture;
        
        // Triangle 2: Missing texture binding (right side)
        private var vbUnbound:VertexBuffer3D;
        private var ibUnbound:IndexBuffer3D;

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

            // Left Triangle Data
            var dataNormal:Vector.<Number> = Vector.<Number>([
                -0.8, -0.5, 0,  0, 1,
                -0.5,  0.5, 0,  0.5, 0,
                -0.2, -0.5, 0,  1, 1
            ]);
            vbNormal = context3D.createVertexBuffer(3, 5);
            vbNormal.uploadFromVector(dataNormal, 0, 3);
            
            ibNormal = context3D.createIndexBuffer(3);
            ibNormal.uploadFromVector(Vector.<uint>([0, 1, 2]), 0, 3);

            // Right Triangle Data
            var dataUnbound:Vector.<Number> = Vector.<Number>([
                 0.2, -0.5, 0,  0, 1,
                 0.5,  0.5, 0,  0.5, 0,
                 0.8, -0.5, 0,  1, 1
            ]);
            vbUnbound = context3D.createVertexBuffer(3, 5);
            vbUnbound.uploadFromVector(dataUnbound, 0, 3);
            
            ibUnbound = context3D.createIndexBuffer(3);
            ibUnbound.uploadFromVector(Vector.<uint>([0, 1, 2]), 0, 3);

            // Create a 2x2 solid red texture for the valid triangle
            textureNormal = context3D.createTexture(2, 2, "bgra", false);
            var bmd:BitmapData = new BitmapData(2, 2, false, 0xFFFF0000);
            textureNormal.uploadFromBitmapData(bmd);

            // Common Shader: Samples fs0 and multiplies with vertex texture coordinates
            var vertexShaderAssembler:AGALMiniAssembler = new AGALMiniAssembler();
            vertexShaderAssembler.assemble(Context3DProgramType.VERTEX,
                "mov op, va0\n" + 
                "mov v0, va1\n"
            );

            var fragmentShaderAssembler:AGALMiniAssembler = new AGALMiniAssembler();
            fragmentShaderAssembler.assemble(Context3DProgramType.FRAGMENT,
                "tex oc, v0, fs0 <2d, linear, mipnone>\n"
            );

            program = context3D.createProgram();
            program.upload(vertexShaderAssembler.agalcode, fragmentShaderAssembler.agalcode);

            context3D.clear(0, 0.4, 0.8, 1);
            context3D.setProgram(program);

            // Draw triangle 1
            context3D.setVertexBufferAt(0, vbNormal, 0, Context3DVertexBufferFormat.FLOAT_3);
            context3D.setVertexBufferAt(1, vbNormal, 3, Context3DVertexBufferFormat.FLOAT_2);
            context3D.setTextureAt(0, textureNormal); // Bind valid texture to fs0
            context3D.drawTriangles(ibNormal, 0, 1);

            // Draw triangle 2
            context3D.setVertexBufferAt(0, vbUnbound, 0, Context3DVertexBufferFormat.FLOAT_3);
            context3D.setVertexBufferAt(1, vbUnbound, 3, Context3DVertexBufferFormat.FLOAT_2);
            context3D.setTextureAt(0, null); // Clear sampler fs0 (unbound state)
            context3D.drawTriangles(ibUnbound, 0, 1);

            context3D.present();
        }
    }
}
