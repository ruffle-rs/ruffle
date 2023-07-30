/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "dowhile-004";
//     var VERSION = "ECMA_2";
//     var TITLE   = "do...while with a labeled continue statement";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    DoWhile( 0, 1 );
    DoWhile( 1, 1 );
    DoWhile( -1, 1 );
    DoWhile( 5, 5 );


    function DoWhile( limit, expect ) {
        i = 0;
        result1 = "pass";
        result2 = "failed: broke out of labeled statement unexpectedly";
    
       foo: {
            do {
                i++;
                if ( ! (i < limit) ) {
                    break;
                    result1 = "fail: evaluated statement after a labeled break";
                }
            } while ( true );
    
            result2 = "pass";
        }
    
        array[item++] = Assert.expectEq(
            
            "do while ( " + i +" < " + limit +" )",
            expect,
            i );
    
        array[item++] = Assert.expectEq(
            
            "breaking out of a do... while loop",
            "pass",
            result1 );
    
    
        array[item++] = Assert.expectEq(
            
            "breaking out of a labeled do...while loop",
            "pass",
            result2 );
    }

    return array;
}
