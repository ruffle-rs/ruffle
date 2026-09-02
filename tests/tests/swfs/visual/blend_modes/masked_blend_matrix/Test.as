package {
    import flash.display.BlendMode;
    import flash.display.Shape;
    import flash.display.Sprite;

    [SWF(width="588", height="168", backgroundColor="#E0E0E0", frameRate="12")]
    public class Test extends Sprite {
        private static const MODES:Array = [
            BlendMode.NORMAL,
            BlendMode.LAYER,
            BlendMode.MULTIPLY,
            BlendMode.SCREEN,
            BlendMode.LIGHTEN,
            BlendMode.DARKEN,
            BlendMode.DIFFERENCE,
            BlendMode.ADD,
            BlendMode.SUBTRACT,
            BlendMode.INVERT,
            BlendMode.ALPHA,
            BlendMode.ERASE,
            BlendMode.OVERLAY,
            BlendMode.HARDLIGHT,
        ];

        public function Test() {
            for (var row:int = 0; row < 4; row++) {
                for (var column:int = 0; column < MODES.length; column++) {
                    addCell(column * 42, row * 42, MODES[column], row);
                }
            }
        }

        private function addCell(
            x:Number,
            y:Number,
            mode:String,
            maskCase:int
        ):void {
            var cell:Sprite = new Sprite();
            cell.x = x;
            cell.y = y;
            addChild(cell);

            cell.addChild(rectangle(2, 2, 38, 38, 0x174A7E));

            var group:Sprite = new Sprite();
            cell.addChild(group);

            var foreground:Sprite = new Sprite();
            foreground.addChild(rectangle(6, 6, 30, 30, 0xE85D75));
            foreground.blendMode = mode;
            group.addChild(foreground);

            if (maskCase == 1 || maskCase == 3) {
                var foregroundMask:Shape = rectangle(17, 2, 17, 38, 0xFFFFFF);
                group.addChild(foregroundMask);
                foreground.mask = foregroundMask;
            }

            if (maskCase == 2 || maskCase == 3) {
                var groupMask:Shape = rectangle(2, 13, 38, 19, 0xFFFFFF);
                cell.addChild(groupMask);
                group.mask = groupMask;
            }

            // Alpha and Erase only have an effect when a containing Layer exists.
            if (mode == BlendMode.ALPHA || mode == BlendMode.ERASE) {
                cell.blendMode = BlendMode.LAYER;
            }
        }

        private function rectangle(
            x:Number,
            y:Number,
            width:Number,
            height:Number,
            color:uint
        ):Shape {
            var result:Shape = new Shape();
            result.graphics.beginFill(color);
            result.graphics.drawRect(x, y, width, height);
            result.graphics.endFill();
            return result;
        }
    }
}
