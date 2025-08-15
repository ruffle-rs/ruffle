package {

import flash.display.MovieClip;
import flash.display.Stage;

public class Test extends MovieClip {
    private var originalStage: Stage;
    private var originalRoot: MovieClip;

    public function Test() {
        super();

        originalStage = this.stage;
        originalRoot = MovieClip(this.root);

        printProps();
        trace("Removing the root movie and setting tabChildren to false");
        originalStage.removeChild(originalRoot);
        originalStage.tabChildren = false;
        printProps();
    }

    private function printProps():void {
        trace("stage.tabChildren = " + originalStage.tabChildren);
        trace("root.tabChildren = " + originalRoot.tabChildren);
    }
}
}
