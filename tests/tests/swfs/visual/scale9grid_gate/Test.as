package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    // Rotation, skew, a negative scale, or serving as a mask each disable slicing. Every
    // case is drawn twice, with and without a grid, and the two must match wherever
    // slicing is refused.
    [SWF(width="400", height="930", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        public function Test() {
            // Slices: the control row.
            pair(0, function (s:Shape):void {
                s.scaleX = 3;
                s.scaleY = 0.8;
            });

            // Refused: rotation.
            pair(1, function (s:Shape):void {
                s.scaleX = 3;
                s.scaleY = 0.8;
                s.rotation = 20;
            });

            // Refused: skew.
            pair(2, function (s:Shape):void {
                var m:* = s.transform.matrix;
                m.a = 3;
                m.d = 0.8;
                m.c = 0.4;
                s.transform.matrix = m;
            });

            // Refused: negative scale on one axis.
            pair(3, function (s:Shape):void {
                s.scaleX = -3;
                s.scaleY = 0.8;
            });

            // Refused: negative on both axes.
            pair(4, function (s:Shape):void {
                s.scaleX = -3;
                s.scaleY = -0.8;
            });

            // Unscaled: the map is the identity, so both must match.
            pair(5, function (s:Shape):void {
            });

            masks(430);
            ancestors(640);
        }

        // An ancestor's scale neither gates nor feeds the slice: the child goes by its
        // own matrix alone and the parent magnifies the result after the fact.
        private function ancestors(y:Number):void {
            // Parent 3x, unscaled gridded child: identity map, so the pair must match.
            for (var i:int = 0; i < 2; i++) {
                var held:Sprite = new Sprite();
                var still:Shape = paint();
                if (i == 0) {
                    still.scale9Grid = GRID;
                }
                held.addChild(still);
                held.scaleX = 3;
                held.x = 10;
                held.y = y + i * 70;
                addChild(held);
            }

            // Parent 3x, child 2x: sliced by two, magnified by three.
            for (i = 0; i < 2; i++) {
                var boosted:Sprite = new Sprite();
                var kid:Shape = paint();
                if (i == 0) {
                    kid.scale9Grid = GRID;
                }
                kid.scaleX = 2;
                boosted.addChild(kid);
                boosted.scaleX = 3;
                boosted.x = 10;
                boosted.y = y + 140 + i * 70;
                addChild(boosted);
            }
        }

        // Serving as a mask refuses slicing, so the top row's two columns must match; being
        // masked does not, so the bottom row's must differ. The art is concave because a
        // silhouette is all a mask contributes.
        private function masks(y:Number):void {
            for (var i:int = 0; i < 2; i++) {
                var gridded:Boolean = (i == 0);
                var x:Number = 10 + i * 195;

                var field:Shape = new Shape();
                field.graphics.beginFill(0xFFFFFF);
                field.graphics.drawRect(0, 0, 180, SIZE);
                field.graphics.endFill();
                var masker:Shape = notch(gridded);
                masker.scaleX = 3;
                addChild(hold(field, masker, x, y));
                field.mask = masker;

                var subject:Shape = notch(gridded);
                subject.scaleX = 3;
                var cover:Shape = new Shape();
                cover.graphics.beginFill(0x40D040);
                cover.graphics.drawRect(0, 0, 180, SIZE);
                cover.graphics.endFill();
                addChild(hold(subject, cover, x, y + 70));
                subject.mask = cover;
            }

            // The masker is a direct child of the gridded container this time, so the
            // grid reaches it by inheritance rather than being set on it. Serving as a
            // mask still refuses slicing, so both columns must match.
            for (i = 0; i < 2; i++) {
                var inherited:Sprite = new Sprite();
                var masked:Shape = new Shape();
                masked.graphics.beginFill(0xFFFFFF);
                masked.graphics.drawRect(0, 0, 180, SIZE);
                masked.graphics.endFill();
                var child:Shape = notch(false);
                inherited.addChild(masked);
                inherited.addChild(child);
                if (i == 0) {
                    inherited.scale9Grid = GRID;
                }
                inherited.scaleX = 3;
                inherited.x = 10 + i * 195;
                inherited.y = y + 140;
                addChild(inherited);
                masked.mask = child;
            }
        }

        private function hold(under:Shape, over:Shape, x:Number, y:Number):Sprite {
            var s:Sprite = new Sprite();
            s.addChild(under);
            s.addChild(over);
            s.x = x;
            s.y = y;
            return s;
        }

        // A rectangle with a notch cut from the bottom middle, between the grid lines.
        private function notch(gridded:Boolean):Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0xD04040);
            s.graphics.moveTo(0, 0);
            s.graphics.lineTo(SIZE, 0);
            s.graphics.lineTo(SIZE, SIZE);
            s.graphics.lineTo(40, SIZE);
            s.graphics.lineTo(40, 40);
            s.graphics.lineTo(20, 40);
            s.graphics.lineTo(20, SIZE);
            s.graphics.lineTo(0, SIZE);
            s.graphics.endFill();
            if (gridded) {
                s.scale9Grid = GRID;
            }
            return s;
        }

        private function pair(index:int, apply:Function):void {
            var y:Number = 30 + index * 60;

            var gridded:Shape = paint();
            gridded.scale9Grid = GRID;
            apply(gridded);
            gridded.x = 190;
            gridded.y = y;
            addChild(gridded);

            var plain:Shape = paint();
            apply(plain);
            plain.x = 190;
            plain.y = y + 30;
            addChild(plain);
        }

        private function paint():Shape {
            var s:Shape = new Shape();
            var colours:Array = [0xD04040, 0x40D040, 0x4040D0];
            for (var i:int = 0; i < 3; i++) {
                s.graphics.beginFill(colours[i]);
                s.graphics.drawRect(i * 20, 0, 20, SIZE);
                s.graphics.endFill();
            }
            return s;
        }
    }
}
