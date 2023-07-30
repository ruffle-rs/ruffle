/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-012";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement";
    var BUGNUMBER="336872";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    // Tests start here.

    TrySomething( "x = \"hi\"", true );
    TrySomething2( "throw \"boo\"", true );
    TrySomething3( "throw 3", true );



    // x = "hi"
    function TrySomething( expression, throwing ) {
        innerFinally = "FAIL: DID NOT HIT INNER FINALLY BLOCK";
        if (throwing) {
            outerCatch = "FAILED: NO EXCEPTION CAUGHT";
        } else {
            outerCatch = "PASS";
        }
        outerFinally = "FAIL: DID NOT HIT OUTER FINALLY BLOCK";


        // If the inner finally does not throw an exception, the result
        // of the try block should be returned.  (Type of inner return
        // value should be throw if finally executes correctly

        try {
            try {
                throw 0;
            } finally {
                innerFinally = "PASS";
        x = hi;
            }
        } catch ( e  ) {
            if (throwing) {
                outerCatch = "PASS";
            } else {
                outerCatch = "FAIL: HIT OUTER CATCH BLOCK";
            }
        } finally {
            outerFinally = "PASS";
        }


        array[item++] = Assert.expectEq(
                
                expression+" evaluated inner finally block",
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer catch block ",
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer finally block",
                "PASS",
                outerFinally );
    }

    function TrySomething2( expression, throwing ) {
        innerFinally = "FAIL: DID NOT HIT INNER FINALLY BLOCK";
        if (throwing) {
            outerCatch = "FAILED: NO EXCEPTION CAUGHT";
        } else {
            outerCatch = "PASS";
        }
        outerFinally = "FAIL: DID NOT HIT OUTER FINALLY BLOCK";


        // If the inner finally does not throw an exception, the result
        // of the try block should be returned.  (Type of inner return
        // value should be throw if finally executes correctly

        try {
            try {
                throw 0;
            } finally {
                innerFinally = "PASS";
        throw "boo";
            }
        } catch ( e  ) {
            if (throwing) {
                outerCatch = "PASS";
            } else {
                outerCatch = "FAIL: HIT OUTER CATCH BLOCK";
            }
        } finally {
            outerFinally = "PASS";
        }


        array[item++] = Assert.expectEq(
                
                expression +" evaluated inner finally block",
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer catch block ",
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer finally block",
                "PASS",
                outerFinally );
    }

    function TrySomething3( expression, throwing ) {
        innerFinally = "FAIL: DID NOT HIT INNER FINALLY BLOCK";
        if (throwing) {
            outerCatch = "FAILED: NO EXCEPTION CAUGHT";
        } else {
            outerCatch = "PASS";
        }
        outerFinally = "FAIL: DID NOT HIT OUTER FINALLY BLOCK";


        // If the inner finally does not throw an exception, the result
        // of the try block should be returned.  (Type of inner return
        // value should be throw if finally executes correctly

        try {
            try {
                throw 0;
            } finally {
                innerFinally = "PASS";
        throw 3;
            }
        } catch ( e  ) {
            if (throwing) {
                outerCatch = "PASS";
            } else {
                outerCatch = "FAIL: HIT OUTER CATCH BLOCK";
            }
        } finally {
            outerFinally = "PASS";
        }


        array[item++] = Assert.expectEq(
                
                expression +" evaluated inner finally block",
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer catch block ",
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression +" evaluated outer finally block",
                "PASS",
                outerFinally );
    }

    return array;
}
