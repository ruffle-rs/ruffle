package {
    import flash.display.DisplayObjectContainer;
    import flash.display.MovieClip;
    import flash.display.Shape;
    import flash.display.Sprite;
    import flash.geom.Rectangle;
    import flash.text.TextField;

    public class Test extends MovieClip {

        [Embed(source="morph_shape.swf", symbol="MorphContainer")]
        private static const MorphContainerCls:Class;

        public function Test() {
            super();
            run_clear_cases();
            run_nonfinite_cases();
            run_geometry_cases();
            run_target_type_cases();
        }

        private static function tryCase(label:String, fn:Function):void {
            try {
                fn();
                trace(label + ": ok");
            } catch (e:ArgumentError) {
                trace(label + ": Got [object ArgumentError] id " + e.errorID);
            } catch (e:Error) {
                trace(label + ": unexpected " + e + " id=" + e.errorID);
            }
        }

        private static function shapeSprite(w:Number = 1000, h:Number = 1000):Sprite {
            var s:Sprite = new Sprite();
            s.graphics.beginFill(0xFF0000);
            s.graphics.drawRect(0, 0, w, h);
            s.graphics.endFill();
            return s;
        }

        private static function fmtRect(r:Rectangle):String {
            return "(x=" + r.x + ", y=" + r.y + ", w=" + r.width + ", h=" + r.height + ")";
        }

        private function run_clear_cases():void {
            var s1:Sprite = shapeSprite();
            s1.scale9Grid = new Rectangle(5, 5, 50, 50);
            tryCase("set_initial", function():void {
                if (s1.scale9Grid == null) throw new Error("expected non-null after set");
            });
            tryCase("null_clears", function():void {
                s1.scale9Grid = null;
                if (s1.scale9Grid != null) throw new Error("expected null after clear");
            });
        }

        private function run_nonfinite_cases():void {
            var s:Sprite = shapeSprite();
            tryCase("nan_x",   function():void { s.scale9Grid = new Rectangle(NaN,  0, 10, 10); });
            tryCase("nan_y",   function():void { s.scale9Grid = new Rectangle(0,  NaN, 10, 10); });
            tryCase("nan_w",   function():void { s.scale9Grid = new Rectangle(0,    0, NaN, 10); });
            tryCase("nan_h",   function():void { s.scale9Grid = new Rectangle(0,    0, 10, NaN); });
            tryCase("pos_inf", function():void { s.scale9Grid = new Rectangle( Infinity, 0, 10, 10); });
            tryCase("neg_inf", function():void { s.scale9Grid = new Rectangle(-Infinity, 0, 10, 10); });
        }

        private function run_geometry_cases():void {
            var s_small:Sprite = shapeSprite(100, 100);
            tryCase("out_of_bounds", function():void {
                s_small.scale9Grid = new Rectangle(-1000, -1000, 5000, 5000);
            });

            var s_big:Sprite = shapeSprite();
            tryCase("pixel_floor", function():void {
                s_big.scale9Grid = new Rectangle(10.5, 79.7, 100, 200);
                trace("pixel_floor_readback: " + fmtRect(s_big.scale9Grid));
            });

            var s_round:Sprite = shapeSprite();
            tryCase("roundtrip", function():void {
                s_round.scale9Grid = new Rectangle(5, 5, 50, 50);
                trace("roundtrip_readback: " + fmtRect(s_round.scale9Grid));
            });
        }

        private function run_target_type_cases():void {
            tryCase("target_is_shape", function():void {
                var sh:Shape = new Shape();
                sh.graphics.beginFill(0x00FF00);
                sh.graphics.drawRect(0, 0, 100, 100);
                sh.graphics.endFill();
                sh.scale9Grid = new Rectangle(10, 10, 50, 50);
            });

            tryCase("direct_graphic_child", function():void {
                var parent:Sprite = new Sprite();
                var sh:Shape = new Shape();
                sh.graphics.beginFill(0x0000FF);
                sh.graphics.drawRect(0, 0, 100, 100);
                sh.graphics.endFill();
                parent.addChild(sh);
                parent.scale9Grid = new Rectangle(10, 10, 50, 50);
            });

            tryCase("direct_morph_child", function():void {
                var morph:DisplayObjectContainer = new MorphContainerCls() as DisplayObjectContainer;
                morph.scale9Grid = new Rectangle(180, 150, 50, 50);
            });

            tryCase("nested_shape_descendant", function():void {
                var outer:Sprite = new Sprite();
                var inner:Sprite = new Sprite();
                var sh:Shape = new Shape();
                sh.graphics.beginFill(0xFF00FF);
                sh.graphics.drawRect(0, 0, 100, 100);
                sh.graphics.endFill();
                inner.addChild(sh);
                outer.addChild(inner);
                outer.scale9Grid = new Rectangle(10, 10, 50, 50);
            });

            tryCase("textfield_only_child", function():void {
                var parent:Sprite = new Sprite();
                var tf:TextField = new TextField();
                tf.width = 200;
                tf.height = 200;
                tf.text = "hello";
                parent.addChild(tf);
                parent.scale9Grid = new Rectangle(10, 10, 50, 50);
            });
        }
    }
}
