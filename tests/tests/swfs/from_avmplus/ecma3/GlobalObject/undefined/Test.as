/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.1.1.3";
//     var VERSION = "ECMA_1";
//     var TITLE   = "undefined";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(  "undefined",               undefined,      undefined );
    array[item++] = Assert.expectEq(  "this.undefined",          undefined,      this.undefined );
    array[item++] = Assert.expectEq(  "typeof undefined",        "undefined",      typeof undefined );

    return ( array );
}
