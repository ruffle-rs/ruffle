/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}


/*
On an unset variable in a class inside a with block with jit and hybrid shells the value is null and
with interp the value is undefined.
*/

import com.adobe.test.Assert;
//     var SECTION = "Regression Tests";       // provide a document reference (ie, Actionscript section)
//     var VERSION = "AS3";        // Version of ECMAScript or ActionScript
//     var TITLE   = "Bug 615544";       // Provide ECMA section title or a description

    // 1st bug example using with
    var actualValue;
    class class1 {}
    class Bug615544 {
        public function Bug615544() {
            var cl:class1=new class1();
            var undef1;
            var undefR;
            with (cl) {
                undefR=undef1;
            }
            actualValue=undefR;
       }
    }

    // 2nd bug example
    var foo;
    class test2 {
        var bar;
    }
    var o=new test2();
    o.bar=foo;

    var testclass:Bug615544=new Bug615544();
    Assert.expectEq("bug 615544: unset variable in 'with' block  causes 'null/undefined inconsistency'", undefined, actualValue);
    Assert.expectEq("bug 615544: unset variable 'null/undefined' causes 'null/undefined inconsistency'", undefined, o.bar);


