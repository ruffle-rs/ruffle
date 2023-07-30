/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "15.7.4.3-1";
//     var VERSION = "ECMA_1";
    var testcases = getTestCases();



function getTestCases() {
    var array = new Array();
    var item = 0;

//  the following two line causes navigator to crash -- cmb 9/16/97
    //changing expectEq from 4args to 3args
    array[item++] = Assert.expectEq("15.7.4.1 Number.prototype.valueOf()",        0,        Number.prototype.valueOf() );

    array[item++] = Assert.expectEq("15.7.4.1 (new Number(1)).valueOf()",         1,       (new Number(1)).valueOf() );
    array[item++] = Assert.expectEq("15.7.4.1 (new Number(-1)).valueOf()",        -1,      (new Number(-1)).valueOf() );
    array[item++] = Assert.expectEq("15.7.4.1 (new Number(0)).valueOf()",         0,       (new Number(0)).valueOf() );
    array[item++] = Assert.expectEq("15.7.4.1 (new Number(Number.POSITIVE_INFINITY)).valueOf()", Number.POSITIVE_INFINITY, (new Number(Number.POSITIVE_INFINITY)).valueOf() );
    array[item++] = Assert.expectEq("15.7.4.1 (new Number(Number.NaN)).valueOf()",  Number.NaN, (new Number(Number.NaN)).valueOf() );
    array[item++] = Assert.expectEq("15.7.4.1 (new Number()).valueOf()",         0,       (new Number()).valueOf() );

    return ( array );
}
