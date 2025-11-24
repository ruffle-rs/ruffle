package flash.display {
    import __ruffle__.stub_method;

    import flash.geom.Matrix;

    [Ruffle(Abstract)]
    public final class Graphics {
        public function Graphics() {
            throw new Error("You cannot construct Graphics directly.");
        }

        public native function beginBitmapFill(
            bitmap:BitmapData,
            matrix:Matrix = null,
            repeat:Boolean = true,
            smooth:Boolean = false
        ):void;

        [Ruffle(FastCall)]
        public native function beginFill(color:uint, alpha:Number = 1.0):void;

        public native function beginGradientFill(
            type:String,
            colors:Array,
            alphas:Array,
            ratios:Array,
            matrix:Matrix = null,
            spreadMethod:String = "pad",
            interpolationMethod:String = "rgb",
            focalPointRatio:Number = 0
        ):void;

        [API("662")]
        public function beginShaderFill(shader:Shader, matrix:Matrix = null):void {
            stub_method("flash.display.Graphics", "beginShaderFill");
        }

        [Ruffle(FastCall)]
        public native function clear():void;

        public native function curveTo(controlX:Number, controlY:Number, anchorX:Number, anchorY:Number):void;

        public native function drawCircle(x:Number, y:Number, radius:Number):void;

        public native function drawEllipse(x:Number, y:Number, width:Number, height:Number):void;

        public native function drawRect(x:Number, y:Number, width:Number, height:Number):void;

        public native function drawRoundRect(
            x:Number,
            y:Number,
            width:Number,
            height:Number,
            ellipseWidth:Number,
            ellipseHeight:Number = NaN
        ):void;

        [Ruffle(FastCall)]
        public native function endFill():void;

        public native function lineStyle(
            thickness:Number = NaN,
            color:uint = 0,
            alpha:Number = 1.0,
            pixelHinting:Boolean = false,
            scaleMode:String = "normal",
            caps:String = null,
            joints:String = null,
            miterLimit:Number = 3
        ):void;

        [Ruffle(FastCall)]
        public native function lineTo(x:Number, y:Number):void;

        [Ruffle(FastCall)]
        public native function moveTo(x:Number, y:Number):void;

        public native function lineGradientStyle(
            type:String,
            colors:Array,
            alphas:Array,
            ratios:Array,
            matrix:Matrix = null,
            spreadMethod:String = "pad",
            interpolationMethod:String = "rgb",
            focalPointRatio:Number = 0
        ):void;

        [API("674")]
        public native function cubicCurveTo(
            controlX1:Number,
            controlY1:Number,
            controlX2:Number,
            controlY2:Number,
            anchorX:Number,
            anchorY:Number
        ):void;

        [API("662")]
        public native function copyFrom(sourceGraphics:Graphics):void;

        [API("662")]
        public native function drawPath(commands:Vector.<int>, data:Vector.<Number>, winding:String = "evenOdd"):void;

        public native function drawRoundRectComplex(
            x:Number,
            y:Number,
            width:Number,
            height:Number,
            topLeftRadius:Number,
            topRightRadius:Number,
            bottomLeftRadius:Number,
            bottomRightRadius:Number
        ):void;

        [API("662")]
        public native function drawTriangles(
            vertices:Vector.<Number>,
            indices:Vector.<int> = null,
            uvtData:Vector.<Number> = null,
            culling:String = "none"
        ):void;

        [API("662")]
        public native function drawGraphicsData(graphicsData:Vector.<IGraphicsData>):void;

        [API("662")]
        public function lineShaderStyle(shader:Shader, matrix:Matrix = null):void {
            stub_method("flash.display.Graphics", "lineShaderStyle");
        }

        [API("662")]
        public native function lineBitmapStyle(
            bitmap:BitmapData,
            matrix:Matrix = null,
            repeat:Boolean = true,
            smooth:Boolean = false
        ):void;

        [API("686")]
        public native function readGraphicsData(recurse:Boolean = true):Vector.<IGraphicsData>;
    }
}
