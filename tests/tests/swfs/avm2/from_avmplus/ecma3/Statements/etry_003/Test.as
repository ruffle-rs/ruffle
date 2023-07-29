/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-003";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement";
    var BUGNUMBER="http://scopus.mcom.com/bugsplat/show_bug.cgi?id=313585";



    var testcases = getTestCases();

    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    // Tests start here.

    TrySomething( "x = \"hi\"", false );
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

        try {
            try {
                x = "hi";
            } finally {
                innerFinally = "PASS";
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
                
                expression,
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerFinally );


    }

    // throw "boo"
    function TrySomething2( expression, throwing ) {
        innerFinally = "FAIL: DID NOT HIT INNER FINALLY BLOCK";
        if (throwing) {
            outerCatch = "FAILED: NO EXCEPTION CAUGHT";
        } else {
            outerCatch = "PASS";
        }
        outerFinally = "FAIL: DID NOT HIT OUTER FINALLY BLOCK";

        try {
            try {
                throw "boo";
            } finally {
                innerFinally = "PASS";
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
                
                expression,
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerFinally );


    }

    // throw 3
    function TrySomething3( expression, throwing ) {
        innerFinally = "FAIL: DID NOT HIT INNER FINALLY BLOCK";
        if (throwing) {
            outerCatch = "FAILED: NO EXCEPTION CAUGHT";
        } else {
            outerCatch = "PASS";
        }
        outerFinally = "FAIL: DID NOT HIT OUTER FINALLY BLOCK";

        try {
            try {
                throw 3;
            } finally {
                innerFinally = "PASS";
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
                
                expression,
                "PASS",
                innerFinally );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerCatch );
        array[item++] = Assert.expectEq(
                
                expression,
                "PASS",
                outerFinally );


    }
    return array;
}
