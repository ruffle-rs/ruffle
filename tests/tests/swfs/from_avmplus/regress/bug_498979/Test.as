/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {import flash.display.MovieClip; public class Test extends MovieClip {}}

import com.adobe.test.Assert;

// var SECTION = "Regression Tests";
// var VERSION = "";
// var TITLE = " Bug 498979 -  jit mangles value when calling a setter with * type";


var _any:*;

function set any(val:*) {
        _any = val;
}

function get any():* {
        return _any;
}

any = 'hello world';

Assert.expectEq("Verify value when calling setter with * type", 'hello world', any);

/*
 The rest of this test file does not work due to this asc bug:
 https://bugs.adobe.com/jira/browse/ASC-3889

function set obj(val:Object) {
        _any = val;
}

function get obj():Object {
        return _any;
}

var testValues = ['hello there', 2.0000000005547, 7.314e24, new Object(), -982743278642,
                  <myXml><testing>hello</testing></myXml>, [4,7,2], false, true, new Date()];

any = testValues[-1];
Assert.expectEq("Verify value when calling setter with * type", testValues[-1], any);


for (var i=0; i<testValues.length; i++) {
    any = testValues[i];
    //Assert.expectEq("Verify value when calling setter with * type", testValues[i], any);
    //obj = testValues[i];
    //Assert.expectEq("Verify value when calling setter with Object type", testValues[i], obj);
}
 */

