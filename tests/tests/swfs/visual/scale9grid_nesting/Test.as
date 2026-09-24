package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.SimpleButton;
    import flash.display.Sprite;
    import flash.geom.Rectangle;

    // A container's grid reaches its own drawing and its direct Shape children and stops
    // there; a child is sliced in the container's space, so its matrix folds into the map.
    [SWF(width="330", height="1255", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 90;
        private static const GRID:Rectangle = new Rectangle(20, 20, 50, 50);

        public function Test() {
            // Own drawing plus a direct Shape child: both slice.
            var mixed:Sprite = frame();
            mixed.addChild(inner(0x40A0E0));
            place(mixed, 0);

            // Direct child carrying its own translate.
            var shifted:Sprite = frame();
            var moved:Shape = inner(0xE0A040);
            moved.x = 14;
            shifted.addChild(moved);
            place(shifted, 1);

            // Art two levels down: the grandchild plain-scales, the container's art slices.
            var deep:Sprite = frame();
            var mid:Sprite = new Sprite();
            mid.addChild(inner(0x60C060));
            deep.addChild(mid);
            place(deep, 2);

            // A child with its own grid uses that instead of inheriting the parent's.
            var owns:Sprite = frame();
            var self:Shape = inner(0xC04060);
            self.scale9Grid = new Rectangle(24, 4, 12, 14);
            owns.addChild(self);
            place(owns, 3);

            button(4, true);
            button(5, false);

            // An ancestor's rotation does not gate slicing: the gridded child slices in
            // its own space and the sliced result turns with the parent, while its plain
            // twin below just stretches.
            var tilted:Sprite = new Sprite();
            var sliced:Sprite = frame();
            sliced.scale9Grid = GRID;
            sliced.scaleX = 2.6;
            var plain:Sprite = frame();
            plain.scaleX = 2.6;
            plain.y = 105;
            tilted.addChild(sliced);
            tilted.addChild(plain);
            tilted.rotation = 20;
            tilted.x = 46;
            tilted.y = 12 + 6 * 105;
            addChild(tilted);

            // A child's own linear matrix folds into the map: rotation, skew and a flip
            // all still slice, conjugated through the child's transform.
            var turned:Sprite = frame();
            var spun:Shape = inner(0x40C0A0);
            spun.rotation = 45;
            spun.x = 30;
            turned.addChild(spun);
            place(turned, 7);

            var sheared:Sprite = frame();
            var slanted:Shape = inner(0xA0C040);
            var m:* = slanted.transform.matrix;
            m.c = 0.3;
            slanted.transform.matrix = m;
            sheared.addChild(slanted);
            place(sheared, 8);

            var flipped:Sprite = frame();
            var mirrored:Shape = inner(0xC0A040);
            mirrored.scaleX = -1;
            mirrored.x = 64;
            flipped.addChild(mirrored);
            place(flipped, 9);
        }

        private function place(art:Sprite, index:int):void {
            art.scale9Grid = GRID;
            art.scaleX = 2.6;
            art.scaleY = 1.0;
            art.x = 12;
            art.y = 12 + index * 105;
            addChild(art);
        }

        // A button is sliced from the state it is showing, which is one level further down
        // than a child: the state is a Shape, so it is remapped rather than plain-scaled.
        private function button(index:int, gridded:Boolean):void {
            var b:SimpleButton = new SimpleButton();
            b.upState = bands(new Shape());
            b.overState = bands(new Shape());
            b.downState = bands(new Shape());
            b.hitTestState = bands(new Shape());
            if (gridded) {
                b.scale9Grid = GRID;
            }
            b.scaleX = 2.6;
            b.scaleY = 1.0;
            b.x = 12;
            b.y = 12 + index * 105;
            addChild(b);
        }

        // Full-size background whose middle band is exactly the grid's centre column.
        private function frame():Sprite {
            return bands(new Sprite());
        }

        private function bands(s:*):* {
            var edges:Array = [0, 20, 70, SIZE];
            for (var i:int = 0; i < 3; i++) {
                s.graphics.beginFill(i == 1 ? 0x503070 : 0x8040C0);
                s.graphics.drawRect(edges[i], 0, edges[i + 1] - edges[i], SIZE);
                s.graphics.endFill();
            }
            return s;
        }

        private function inner(tint:uint):Shape {
            var s:Shape = new Shape();
            var edges:Array = [0, 12, 40, 52];
            for (var i:int = 0; i < 3; i++) {
                s.graphics.beginFill(i == 1 ? tint : (tint ^ 0x303030));
                s.graphics.drawRect(edges[i], 0, edges[i + 1] - edges[i], 22);
                s.graphics.endFill();
            }
            s.y = 34;
            return s;
        }
    }
}
