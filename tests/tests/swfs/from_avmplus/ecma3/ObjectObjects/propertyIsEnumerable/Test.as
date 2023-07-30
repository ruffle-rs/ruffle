/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = '15.2.4.7';
//     var VERSION = 'no version';
//     var TITLE = 'propertyIsEnumerable';


    var count = 0;
    var testcases = new Array();

    var a = new Array("apple", "banana", "cactus", "orange");


    testcases[count++] = Assert.expectEq(  "a.propertyIsEnumerable(0)", true, a.propertyIsEnumerable(0));
    testcases[count++] = Assert.expectEq(  "a.propertyIsEnumerable(1)", true, a.propertyIsEnumerable(1));
    testcases[count++] = Assert.expectEq(  "a.propertyIsEnumerable(2)", true, a.propertyIsEnumerable(2));
    testcases[count++] = Assert.expectEq(  "a.propertyIsEnumerable(3)", true, a.propertyIsEnumerable(3));
    testcases[count++] = Assert.expectEq(  "a.propertyIsEnumerable(4)", false, a.propertyIsEnumerable(4));


