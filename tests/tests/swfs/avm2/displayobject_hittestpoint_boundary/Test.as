package {
  import flash.display.*;
  import flash.text.*;
  import flash.media.*;

  [SWF(width="500", height="250")]
    public class Test extends MovieClip {
      public function Test() {
        trace("");
        TestOctagonCounterClock();

        trace("");
        TestOctagonClock();

        trace("");
        DrawUpwardCurve();
        DrawDownwardCurve();
        TestHitTestImageComparison();
      }

      public function TestOctagonCounterClock() {
        var xx = [10, 0, 0, 10, 20, 30, 30, 20, 10];
        var yy = [0, 10, 20, 30, 30, 20, 10, 0, 0];

        var s: Sprite = new Sprite();
        this.addChild(s);

        s.x = 10;
        s.y = 10;
        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(xx[0], yy[0]);
        for(var i=1; i<=8; i++) {
          s.graphics.lineTo(xx[i], yy[i]);
        }
        s.graphics.endFill();

        for(var i=0; i<8; i++) {
          var j = (i+1) % 8;
          trace(
              "s.hitTestPoint(" + s.x + "+" + xx[i] + ", " + s.y + "+" + yy[i] + ", true):",
              s.hitTestPoint(s.x + xx[i], s.y + yy[i], true)
              );
          trace(
              "s.hitTestPoint(" + s.x + "+" + (xx[i]+xx[j])/2 + ", " + s.y + "+" + (yy[i]+yy[j])/2 + ", true):",
              s.hitTestPoint(s.x + (xx[i]+xx[j])/2, s.y + (yy[i]+yy[j])/2, true)
              );
        }
        for(var i=0; i<8; i++) {
          var j = (i+1) % 8;
          trace(
              "s.hitTestPoint(" + s.x + "+" + xx[i] + ", " + s.y + "+" + yy[i] + ", false):",
              s.hitTestPoint(s.x + xx[i], s.y + yy[i], false)
              );
          trace(
              "s.hitTestPoint(" + s.x + "+" + (xx[i]+xx[j])/2 + ", " + s.y + "+" + (yy[i]+yy[j])/2 + ", false):",
              s.hitTestPoint(s.x + (xx[i]+xx[j])/2, s.y + (yy[i]+yy[j])/2, false)
              );
        }
      }

      public function TestOctagonClock() {
        var xx = [10, 0, 0, 10, 20, 30, 30, 20, 10];
        var yy = [0, 10, 20, 30, 30, 20, 10, 0, 0];

        var s: Sprite = new Sprite();
        this.addChild(s);

        s.x = 50;
        s.y = 10;
        s.graphics.beginFill(0x000000);
        s.graphics.moveTo(xx[8], yy[8]);
        for(var i=7; i>=0; i--) {
          s.graphics.lineTo(xx[i], yy[i]);
        }
        s.graphics.endFill();

        for(var i=0; i<8; i++) {
          var j = (i+1) % 8;
          trace(
              "s.hitTestPoint(" + s.x + "+" + xx[i] + ", " + s.y + "+" + yy[i] + ", true):",
              s.hitTestPoint(s.x + xx[i], s.y + yy[i], true)
              );
          trace(
              "s.hitTestPoint(" + s.x + "+" + (xx[i]+xx[j])/2 + ", " + s.y + "+" + (yy[i]+yy[j])/2 + ", true):",
              s.hitTestPoint(s.x + (xx[i]+xx[j])/2, s.y + (yy[i]+yy[j])/2, true)
              );
        }
        for(var i=0; i<8; i++) {
          var j = (i+1) % 8;
          trace(
              "s.hitTestPoint(" + s.x + "+" + xx[i] + ", " + s.y + "+" + yy[i] + ", false):",
              s.hitTestPoint(s.x + xx[i], s.y + yy[i], false)
              );
          trace(
              "s.hitTestPoint(" + s.x + "+" + (xx[i]+xx[j])/2 + ", " + s.y + "+" + (yy[i]+yy[j])/2 + ", false):",
              s.hitTestPoint(s.x + (xx[i]+xx[j])/2, s.y + (yy[i]+yy[j])/2, false)
              );
        }
      }

      public function TestHitTestImageComparison() {
        var b = new BitmapData(500, 500);
        for(var i=0; i<500; i++) {
          for(var j=0; j<500; j++) {
            if(this.hitTestPoint(i, j, true)) {
              b.setPixel(i, j, 0xFF0000);
            }
          }
        }
        addChild(new Bitmap(b));
      }

      public function DrawUpwardCurve() {
        var s: Sprite = new Sprite();
        this.addChild(s);

        s.x = 0;
        s.y = 0;
        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(100, 50);
        s.graphics.curveTo(50, 50, 50, 100);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(200, 50);
        s.graphics.curveTo(175, 0, 150, 100);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(250, 50);
        s.graphics.curveTo(275, 0, 300, 100);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(350, 50);
        s.graphics.curveTo(400, 50, 400, 100);
        s.graphics.endFill();
      }

      public function DrawDownwardCurve() {
        var s: Sprite = new Sprite();
        this.addChild(s);

        s.x = 0;
        s.y = 150;
        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(100, 50);
        s.graphics.curveTo(50, 50, 50, 0);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(200, 50);
        s.graphics.curveTo(175, 100, 150, 0);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(250, 50);
        s.graphics.curveTo(275, 100, 300, 0);
        s.graphics.endFill();

        s.graphics.beginFill(0x00FFFF);
        s.graphics.moveTo(350, 50);
        s.graphics.curveTo(400, 50, 400, 0);
        s.graphics.endFill();
      }
    }
}
