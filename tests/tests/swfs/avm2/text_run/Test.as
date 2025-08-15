package {
import flash.display.Sprite;
import flash.text.TextRun;
import flash.text.TextFormat;

public class Test extends Sprite {
    public function Test() {
        var run = new TextRun(1, 2, new TextFormat());
        trace(run);
        trace(run.beginIndex);
        trace(run.endIndex);
        trace(run.textFormat);
        run.beginIndex = 45;
        run.endIndex = 54;
        run.textFormat = null;
        trace(run.beginIndex);
        trace(run.endIndex);
        trace(run.textFormat);
    }
}
}
