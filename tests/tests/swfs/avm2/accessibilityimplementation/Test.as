package {

import flash.display.*;
import flash.accessibility.*;
import flash.geom.*;

public class Test extends Sprite {
    public function Test() {
        var ai:AccessibilityImplementation = new AccessibilityImplementation();
        trace(ai.errno);
        trace(ai.stub);

        ai.accDoDefaultAction(4);
        ai.accSelect(4, 4);
        trace("accLocation(): " + ai.accLocation(5));
        trace("get_accDefaultAction(): " + ai.get_accDefaultAction(5));
        trace("get_accFocus(): " + ai.get_accFocus());
        trace("get_accName(): " + ai.get_accName(5));
        trace("get_accSelection(): " + ai.get_accSelection());
        trace("get_accValue(): " + ai.get_accValue(5));
        trace("getChildIDArray(): " + ai.getChildIDArray());
        trace("isLabeledBy(): " + ai.isLabeledBy(new Rectangle()));

        try {
            trace("get_accRole(): " + ai.get_accRole(5));
        } catch (e) {
            trace("get_accRole(): " + e.getStackTrace());
        }

        try {
            trace("get_accState(): " + ai.get_accState(5));
        } catch (e) {
            trace("get_accState(): " + e.getStackTrace());
        }
    }
}

}
