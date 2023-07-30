/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.6.4.3-3";
//     var VERSION = "ECMA_4";
//     var TITLE   = "Boolean.prototype.valueOf()";


    var testcases = new Array();

    testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    valof=Boolean.prototype.valueOf;
    Boolean.prototype.valueOf=valof;
    x=true;
    array[item++] = Assert.expectEq( 
                                "valof=Boolean.prototype.valueOf; Boolean.prototype.valueOf=valof; x=true; x.valueOf()",
                                true,
                                x.valueOf());
    x=false;
    array[item++] = Assert.expectEq( 
                                "valof=Boolean.prototype.valueOf; Boolean.prototype.valueOf=valof; x=false; x.valueOf()",
                                false,
                                x.valueOf());
    return ( array );
}
