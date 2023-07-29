/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.7.3";
//     var VERSION = "ECMA_2";
//     var TITLE   = "Properties of the Number Constructor";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    array[item++] = Assert.expectEq( "Number.constructor.prototype",   Class.prototype, Number.constructor.prototype);
    array[item++] = Assert.expectEq( "Number.length",      1,                  Number.length );

    return ( array );
}
