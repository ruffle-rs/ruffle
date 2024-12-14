package flash.display {
    import flash.geom.Rectangle;
    import flash.geom.ColorTransform;
    import flash.geom.Point;
    import flash.geom.Matrix;
    import flash.filters.BitmapFilter;
    import flash.filters.ShaderFilter;
    import flash.utils.ByteArray;
    import __ruffle__.stub_method;

    [Ruffle(InstanceAllocator)]
    public class BitmapData implements IBitmapDrawable {
        public function BitmapData(width:int, height:int, transparent:Boolean = true, fillColor:uint = 0xFFFFFFFF) {
            this.init(width, height, transparent, fillColor);
        }

        private native function init(width:int, height:int, transparent:Boolean, fillColor:uint);

        public native function get height():int;
        public native function get width():int;
        public native function get rect():Rectangle;
        public native function get transparent():Boolean;

        public native function getPixels(rect:Rectangle):ByteArray;
        [API("682")]
        public native function copyPixelsToByteArray(rect:Rectangle, data:ByteArray):void;
        public native function getVector(rect:Rectangle):Vector.<uint>;
        public native function getPixel(x:int, y:int):uint;
        public native function getPixel32(x:int, y:int):uint;
        public native function setPixel(x:int, y:int, color:uint):void;
        public native function setPixel32(x:int, y:int, color:uint):void;
        public native function setPixels(rect:Rectangle, inputByteArray:ByteArray):void;
        public native function setVector(rect:Rectangle, inputVector:Vector.<uint>):void;
        public native function copyChannel(sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, sourceChannel:uint, destChannel:uint):void;
        public native function floodFill(x:int, y:int, color:uint):void;
        public native function noise(randomSeed:int, low:uint = 0, high:uint = 255, channelOptions:uint = 7, grayScale:Boolean = false):void;
        public native function colorTransform(rect:Rectangle, colorTransform:ColorTransform):void;
        public native function getColorBoundsRect(mask:uint, color:uint, findColor:Boolean = true):Rectangle;
        public native function scroll(x:int, y:int):void;
        public native function lock():void;
        public native function hitTest(firstPoint:Point, firstAlphaThreshold:uint, secondObject:Object, secondBitmapDataPoint:Point = null, secondAlphaThreshold:uint = 1):Boolean;
        public function histogram(rect:Rectangle = null): Vector.<Vector.<Number>> {
            if (!rect) {
                rect = this.rect;
            }

            var a = new Vector.<Number>(256);
            var r = new Vector.<Number>(256);
            var g = new Vector.<Number>(256);
            var b = new Vector.<Number>(256);

            var pixels = getPixels(rect);
            for (var i = 0; i < pixels.length; i += 4) {
                a[pixels[i]]++;
                r[pixels[i + 1]]++;
                g[pixels[i + 2]]++;
                b[pixels[i + 3]]++;
            }

            var result = new Vector.<Vector.<Number>>(4);
            result[0] = r;
            result[1] = g;
            result[2] = b;
            result[3] = a;
            return result;
        }
        public native function unlock(changeRect:Rectangle = null):void;
        public native function copyPixels(
            sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, alphaBitmapData:BitmapData = null, alphaPoint:Point = null, mergeAlpha:Boolean = false
        ):void;
        public native function draw(
            source:IBitmapDrawable, matrix:Matrix = null, colorTransform:ColorTransform = null, blendMode:String = null, clipRect:Rectangle = null, smoothing:Boolean = false
        ):void;
        [API("680")]
        public native function drawWithQuality(
            source:IBitmapDrawable, matrix:Matrix = null, colorTransform:ColorTransform = null, blendMode:String = null, clipRect:Rectangle = null, smoothing:Boolean = false, quality:String = null
        ):void;
        public native function fillRect(rect:Rectangle, color:uint):void;
        public native function dispose():void;
        public native function applyFilter(sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, filter:BitmapFilter):void;
        public native function clone():BitmapData;
        public native function paletteMap(
            sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, redArray:Array = null, greenArray:Array = null, blueArray:Array = null, alphaArray:Array = null
        ):void;
        public native function perlinNoise(
            baseX:Number, baseY:Number, numOctaves:uint, randomSeed:int, stitch:Boolean, fractalNoise:Boolean, channelOptions:uint = 7, grayScale:Boolean = false, offsets:Array = null
        ):void;
        public native function threshold(
            sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, operation:String, threshold:uint, color:uint = 0, mask:uint = 0xFFFFFFFF, copySource:Boolean = false
        ):uint;
        public native function compare(otherBitmapData:BitmapData):Object;
        public native function pixelDissolve(
            sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, randomSeed:int = 0, numPixels:int = 0,
            fillColor:uint = 0
        ):int;
        public native function merge(
            sourceBitmapData:BitmapData, sourceRect:Rectangle, destPoint:Point, redMultiplier:uint, greenMultiplier:uint, blueMultiplier:uint, alphaMultiplier:uint
        ):void 
        public function generateFilterRect(sourceRect:Rectangle, filter:BitmapFilter):Rectangle {
            // Flash always reports that a ShaderFilter affects the entire BitampData, ignoring SourceRect.
            if (filter is ShaderFilter) {
                return this.rect.clone();
            }
            stub_method("flash.display.BitmapData", "generateFilterRect");
            return sourceRect.clone();
        }
    }
}
