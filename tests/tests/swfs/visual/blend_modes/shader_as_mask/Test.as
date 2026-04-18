package {

import flash.events.*;
import flash.display.*;
import flash.geom.*;

[SWF(width="40", height="60", backgroundColor="#FFFFFF")]
public class Test extends MovieClip {
    [Embed(source = "shader.pbj", mimeType="application/octet-stream")]
    public static var ShaderBytes: Class;
    [Embed(source = "bitmap1.png")]
    private static const Bitmap1:Class;
    [Embed(source = "bitmap2.png")]
    private static const Bitmap2:Class;

    public function Test() {
        super();

        var bg:Sprite = new Sprite();
        bg.graphics.beginFill(0x000000);
        bg.graphics.drawRect(0, 0, 40, 60);
        bg.graphics.endFill();
        addChild(bg);

        var d:DisplayObject;
        addChild(createMask(true));
        d = createMask(false);
        d.x = 20;
        addChild(d);

        addChild(createMaskeeWithMask(false, false, 0,  20));
        addChild(createMaskeeWithMask(true,  false, 20, 20));

        addChild(createMaskeeWithMask(false, true, 0,  40));
        addChild(createMaskeeWithMask(true,  true, 20, 40));
    }

    private function createMaskeeWithMask(cab:Boolean, shader:Boolean, x:Number = 0, y:Number = 0):MovieClip {
        var c:MovieClip = new MovieClip();

        var maskee:Sprite = createMaskee();
        var mask:MovieClip = createMask(shader);
        mask.cacheAsBitmap = cab;
        maskee.cacheAsBitmap = cab;
        maskee.mask = mask;

        c.addChild(maskee);
        c.addChild(mask);
        c.x = x;
        c.y = y;
        return c;
    }

    private function createMaskee():Sprite {
        var maskee:Sprite = new Sprite();
        maskee.graphics.beginFill(0xFF00FF);
        maskee.graphics.drawRect(0, 0, 40, 40);
        maskee.graphics.endFill();
        return maskee;
    }

    private function createMask(shader:Boolean):MovieClip {
        var mask:MovieClip = new MovieClip();

        var colorA:Bitmap = new Bitmap(Bitmap(new Bitmap1()).bitmapData);

        var colorB:Bitmap = new Bitmap(Bitmap(new Bitmap2()).bitmapData);

        if (shader) {
            colorB.blendShader = new Shader(new ShaderBytes());
            colorB.blendMode = BlendMode.SHADER;
        }

        mask.addChild(colorA);
        mask.addChild(colorB);

        return mask;
    }
}
}
