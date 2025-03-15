package {
    import flash.display.*;
    import flash.geom.*;

    [SWF(width="100", height="100")]
    public class Test extends MovieClip {
        public function Test() {
            // this -> s1
            var s1: Sprite = new Sprite();
            s1.name = "s1";
            s1.graphics.beginFill(0xFF0000);
            s1.graphics.drawRect(0, 0, 10, 10);
            this.addChild(s1);

            // this -> s2
            var s2: Sprite = new Sprite();
            s2.name = "s2";
            s2.graphics.beginFill(0xFF0000);
            s2.graphics.drawRect(0, 0, 10, 10);
            this.addChild(s2);

            // this -> s1 -> s3
            var s3: Sprite = new Sprite();
            s3.name = "s3";
            s3.graphics.beginFill(0xFF0000);
            s3.graphics.drawRect(0, 0, 10, 10);
            s1.addChild(s3);

            // this -> s2 -> s4
            var s4: Sprite = new Sprite();
            s4.name = "s4";
            s4.graphics.beginFill(0xFF0000);
            s4.graphics.drawRect(0, 0, 10, 10);
            s2.addChild(s4);

            // this -> s1 -> s3 -> s5
            var s5: Sprite = new Sprite();
            s5.name = "s5";
            s5.graphics.beginFill(0xFF0000);
            s5.graphics.drawRect(0, 0, 10, 10);
            s3.addChild(s5);

            // invisible
            var s6: Sprite = new Sprite();
            s6.name = "s6";
            s6.graphics.beginFill(0xFF0000);
            s6.graphics.drawRect(0, 0, 10, 10);
            s6.visible = false
            this.addChild(s6);

            // not child
            var s7: Sprite = new Sprite();
            s7.name = "s7";
            s7.graphics.beginFill(0xFF0000);
            s7.graphics.drawRect(0, 0, 10, 10);

            // this -> s8(no graphics) -> s9
            var s8: Sprite = new Sprite();
            s8.name = "s8";
            this.addChild(s8);
            var s9: Sprite = new Sprite();
            s9.name = "s9";
            s9.graphics.beginFill(0xFF0000);
            s9.graphics.drawRect(0, 0, 10, 10);
            s8.addChild(s9);

            // mask
            var mask: Sprite = new Sprite();
            mask.name = "mask";
            mask.graphics.beginFill(0xFF0000);
            mask.graphics.drawRect(0, 0, 10, 10);
            this.addChild(mask);

            // masked
            var masked: Sprite = new Sprite();
            masked.name = "masked";
            masked.graphics.beginFill(0xFF0000);
            masked.graphics.drawRect(0, 0, 10, 10);
            masked.mask = mask;
            this.addChild(masked);

            trace("this.hitTestPoint", this.hitTestPoint(5, 5));
            trace("s1.hitTestPoint", s1.hitTestPoint(5, 5));
            trace("s2.hitTestPoint", s2.hitTestPoint(5, 5));
            trace("s3.hitTestPoint", s3.hitTestPoint(5, 5));
            trace("s4.hitTestPoint", s4.hitTestPoint(5, 5));
            trace("s5.hitTestPoint", s5.hitTestPoint(5, 5));
            trace("s6.hitTestPoint", s6.hitTestPoint(5, 5));
            trace("s7.hitTestPoint", s7.hitTestPoint(5, 5));
            trace("s8.hitTestPoint", s8.hitTestPoint(5, 5));
            trace("s9.hitTestPoint", s9.hitTestPoint(5, 5));
            trace("mask.hitTestPoint", mask.hitTestPoint(5, 5));
            trace("masked.hitTestPoint", masked.hitTestPoint(5, 5));

            trace("getObjectsUnderPoint():")
            var result: Array = this.getObjectsUnderPoint(new Point(5, 5));
            trace(result);
            trace(result.map(function (e) { return e.name; }).join(","));
        }
    }
}
