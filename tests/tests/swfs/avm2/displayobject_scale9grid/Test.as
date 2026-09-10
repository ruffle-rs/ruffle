package {
    import flash.display.Bitmap;
    import flash.display.BitmapData;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.SimpleButton;
    import flash.display.Sprite;
    import flash.geom.Point;
    import flash.geom.Rectangle;
    import flash.text.TextField;

    [SWF(width="800", height="600", backgroundColor="#000000")]
    public class Test extends MovieClip {
        public function Test() {
            var shape:Sprite = box();
            trace("initial: " + shape.scale9Grid);

            // Accepted only when strictly inside the bounds.
            set("inside", shape, new Rectangle(20, 20, 20, 20));
            set("touching left", shape, new Rectangle(0, 20, 20, 20));
            set("touching right", shape, new Rectangle(20, 20, 40, 20));
            set("equal to bounds", shape, new Rectangle(0, 0, 60, 60));
            set("outside", shape, new Rectangle(10, 10, 100, 100));
            set("zero width", shape, new Rectangle(20, 20, 0, 20));
            set("negative width", shape, new Rectangle(20, 20, -10, 20));

            // Non-finite fields are refused.
            set("infinite x", shape, new Rectangle(Infinity, 20, 20, 20));
            set("nan width", shape, new Rectangle(20, 20, NaN, 20));
            set("huge", shape, new Rectangle(1e8, 0, 1e8, 1));

            // The setter rounds each field to the nearest twip; the getter truncates to
            // whole pixels.
            set("quantise up", shape, new Rectangle(10.99, 10.99, 20.01, 20.01));
            set("quantise down", shape, new Rectangle(10.9, 10.9, 20.01, 20.01));
            set("half twip", shape, new Rectangle(10.025, 10.025, 20, 20));
            // Width truncates independently of x: (10, 10, 39, 39), not (10, 10, 40, 40).
            set("half pixel", shape, new Rectangle(10.5, 10.5, 39.5, 39.5));

            shape.scale9Grid = new Rectangle(20, 20, 20, 20);
            shape.scale9Grid = null;
            trace("cleared: " + shape.scale9Grid);

            // Bounds are the object's own geometry plus its direct children only.
            var deep:Sprite = new Sprite();
            var mid:Sprite = new Sprite();
            mid.addChild(box());
            deep.addChild(mid);
            trace("grandchild getBounds: " + deep.getBounds(deep));
            set("art two levels down", deep, new Rectangle(20, 20, 20, 20));

            // Text is excluded from those bounds even though getBounds reports it.
            var texty:Sprite = new Sprite();
            var tf:TextField = new TextField();
            tf.width = 60;
            tf.height = 60;
            tf.background = true;
            texty.addChild(tf);
            trace("textfield getBounds: " + texty.getBounds(texty));
            set("textfield child", texty, new Rectangle(20, 20, 20, 20));
            set("textfield itself", tf, new Rectangle(20, 20, 20, 20));

            // Strokes do not inflate what the grid may cover.
            var stroked:Sprite = new Sprite();
            stroked.graphics.lineStyle(16, 0x000000);
            stroked.graphics.drawRect(0, 0, 60, 60);
            trace("stroked getBounds: " + stroked.getBounds(stroked));
            set("stroke inflated edge", stroked, new Rectangle(-4, -4, 68, 68));

            // SimpleButton bounds come from the state it is currently showing.
            var button:SimpleButton = new SimpleButton();
            button.upState = box();
            button.overState = box();
            button.downState = box();
            button.hitTestState = box();
            set("button", button, new Rectangle(20, 20, 20, 20));
            set("empty button", new SimpleButton(), new Rectangle(20, 20, 20, 20));

            // A bitmap accepts a grid and ignores it when rendering.
            set("bitmap", new Bitmap(new BitmapData(60, 60, false, 0)),
                new Rectangle(20, 20, 20, 20));

            // Stage refuses with a different error code.
            set("stage", stage, new Rectangle(20, 20, 20, 20));

            geometry();
            hitRegions();
        }

        // Bounds, hit testing and coordinate conversion all follow the plainly scaled shape.
        private function geometry():void {
            var plain:Sprite = box();
            plain.scaleX = 3;
            measure("ungridded 3x", plain);

            var wide:Sprite = box();
            wide.scale9Grid = new Rectangle(20, 20, 20, 20);
            wide.scaleX = 3;
            measure("gridded 3x", wide);

            // Scaled below the combined corner width; the reported width must still be
            // the plainly scaled one, not the sliced one.
            var squeezed:Sprite = box();
            squeezed.scale9Grid = new Rectangle(20, 20, 20, 20);
            squeezed.scaleX = 0.25;
            measure("gridded 0.25x", squeezed);

            // An L, so at y=40 only the left bar is drawn. Sliced it stays 20 wide, plainly
            // scaled it reaches 60, and hit testing follows the latter: x=30 through x=50
            // report a hit over pixels the sliced shape never paints.
            var ell:Sprite = new Sprite();
            ell.graphics.beginFill(0x0000FF);
            ell.graphics.drawRect(0, 0, 20, 60);
            ell.graphics.drawRect(0, 0, 60, 20);
            ell.graphics.endFill();
            ell.scale9Grid = new Rectangle(20, 20, 20, 20);
            ell.scaleX = 3;
            stage.addChild(ell);

            var xs:Array = [10, 30, 40, 50, 70, 130, 170];
            for (var i:int = 0; i < xs.length; i++) {
                trace("hit x=" + xs[i]
                    + ": shape " + ell.hitTestPoint(xs[i], 40, true)
                    + ", box " + ell.hitTestPoint(xs[i], 40, false));
            }
            trace("localToGlobal: " + ell.localToGlobal(new Point(20, 20)));
            trace("globalToLocal: " + ell.globalToLocal(new Point(60, 20)));
            stage.removeChild(ell);
        }

        // A grid never moves the region that answers the mouse, whether it sits on the
        // shape, on a button's hit state, or on the button itself.
        private function hitRegions():void {
            var plain:SimpleButton = new SimpleButton();
            plain.upState = notch(false);
            plain.hitTestState = notch(false);
            pick("button plain", plain);

            var griddedHit:SimpleButton = new SimpleButton();
            griddedHit.upState = notch(false);
            griddedHit.hitTestState = notch(true);
            pick("button gridded hit", griddedHit);

            var griddedSelf:SimpleButton = new SimpleButton();
            griddedSelf.upState = notch(false);
            griddedSelf.hitTestState = notch(false);
            griddedSelf.scale9Grid = new Rectangle(20, 20, 20, 20);
            pick("button gridded self", griddedSelf);
        }

        private function pick(name:String, b:SimpleButton):void {
            b.scaleX = 3;
            stage.addChild(b);
            var hits:String = "";
            var xs:Array = [10, 40, 90, 140, 170];
            for (var i:int = 0; i < xs.length; i++) {
                hits += (b.hitTestPoint(xs[i], 50, true) ? "1" : "0");
            }
            trace(name + ": " + hits);
            stage.removeChild(b);
        }

        private function notch(gridded:Boolean):Shape {
            var s:Shape = new Shape();
            s.graphics.beginFill(0xFF0000);
            s.graphics.moveTo(0, 0);
            s.graphics.lineTo(60, 0);
            s.graphics.lineTo(60, 60);
            s.graphics.lineTo(40, 60);
            s.graphics.lineTo(40, 40);
            s.graphics.lineTo(20, 40);
            s.graphics.lineTo(20, 60);
            s.graphics.lineTo(0, 60);
            s.graphics.endFill();
            if (gridded) {
                s.scale9Grid = new Rectangle(20, 20, 20, 20);
            }
            return s;
        }

        private function measure(name:String, s:Sprite):void {
            var holder:Sprite = new Sprite();
            holder.addChild(s);
            trace(name + ": w=" + s.width + " h=" + s.height + " " + s.getBounds(holder));
        }

        private function box():Sprite {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF0000);
            s.graphics.drawRect(0, 0, 60, 60);
            s.graphics.endFill();
            return s;
        }

        private function set(name:String, o:Object, rect:Rectangle):void {
            try {
                o.scale9Grid = rect;
                trace(name + ": " + o.scale9Grid);
                o.scale9Grid = null;
            } catch (e:Error) {
                trace(name + ": threw " + e.getStackTrace());
            }
        }
    }
}
