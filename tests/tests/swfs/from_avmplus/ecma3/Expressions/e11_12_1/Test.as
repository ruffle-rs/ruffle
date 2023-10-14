/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_12";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(     "true ? 'PASSED' : 'FAILED'",     "PASSED",       (true?"PASSED":"FAILED"));
    array[item++] = Assert.expectEq(     "false ? 'FAILED' : 'PASSED'",     "PASSED",      (false?"FAILED":"PASSED"));

    array[item++] = Assert.expectEq(     "1 ? 'PASSED' : 'FAILED'",     "PASSED",          (1?"PASSED":"FAILED"));
    array[item++] = Assert.expectEq(     "0 ? 'FAILED' : 'PASSED'",     "PASSED",          (0?"FAILED":"PASSED"));
    array[item++] = Assert.expectEq(     "-1 ? 'PASSED' : 'FAILED'",     "PASSED",          (-1?"PASSED":"FAILED"));

    array[item++] = Assert.expectEq(     "NaN ? 'FAILED' : 'PASSED'",     "PASSED",          (Number.NaN?"FAILED":"PASSED"));

    array[item++] = Assert.expectEq(     "var VAR = true ? , : 'FAILED'", "PASSED",           (VAR = true ? "PASSED" : "FAILED") );

    return ( array );
}
