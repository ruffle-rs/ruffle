package flash.display
{
    import flash.geom.Matrix;
    import __ruffle__.stub_method;

    // note: no need for an allocator, as it's never constructed from AS
    public final class Graphics
    {
        public function Graphics()
        {
            throw new Error("You cannot construct Graphics directly.");
        }

        public native function beginBitmapFill(bitmap:BitmapData, matrix:Matrix = null, repeat:Boolean = true, smooth:Boolean = false):void;
        public native function beginFill(color:uint, alpha:Number = 1.0):void;
        public native function beginGradientFill(
            type:String, colors:Array, alphas:Array, ratios:Array, matrix:Matrix = null, spreadMethod:String = "pad", interpolationMethod:String = "rgb", focalPointRatio:Number = 0
        ): void;
        public function beginShaderFill(shader:Shader, matrix:Matrix = null):void {
            stub_method("flash.display.Graphics", "beginShaderFill");
        }
        public native function clear(): void;
        public native function curveTo(controlX:Number, controlY:Number, anchorX:Number, anchorY:Number): void;
        public native function drawCircle(x:Number, y:Number, radius:Number): void;
        public native function drawEllipse(x:Number, y:Number, width:Number, height:Number): void;
        public native function drawRect(x:Number, y:Number, width:Number, height:Number): void;
        public native function drawRoundRect(x:Number, y:Number, width:Number, height:Number, ellipseWidth:Number, ellipseHeight:Number = NaN): void;
        public native function endFill(): void;
        public native function lineStyle(
            thickness:Number = NaN, color:uint = 0, alpha:Number = 1.0, pixelHinting:Boolean = false, scaleMode:String = "normal", caps:String = null, joints:String = null, miterLimit:Number = 3
        ): void;
        public native function lineTo(x:Number, y:Number): void;
        public native function moveTo(x:Number, y:Number): void;
        //public native function beginShaderFill(shader:Shader, matrix:Matrix = null):void;
        public native function lineGradientStyle(
            type:String, colors:Array, alphas:Array, ratios:Array, matrix:Matrix = null, spreadMethod:String = "pad", interpolationMethod:String = "rgb", focalPointRatio:Number = 0
        ):void;
        [API("674")]
        public native function cubicCurveTo(controlX1:Number, controlY1:Number, controlX2:Number, controlY2:Number, anchorX:Number, anchorY:Number):void;
        public native function copyFrom(sourceGraphics:Graphics):void;
        public native function drawPath(commands:Vector.<int>, data:Vector.<Number>, winding:String = "evenOdd"):void;
        public native function drawRoundRectComplex(
            x:Number, y:Number, width:Number, height:Number, topLeftRadius:Number, topRightRadius:Number, bottomLeftRadius:Number, bottomRightRadius:Number
        ):void;
        public native function drawTriangles(vertices:Vector.<Number>, indices:Vector.<int> = null, uvtData:Vector.<Number> = null, culling:String = "none"):void;
        public native function drawGraphicsData(graphicsData:Vector.<IGraphicsData>):void;
        //public native function lineShaderStyle(shader:Shader, matrix:Matrix = null):void;
        public native function lineBitmapStyle(bitmap:BitmapData, matrix:Matrix = null, repeat:Boolean = true, smooth:Boolean = false):void;
        [API("686")]
        public native function readGraphicsData(recurse:Boolean = true):Vector.<IGraphicsData>;
    }
}
