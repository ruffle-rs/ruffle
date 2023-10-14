/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "StrictEquality_001 - 11.9.6";
//     var VERSION = "ECMA_2";
//     var TITLE   =  "The strict equality operator ( === )";

    
    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;


    // 1. If Type(x) is different from Type(y) return false

    // see bug 103471 for details
    StrictEquality( true, new Boolean(true), true, array, item++);
    StrictEquality( new Boolean(), false, true, array, item++);
    StrictEquality( "", new String(),    true, array, item++);
    StrictEquality( new String("hi"), "hi", true, array, item++);

    // 2. If Type(x) is not Number go to step 9.

    // 3. If x is NaN, return false
    StrictEquality( NaN, NaN,   false, array, item++ );
    StrictEquality( NaN, 0,     false, array, item++ );

    // 4. If y is NaN, return false.
    StrictEquality( 0,  NaN,    false, array, item++ );

    // 5. if x is the same number value as y, return true

    // 6. If x is +0 and y is -0, return true

    // 7. If x is -0 and y is +0, return true

    // 8. Return false.


    // 9.  If Type(x) is String, then return true if x and y are exactly
    //  the same sequence of characters ( same length and same characters
    //  in corresponding positions.) Otherwise return false.

    //  10. If Type(x) is Boolean, return true if x and y are both true or
    //  both false. otherwise return false.


    //  Return true if x and y refer to the same object.  Otherwise return
    //  false.

    // Return false.
    return array;
}

function StrictEquality( x, y, expect, array, item ) {
    result = ( x === y );

    array[item] = Assert.expectEq(

        x +" === " + y,
        expect,
        result );
}

