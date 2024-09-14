package flash.display {

    public final class GraphicsPath implements IGraphicsPath, IGraphicsData {
        public var commands : Vector.<int>;
        public var data : Vector.<Number>;
        public var winding : String;

        public function GraphicsPath(commands:Vector.<int> = null, data:Vector.<Number> = null, winding:String = "evenOdd") {
            this.commands = commands;
            this.data = data;
            this.winding = winding;
        }

        [API("674")] // The online docs say 694, but that's a lie. This is the correct number from playerglobal.swc.
        public function cubicCurveTo(controlX1:Number, controlY1:Number, controlX2:Number, controlY2:Number, anchorX:Number, anchorY:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            commands.push(GraphicsPathCommand.CUBIC_CURVE_TO);
            data.push(controlX1, controlY1, controlX2, controlY2, anchorX, anchorY);
        }

        public function curveTo(controlX:Number, controlY:Number, anchorX:Number, anchorY:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            commands.push(GraphicsPathCommand.CURVE_TO);
            data.push(controlX, controlY, anchorX, anchorY);
        }

        public function lineTo(x:Number, y:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            commands.push(GraphicsPathCommand.LINE_TO);
            data.push(x, y);
        }

        public function moveTo(x:Number, y:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            commands.push(GraphicsPathCommand.MOVE_TO);
            data.push(x, y);
        }

        public function wideLineTo(x:Number, y:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            // "Wide" variant seems to literally just exist to use the same amount of data values as curveTo
            // The first two values are arbitrary. When consuming, they are ignored.
            // https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/GraphicsPathCommand.html#WIDE_LINE_TO
            commands.push(GraphicsPathCommand.WIDE_LINE_TO);
            data.push(0, 0, x, y);
        }

        public function wideMoveTo(x:Number, y:Number):void {
            if (commands == null) {
                commands = new Vector.<int>();
            }
            if (data == null) {
                data = new Vector.<Number>();
            }
            // "Wide" variant seems to literally just exist to use the same amount of data values as curveTo
            // The first two values are arbitrary. When consuming, they are ignored.
            // https://help.adobe.com/en_US/FlashPlatform/reference/actionscript/3/flash/display/GraphicsPathCommand.html#WIDE_MOVE_TO
            commands.push(GraphicsPathCommand.WIDE_MOVE_TO);
            data.push(0, 0, x, y);
        }
    }

}