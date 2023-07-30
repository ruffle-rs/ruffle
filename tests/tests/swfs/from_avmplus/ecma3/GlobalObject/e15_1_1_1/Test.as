/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.1.1.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "NaN";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[array.length] = Assert.expectEq(  "NaN",               Number.NaN,     NaN );
    array[array.length] = Assert.expectEq(  "this.NaN",          undefined,      this.NaN );
    array[array.length] = Assert.expectEq(  "typeof NaN",        "number",       typeof NaN );

    return ( array );
}
