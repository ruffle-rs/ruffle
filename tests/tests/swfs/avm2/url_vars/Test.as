package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}

import flash.net.URLVariables;

var variables:URLVariables = new URLVariables();

variables.foo = "a";
trace(variables);

variables = new URLVariables();
variables.foo = [0,1,2];
trace(variables);

variables = new URLVariables();
variables["😭"] = [0,"😭",2];
trace(variables);


