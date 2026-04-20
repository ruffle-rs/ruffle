/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15.8.2.12";
//     var VERSION = "";
//     var TITLE   = "Math.min(x, y, ... rest) and additional tests base on code coverage";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    /*
      Testcases based on code coverage analysis:
      core/MathClass.cpp
      Lars: "You can tell -0 from 0 by dividing 1 by the zero, -0 gives -Infinity
      and 0 gives Infinity, so if you know x is a zero the test for negative
      zero is (1/x < 0)."
    */
    array[item++] = Assert.expectEq(   "Math.min(Number.NaN, 2, 1)", Number.NaN, Math.min(Number.NaN, 2, 1) );
    array[item++] = Assert.expectEq(   "Math.min(1, Number.NaN, 1)", Number.NaN, Math.min(1, Number.NaN, 1) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(-0.0, 0.0)",    -Infinity,  1.0/Math.min(-0.0, 0.0) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(-0.0, -0.0)",   -Infinity,  1.0/Math.min(-0.0, -0.0) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(0.0, 0.0)",     Infinity,   1.0/Math.min(0.0, 0.0) );
    array[item++] = Assert.expectEq(   "Math.min(2, 2, 1)",          1,          Math.min(2, 2, 1) );
    array[item++] = Assert.expectEq(   "Math.min(1, 2, 2)",          1,          Math.min(1, 2, 2) );
    array[item++] = Assert.expectEq(   "Math.min(1, 2, 1)",          1,          Math.min(1, 2, 1) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(0.0, 2, 0.0)",  Infinity,   1.0/Math.min(0.0, 2, 0.0) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(-0.0, 2, -0.0)",-Infinity,  1.0/Math.min(-0.0, 2, -0.0) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(-0.0, 2, 0.0)", -Infinity,  1.0/Math.min(-0.0, 2, 0.0) );
    array[item++] = Assert.expectEq(   "1.0/Math.min(3, 2, 1, 0, 0.0, -0.0)", -Infinity, 1.0/Math.min(3, 2, 1, 0, 0.0, -0.0) );

    return ( array );
}
