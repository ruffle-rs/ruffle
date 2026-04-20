/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "regress_636535";
//     var VERSION = "AS3";
//     var TITLE   = "Array.splice should not crash";
//     var bug     = "636535";

    var testcases = getTestCases();

function getTestCases()
{
    var item = 0;
    var array = [];

    var a = [4]
    array[item++] = Assert.expectEq( "a[0] == 4", 4, a[0]);
    array[item++] = Assert.expectEq( "a.length == 1", 1, a.length);
    delete a[0];
    array[item++] = Assert.expectEq( "a[0] == undefined", undefined, a[0]);
    array[item++] = Assert.expectEq( "a.length == 1", 1, a.length);
    a.splice(0, 1);
    array[item++] = Assert.expectEq( "a[0] == undefined", undefined, a[0]);
    array[item++] = Assert.expectEq( "a.length == 0", 0, a.length);

    return ( array );
}
