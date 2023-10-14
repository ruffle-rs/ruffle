/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_4_9";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Logical NOT operator (!)";


//    version("130")

    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    array[item++] = Assert.expectEq(    "!(null)",                true,   !(null) );
    array[item++] = Assert.expectEq(    "!(void 0)",              true,   !(void 0) );

    array[item++] = Assert.expectEq(    "!(false)",               true,   !(false) );
    array[item++] = Assert.expectEq(    "!(true)",                false,  !(true) );
    array[item++] = Assert.expectEq(    "!(0)",                   true,   !(0) );
    array[item++] = Assert.expectEq(    "!(-0)",                  true,   !(-0) );
    array[item++] = Assert.expectEq(    "!(NaN)",                 true,   !(Number.NaN) );
    array[item++] = Assert.expectEq(    "!(Infinity)",            false,  !(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "!(-Infinity)",           false,  !(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "!(Math.PI)",             false,  !(Math.PI) );
    array[item++] = Assert.expectEq(    "!(1)",                   false,  !(1) );
    array[item++] = Assert.expectEq(    "!(-1)",                  false,  !(-1) );
    array[item++] = Assert.expectEq(    "!('')",                  true,   !("") );
    array[item++] = Assert.expectEq(    "!('\t')",                false,  !("\t") );
    array[item++] = Assert.expectEq(    "!('0')",                 false,  !("0") );
    array[item++] = Assert.expectEq(    "!('string')",            false,  !("string") );
    array[item++] = Assert.expectEq(    "!(new String(''))",      true,  !(new String("")) );
    array[item++] = Assert.expectEq(    "!(new String('string'))",    false,  !(new String("string")) );
    array[item++] = Assert.expectEq(    "!(new String())",        true,  !(new String()) );
    array[item++] = Assert.expectEq(    "!(new Boolean(true))",   false,   !(new Boolean(true)) );
    array[item++] = Assert.expectEq(    "!(new Boolean(false))",  true,   !(new Boolean(false)) );
    array[item++] = Assert.expectEq(    "!(new Array())",         false,  !(new Array()) );
    array[item++] = Assert.expectEq(    "!(new Array(1,2,3)",     false,  !(new Array(1,2,3)) );
    array[item++] = Assert.expectEq(    "!(new Number())",        true,  !(new Number()) );
    array[item++] = Assert.expectEq(    "!(new Number(0))",       true,  !(new Number(0)) );
    array[item++] = Assert.expectEq(    "!(new Number(NaN))",     true,  !(new Number(Number.NaN)) );
    array[item++] = Assert.expectEq(    "!(new Number(Infinity))", false, !(new Number(Number.POSITIVE_INFINITY)) );

    return (array);
}

