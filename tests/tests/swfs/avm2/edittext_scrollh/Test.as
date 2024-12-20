package {
import flash.display.Sprite;
import flash.text.TextField;
import flash.text.TextFormat;

public class Test extends Sprite {
    public function Test() {
        stage.scaleMode = "noScale";
        var text = new TextField();
        text.width = 20;
        text.height = 20;
        addChild(text);

        trace(text.scrollH);
        text.scrollH = 1;
        trace(text.scrollH);
        text.scrollH = -1;
        trace(text.scrollH);

        text.text = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

        text.scrollH = 10;
        trace(text.scrollH);
        text.scrollH = 5;
        trace(text.scrollH);
        text.scrollH = -15;
        trace(text.scrollH);
        text.scrollH = -1;
        trace(text.scrollH);
        text.scrollH = 0;
        trace(text.scrollH);

        text.scrollH = 1.23;
        trace(text.scrollH);
        text.scrollH = 3.93;
        trace(text.scrollH);
    }
}
}
