/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_14_1";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(     "true, false",                    false,  (true, false) );
    array[item++] = Assert.expectEq(     "VAR1=true, VAR2=false",          false,  (VAR1=true, VAR2=false) );
    array[item++] = Assert.expectEq(     "VAR1=true, VAR2=false;VAR1",     true,   (VAR1=true, VAR2=false, VAR1) );
    return ( array );
}
