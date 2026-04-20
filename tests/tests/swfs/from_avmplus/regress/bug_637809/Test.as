/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;


// var SECTION = "Regression Tests";
// var VERSION = "AS3";
// var TITLE   = "Bug 637809";



function foo(type:int, ...additionalArgs):String {
    var message:String;
    switch (type) {
        case 1:
            message = "foo";
            break;
        case 2:
            message = additionalArgs[0];
            break;
    }
    return message;
}

function bar(d, ...args) {
    args[0]
    switch (1) {
        case 1:
        case 2: break;
    }
    return d.z as Array
}

bar({});

Assert.expectEq('foo(1)', "foo", foo(1));
Assert.expectEq('foo(2)', null, foo(2));
Assert.expectEq('foo(2, ["1"])', "1", foo(2, ["1"]));

