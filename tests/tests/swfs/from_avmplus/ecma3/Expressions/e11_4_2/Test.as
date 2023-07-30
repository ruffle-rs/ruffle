/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "e11_4_2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The void operator";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    array[item++] = Assert.expectEq(    "void(new String('string object'))",      void 0,  void(new String( 'string object' )) );
    array[item++] = Assert.expectEq(    "void('string primitive')",               void 0,  void("string primitive") );
    array[item++] = Assert.expectEq(    "void(Number.NaN)",                       void 0,  void(Number.NaN) );
    array[item++] = Assert.expectEq(    "void(Number.POSITIVE_INFINITY)",         void 0,  void(Number.POSITIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "void(1)",                                void 0,  void(1) );
    array[item++] = Assert.expectEq(    "void(0)",                                void 0,  void(0) );
    array[item++] = Assert.expectEq(    "void(-1)",                               void 0,  void(-1) );
    array[item++] = Assert.expectEq(    "void(Number.NEGATIVE_INFINITY)",         void 0,  void(Number.NEGATIVE_INFINITY) );
    array[item++] = Assert.expectEq(    "void(Math.PI)",                          void 0,  void(Math.PI) );
    array[item++] = Assert.expectEq(    "void(true)",                             void 0,  void(true) );
    array[item++] = Assert.expectEq(    "void(false)",                            void 0,  void(false) );
    array[item++] = Assert.expectEq(    "void(null)",                             void 0,  void(null) );
    array[item++] = Assert.expectEq(    "void new String('string object')",      void 0,  void new String( 'string object' ) );
    array[item++] = Assert.expectEq(    "void 'string primitive'",               void 0,  void "string primitive" );
    array[item++] = Assert.expectEq(    "void Number.NaN",                       void 0,  void Number.NaN );
    array[item++] = Assert.expectEq(    "void Number.POSITIVE_INFINITY",         void 0,  void Number.POSITIVE_INFINITY );
    array[item++] = Assert.expectEq(    "void 1",                                void 0,  void 1 );
    array[item++] = Assert.expectEq(    "void 0",                                void 0,  void 0 );
    array[item++] = Assert.expectEq(    "void -1",                               void 0,  void -1 );
    array[item++] = Assert.expectEq(    "void Number.NEGATIVE_INFINITY",         void 0,  void Number.NEGATIVE_INFINITY );
    array[item++] = Assert.expectEq(    "void Math.PI",                          void 0,  void Math.PI );
    array[item++] = Assert.expectEq(    "void true",                             void 0,  void true );
    array[item++] = Assert.expectEq(    "void false",                            void 0,  void false );
    array[item++] = Assert.expectEq(    "void null",                             void 0,  void null );

//     array[item++] = Assert.expectEq(    "void()",                                 void 0,  void() );

    return ( array );
}

function test() {
    for ( i = 0; i < testcases.length; i++ ) {
            testcases[i].passed = writeTestCaseResult(
                    testcases[i].expect,
                    testcases[i].actual,
                    testcases[i].description +" = "+ testcases[i].actual );
            testcases[i].reason += ( testcases[i].passed ) ? "" : "wrong value "
    }
    stopTest();
    return ( testcases );
}
