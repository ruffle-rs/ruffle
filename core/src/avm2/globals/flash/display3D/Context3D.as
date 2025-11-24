package flash.display3D {
    import __ruffle__.stub_method;
    import __ruffle__.stub_getter;
    import __ruffle__.stub_setter;

    import flash.display.BitmapData;
    import flash.events.EventDispatcher;
    import flash.geom.Matrix3D;
    import flash.geom.Rectangle;
    import flash.utils.ByteArray;
    import flash.display3D.textures.CubeTexture;
    import flash.display3D.textures.TextureBase;
    import flash.display3D.textures.RectangleTexture;
    import flash.display3D.textures.Texture;
    import flash.display3D.textures.VideoTexture;

    [API("674")]
    [Ruffle(Abstract)]
    public final class Context3D extends EventDispatcher {
        public native function clear(
            red:Number = 0.0,
            green:Number = 0.0,
            blue:Number = 0.0,
            alpha:Number = 1.0,
            depth:Number = 1.0,
            stencil:uint = 0,
            mask:uint = 0xffffffff
        ):void;

        public native function createIndexBuffer(numIndices:int, bufferUsage:String = "staticDraw"):IndexBuffer3D;

        public native function createVertexBuffer(
            numVertices:int,
            data32PerVertex:int,
            bufferUsage:String = "staticDraw"
        ):VertexBuffer3D;

        public native function configureBackBuffer(
            width:int,
            height:int,
            antiAlias:int,
            enableDepthAndStencil:Boolean = true,
            wantsBestResolution:Boolean = false,
            wantsBestResolutionOnBrowserZoom:Boolean = false
        ):void;

        public native function setVertexBufferAt(
            index:int,
            buffer:VertexBuffer3D,
            bufferOffset:int = 0,
            format:String = "float4"
        ):void;

        public native function createProgram():Program3D;

        public native function setProgram(program:Program3D):void;

        public native function drawTriangles(
            indexBuffer:IndexBuffer3D,
            firstIndex:int = 0,
            numTriangles:int = -1
        ):void;

        public native function present():void;

        public native function setCulling(triangleFaceToCull:String):void;

        public native function createTexture(
            width:int, height:int,
            format:String,
            optimizeForRenderToTexture:Boolean,
            streamingLevels:int = 0
        ):Texture;

        public native function createCubeTexture(
            size:int,
            format:String,
            optimizeForRenderToTexture:Boolean,
            streamingLevels:int = 0
        ):CubeTexture;

        [API("690")]
        public native function createRectangleTexture(
            width:int,
            height:int,
            format:String,
            optimizeForRenderToTexture:Boolean
        ):RectangleTexture;

        [API("706")]
        public function createVideoTexture():VideoTexture {
            stub_method("flash.display3D.Context3D", "createVideoTexture");
            return null;
        }

        [API("706")]
        public static function get supportsVideoTexture():Boolean {
            return false;
        }

        public function get driverInfo():String {
            stub_getter("flash.display3D.Context3D", "driverInfo");
            return "Dummy Ruffle driver";
        }

        private var _enableErrorChecking:Boolean = false;
        public function get enableErrorChecking():Boolean {
            return this._enableErrorChecking;
        }
        public function set enableErrorChecking(enable:Boolean):void {
            this._enableErrorChecking = enable;
        }

        public native function setProgramConstantsFromMatrix(
            programType:String,
            firstRegister:int,
            matrix:Matrix3D,
            transposedMatrix:Boolean = false
        ):void;

        public native function setProgramConstantsFromVector(
            programType:String,
            firstRegister:int,
            data:Vector.<Number>,
            numRegisters:int = -1
        ):void;

        [API("676")]
        public function setProgramConstantsFromByteArray(
            programType:String,
            firstRegister:int,
            numRegisters:int,
            data:ByteArray,
            dataOffset:uint
        ):void {
            stub_method("flash.display3D.Context3D", "setProgramConstantsFromByteArray");
        }

        public native function setColorMask(red:Boolean, green:Boolean, blue:Boolean, alpha:Boolean):void;

        public native function setDepthTest(depthMask:Boolean, passCompareMode:String):void;

        public native function setScissorRectangle(rectangle:Rectangle):void;

        public native function setRenderToBackBuffer():void;

        public native function setBlendFactors(sourceFactor:String, destinationFactor:String):void;

        public native function setTextureAt(sampler:int, texture:TextureBase):void;

        // NOTE docs say 694, but playerglobals says 692; playerglobals is correct
        [API("692")]
        public native function get profile():String;

        [API("700")]
        public function get backBufferWidth():int {
            stub_getter("flash.display3D.Context3D", "backBufferWidth");
            return 2048;
        }

        [API("700")]
        public function get backBufferHeight():int {
            stub_getter("flash.display3D.Context3D", "backBufferHeight");
            return 2048;
        }

        [API("700")]
        public function get maxBackBufferWidth():int {
            stub_getter("flash.display3D.Context3D", "maxBackBufferWidth");
            return 2048;
        }
        [API("700")]
        public function set maxBackBufferWidth(value:int):void {
            stub_setter("flash.display3D.Context3D", "maxBackBufferWidth");
        }

        [API("700")]
        public function get maxBackBufferHeight():int {
            stub_getter("flash.display3D.Context3D", "maxBackBufferHeight");
            return 2048;
        }
        [API("700")]
        public function set maxBackBufferHeight(value:int):void {
            stub_setter("flash.display3D.Context3D", "maxBackBufferHeight");
        }

        public function setStencilReferenceValue(referenceValue:uint, readMask:uint = 255, writeMask:uint = 255):void {
            stub_method("flash.display3D.Context3D", "setStencilReferenceValue");
        }

        [API("686")]
        public native function setSamplerStateAt(sampler:int, wrap:String, filter:String, mipfilter:String):void;

        public native function setRenderToTexture(
            texture:TextureBase,
            enableDepthAndStencil:Boolean = false,
            antiAlias:int = 0,
            surfaceSelector:int = 0,
            colorOutputIndex:int = 0
        ):void;

        public function setStencilActions(
            triangleFace:String = "frontAndBack",
            compareMode:String = "always",
            actionOnBothPass:String = "keep",
            actionOnDepthFail:String = "keep",
            actionOnDepthPassStencilFail:String = "keep"
        ):void {
            stub_method("flash.display3D.Context3D", "setStencilActions");
        }

        public function drawToBitmapData(destination:BitmapData):void {
            stub_method("flash.display3D.Context3D", "drawToBitmapData");
        }

        public native function dispose(recreate:Boolean = true):void;
    }
}
