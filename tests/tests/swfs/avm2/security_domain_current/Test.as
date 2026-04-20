package {
import flash.display.*;
import flash.system.*;

public class Test extends MovieClip {
    public function Test() {
        trace(SecurityDomain.currentDomain);
        trace(SecurityDomain.currentDomain === SecurityDomain.currentDomain);
    }
}
}
