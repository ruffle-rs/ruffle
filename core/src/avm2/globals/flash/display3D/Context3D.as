package flash.display3D {
    import flash.events.EventDispatcher;
    import flash.geom.Matrix3D;

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

        // FIXME - implement this
        public function get driverInfo():String {
            return "Dummy Ruffle driver";
        }

        public var enableErrorChecking:Boolean = true;

        public native function setProgramConstantsFromMatrix(programType:String, firstRegister:int, matrix:Matrix3D, transposedMatrix:Boolean = false):void;
    }
}