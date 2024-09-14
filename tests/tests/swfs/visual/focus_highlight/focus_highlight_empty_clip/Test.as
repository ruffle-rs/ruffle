package {

import flash.display.MovieClip;
import flash.display.Sprite;

[SWF(width="50", height="50", backgroundColor="#000000")]
public class Test extends MovieClip {
    public function Test() {
        super();

        var sprite:Sprite = new Sprite();
        sprite.graphics.beginFill(0xFF00FF);
        sprite.graphics.drawRect(0, 0, 20, 20);

        // 1. Create a movie clip and add it as a child.
        var clip:MovieClip = new MovieClip();
        var clip2:MovieClip = new MovieClip();
        addChild(clip);
        clip.addChild(clip2);

        clip.x = 15;
        clip.y = 15;

        // 2. Focus the empty clip.
        stage.focus = clip;

        // 3. Add some content to the clip after being focused.
        clip2.addChild(sprite);
    }
}
}
