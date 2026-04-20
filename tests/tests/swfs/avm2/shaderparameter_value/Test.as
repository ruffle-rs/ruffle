// compiled with mxmlc

package {
    import flash.display.Sprite;
    import flash.events.TextEvent;
    import flash.text.TextField;
    import flash.text.TextFieldType;
    import flash.utils.*;
    import flash.geom.*;
    import flash.net.*;
    import flash.display.*;
    import flash.system.*;
    import flash.utils.*;

    public class Test extends Sprite {
        public function Test() {
            super();
        }
    }
}


import flash.display.*;

var param = new ShaderParameter();
trace(param.value);
var arr = [1, 2, 3];
param.value = arr;
trace(param.value);
trace(param.value === arr);
param.value[0] = 7;
trace(param.value);
