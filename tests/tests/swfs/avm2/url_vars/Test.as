package {
    import flash.display.MovieClip;
    public class Test extends MovieClip {
        public function Test() {

        }
    }
}

import flash.net.URLVariables;
import flash.utils.getQualifiedClassName;

var variables:URLVariables = new URLVariables();

variables.foo = "a";
trace(variables);

variables = new URLVariables();
variables.foo = [0,1,2];
trace(variables);
trace(variables.foo);
variables.decode("foo=3&foo=4");
trace(variables.foo);

variables = new URLVariables();
variables["😭"] = [0,"😭",2];
trace(variables);
trace(variables["😭"]);

variables = new URLVariables("hello=world&test=%F0%9F%98%80");
// Ruffle's property iteration order is not consistent with Flash's (yet)
trace(variables.toString().split("&").sort());
variables.decode("test=1&hello%3Dworld=hi%21&test=equals%3Dtest");
trace(variables.hello);
trace(variables["hello=world"]);
trace(variables.test);

variables = new URLVariables();
trace(variables.empty);
variables.empty = null;
trace(variables.empty);
variables.decode("empty=10");
trace(variables.empty);
trace(typeof variables.empty);
variables.testArr1 = [];
variables.testArr2 = ["hi"];
variables.decode("testArr1=1&testArr2=bye");
trace(variables["testArr1"]);
trace(variables["testArr2"]);

variables = new URLVariables("=");
trace(variables);
variables.decode("a=&=b");
trace(variables.a);
trace(variables[""]);

variables = new URLVariables("hello=world=2");
trace(variables.toString().split("&").sort());
variables["test space"] = "s p a c e !";
variables.test = {"hello": "world"};
trace(variables["test space"]);
trace(variables.test);

variables = new URLVariables("test=1&hello%3Dworld=world&te+st=equals%3Dtest&test=2+3");
trace(variables.toString().split("&").sort());
variables.decode("test=4&te%20st=hi+bye");
trace(variables.test);
trace(variables["te st"]);
variables.decode("te+st=hi&test=1");
trace(variables["test"]);
trace(variables["te st"]);
