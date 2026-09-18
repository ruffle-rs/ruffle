package {
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.events.Event;
    import flash.geom.Rectangle;

    // Everything here renders once, then changes on the first frame event: rescaled,
    // regridded, cleared, cached, redrawn, and a cached child under a container's grid.
    // The capture is of the state after the change, so any slice served from a stale
    // cache shows up.
    [SWF(width="220", height="600", backgroundColor="#101010")]
    public class Test extends MovieClip {
        private static const SIZE:Number = 60;
        private static const GRID:Rectangle = new Rectangle(20, 20, 20, 20);

        private var rescaled:Shape;
        private var regridded:Shape;
        private var cleared:Shape;
        private var cached:Shape;
        private var redrawn:Shape;
        private var extended:Shape;
        private var holder:Sprite;
        private var toggled:Sprite;
        private var togglee:Shape;
        private var done:Boolean = false;

        public function Test() {
            rescaled = add(0);
            regridded = add(1);
            cleared = add(2);
            cached = add(3);
            cached.cacheAsBitmap = true;

            // A container whose grid arrives only after its cached child has rendered.
            // The child must stay plain: a bitmap-cached child is exempt from the
            // parent's grid no matter when the grid was set.
            holder = new Sprite();
            var child:Shape = notch();
            child.cacheAsBitmap = true;
            holder.addChild(child);
            holder.scaleX = 3;
            holder.x = 20;
            holder.y = 20 + 4 * 70;
            addChild(holder);

            // Redrawn in place with the notch on the other edge. The bounds, grid and
            // scale are unchanged, so only dropping the drawing's cached slice keeps the
            // old geometry off the screen.
            redrawn = add(5);

            // Drawn onto rather than cleared first, so the drawing is only marked dirty
            // by the new commands. The extra fill sits inside the existing bounds, so the
            // cache key is unchanged and the slice has to be dropped on its own account.
            extended = add(6);

            // The child flips from sliced to exempt when cacheAsBitmap arrives on the
            // frame event, and the cached container's bitmap has to follow.
            toggled = new Sprite();
            togglee = notch();
            toggled.addChild(togglee);
            toggled.scale9Grid = GRID;
            toggled.scaleX = 3;
            toggled.cacheAsBitmap = true;
            toggled.x = 20;
            toggled.y = 20 + 7 * 70;
            addChild(toggled);

            addEventListener(Event.ENTER_FRAME, change);
        }

        private function change(e:Event):void {
            if (done) {
                return;
            }
            done = true;
            rescaled.scaleX = 1.5;
            regridded.scale9Grid = new Rectangle(5, 5, 50, 50);
            cleared.scale9Grid = null;
            cached.scale9Grid = new Rectangle(5, 5, 50, 50);
            holder.scale9Grid = GRID;
            redrawn.graphics.clear();
            paint(redrawn, true);
            extended.graphics.beginFill(0x2020C0);
            extended.graphics.drawRect(20, SIZE - 20, 20, 20);
            extended.graphics.endFill();
            togglee.cacheAsBitmap = true;
        }

        private function add(index:int):Shape {
            var s:Shape = notch();
            s.scale9Grid = GRID;
            s.scaleX = 3;
            s.x = 20;
            s.y = 20 + index * 70;
            addChild(s);
            return s;
        }

        private function notch():Shape {
            var s:Shape = new Shape();
            paint(s, false);
            return s;
        }

        // A square with a notch cut out of the bottom edge, or the top when flipped. Both
        // fill the same 60x60 bounds, so a stale slice is the only reason the wrong one
        // could still be on screen.
        private function paint(s:Shape, flipped:Boolean):void {
            var near:Number = flipped ? SIZE : 0;
            var far:Number = flipped ? 0 : SIZE;
            var step:Number = flipped ? 20 : -20;
            s.graphics.beginFill(0xC02020);
            s.graphics.moveTo(0, near);
            s.graphics.lineTo(SIZE, near);
            s.graphics.lineTo(SIZE, far);
            s.graphics.lineTo(SIZE - 20, far);
            s.graphics.lineTo(SIZE - 20, far + step);
            s.graphics.lineTo(20, far + step);
            s.graphics.lineTo(20, far);
            s.graphics.lineTo(0, far);
            s.graphics.endFill();
        }
    }
}
