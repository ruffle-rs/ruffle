package  {

import flash.display.MovieClip;

public class Test extends MovieClip {
    
    public function Test() {
        try {
            addFrameScript();
        } catch(e) {
            trace(e.getStackTrace());
        }

        try {
            addFrameScript(1);
        } catch(e) {
            trace(e.getStackTrace());
        }

        try {
            addFrameScript(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        } catch(e) {
            trace(e.getStackTrace());
        }
    }

}

}
