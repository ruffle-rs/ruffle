/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "9.2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "ToBoolean";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    // special cases here

    array[item++] = Assert.expectEq(    "Boolean()",                     false,  Boolean() );
    var x;
    array[item++] = Assert.expectEq(    "Boolean(var x)",                false,  Boolean(x) );
    array[item++] = Assert.expectEq(    "Boolean(void 0)",               false,  Boolean(void 0) );
    array[item++] = Assert.expectEq(    "Boolean(null)",                 false,  Boolean(null) );
    array[item++] = Assert.expectEq(    "Boolean(false)",                false,  Boolean(false) );
    array[item++] = Assert.expectEq(    "Boolean(true)",                 true,   Boolean(true) );
    array[item++] = Assert.expectEq(    "Boolean(0)",                    false,  Boolean(0) );
    array[item++] = Assert.expectEq(    "Boolean(-0)",                   false,  Boolean(-0) );
    array[item++] = Assert.expectEq(    "Boolean(NaN)",                  false,  Boolean(Number.NaN) );
    array[item++] = Assert.expectEq(    "Boolean('')",                   false,  Boolean("") );

    // normal test cases here

    array[item++] = Assert.expectEq(    "Boolean(Infinity)",             true,   Boolean(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "Boolean(-Infinity)",            true,   Boolean(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "Boolean(Math.PI)",              true,   Boolean(Math.PI) );
    array[item++] = Assert.expectEq(    "Boolean(1)",                    true,   Boolean(1) );
    array[item++] = Assert.expectEq(    "Boolean(-1)",                   true,   Boolean(-1) );
    array[item++] = Assert.expectEq(    "Boolean([tab])",                true,   Boolean("\t") );
    array[item++] = Assert.expectEq(    "Boolean('0')",                  true,   Boolean("0") );
    array[item++] = Assert.expectEq(    "Boolean('string')",             true,   Boolean("string") );

    // ToBoolean (object) should always return true.
    array[item++] = Assert.expectEq(    "Boolean(new String() )",        false,   Boolean(new String()) );
    array[item++] = Assert.expectEq(    "Boolean(new String('') )",      false,   Boolean(new String("")) );

    array[item++] = Assert.expectEq(    "Boolean(new Boolean(true))",    true,   Boolean(new Boolean(true)) );
    array[item++] = Assert.expectEq(    "Boolean(new Boolean(false))",   false,   Boolean(new Boolean(false)) );
    array[item++] = Assert.expectEq(    "Boolean(new Boolean() )",       false,   Boolean(new Boolean()) );

    array[item++] = Assert.expectEq(    "Boolean(new Array())",          true,   Boolean(new Array()) );

    array[item++] = Assert.expectEq(    "Boolean(new Number())",         false,   Boolean(new Number()) );
    array[item++] = Assert.expectEq(    "Boolean(new Number(-0))",       false,   Boolean(new Number(-0)) );
    array[item++] = Assert.expectEq(    "Boolean(new Number(0))",        false,   Boolean(new Number(0)) );
    array[item++] = Assert.expectEq(    "Boolean(new Number(NaN))",      false,   Boolean(new Number(Number.NaN)) );

    array[item++] = Assert.expectEq(    "Boolean(new Number(-1))",       true,   Boolean(new Number(-1)) );
    array[item++] = Assert.expectEq(    "Boolean(new Number(Infinity))", true,   Boolean(new Number(Number.POSITIVE_INFINITY)) );
    array[item++] = Assert.expectEq(    "Boolean(new Number(-Infinity))",true,   Boolean(new Number(Number.NEGATIVE_INFINITY)) );

    array[item++] = Assert.expectEq(     "Boolean(new Object())",       true,       Boolean(new Object()) );
    array[item++] = Assert.expectEq(     "Boolean(new Function())",     true,       Boolean(new Function()) );
    array[item++] = Assert.expectEq(     "Boolean(new Date())",         true,       Boolean(new Date()) );
    array[item++] = Assert.expectEq(     "Boolean(new Date(0))",         true,       Boolean(new Date(0)) );
    array[item++] = Assert.expectEq(     "Boolean(Math)",         true,       Boolean(Math) );
    
    return array;
}
