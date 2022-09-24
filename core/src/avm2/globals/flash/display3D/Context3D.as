package flash.display3D {
    import flash.events.EventDispatcher;
    import flash.geom.Matrix3D;
    import flash.geom.Rectangle;
    import flash.display3D.textures.CubeTexture;
    import flash.display3D.textures.TextureBase;
    import flash.display3D.textures.RectangleTexture;
    import flash.display3D.textures.Texture;
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;

    public final class Context3D extends EventDispatcher {
        public native function clear(red:Number = 0.0, green:Number = 0.0, blue:Number = 0.0, alpha:Number = 1.0, depth:Number = 1.0, stencil:uint = 0, mask:uint = 0xffffffff):void;

        public native function createIndexBuffer(numIndices:int, bufferUsage:String = "staticDraw"):IndexBuffer3D;
        public native function createVertexBuffer(numVertices:int, data32PerVertex:int, bufferUsage:String = "staticDraw"):VertexBuffer3D;
        public native function configureBackBuffer(
            width:int, height:int, antiAlias:int, enableDepthAndStencil:Boolean = true, wantsBestResolution:Boolean = false, wantsBestResolutionOnBrowserZoom:Boolean = false
        ):void;
        public native function setVertexBufferAt(index:int, buffer:VertexBuffer3D, bufferOffset:int = 0, format:String = "float4"):void
        public native function createProgram():Program3D;
        public native function setProgram(program:Program3D):void;
        public native function drawTriangles(indexBuffer:IndexBuffer3D, firstIndex:int = 0, numTriangles:int = -1):void;
        public native function present():void;
        public native function setCulling(triangleFaceToCull:String):void;
        public native function createTexture(width:int, height:int, format:String, optimizeForRenderToTexture:Boolean, streamingLevels:int = 0):Texture;
        public native function createCubeTexture(size:int, format:String, optimizeForRenderToTexture:Boolean, streamingLevels:int = 0):CubeTexture;
        public native function createRectangleTexture(width:int, height:int, format:String, optimizeForRenderToTexture:Boolean):RectangleTexture;

        public function get driverInfo():String {
            stub_getter("flash.display3D.Context3D", "driverInfo");
            return "Dummy Ruffle driver";
        }

        public var enableErrorChecking:Boolean = true;

        public native function setProgramConstantsFromMatrix(programType:String, firstRegister:int, matrix:Matrix3D, transposedMatrix:Boolean = false):void;
        public native function setProgramConstantsFromVector(programType:String, firstRegister:int, data:Vector.<Number>, numRegisters:int = -1):void;

        public function setDepthTest(depthMask:Boolean, passCompareMode:String):void {
            stub_method("flash.display3D.Context3D", "setDepthTest");
        }
        public function setScissorRectangle(rectangle:Rectangle):void {
            stub_method("flash.display3D.Context3D", "setScissorRectangle");
        }

        public function setRenderToBackBuffer():void {
            stub_method("flash.display3D.Context3D", "setRenderToBackBuffer");
        }

        public function setBlendFactors(sourceFactor:String, destinationFactor:String):void {
            stub_method("flash.display3D.Context3D", "setBlendFactors");
        }

        public native function setTextureAt(sampler:int, texture:TextureBase):void;

        public function get profile():String {
            stub_getter("flash.display3D.Context3D", "profile");
            return "baseline";
        }

        public function get maxBackBufferWidth():int {
            stub_getter("flash.display3D.Context3D", "maxBackBufferWidth");
            return 2048;
        }

         public function get maxBackBufferHeight():int {
            stub_getter("flash.display3D.Context3D", "maxBackBufferHeight");
            return 2048;
         }
    }
}