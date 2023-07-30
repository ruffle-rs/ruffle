/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.6.4.3-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Boolean.prototype.valueOf()";


    var testcases = new Array();

    testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    valof=Boolean.prototype.valueOf;
    Boolean.prototype.valueOf=valof;
    x=new Boolean();

    array[item++] = Assert.expectEq( 
                            "valof=Boolean.prototype.valueOf; Boolean.prototpye.valueOf=valof; x=new Boolean();x.valueOf()",
                            false,
                            x.valueOf());

    x=new Boolean(true);
    array[item++] = Assert.expectEq( 
                            "valof=Boolean.prototype.valueOf; Boolean.prototpye.valueOf=valof; x=new Boolean(true);x.valueOf()",
                            true,
                            x.valueOf());

    x=new Boolean(false);
    array[item++] = Assert.expectEq( 
                            "valof=Boolean.prototype.valueOf; Boolean.prototpye.valueOf=valof; x=new Boolean(false);x.valueOf()",
                            false,
                            x.valueOf());

    return ( array );
}
