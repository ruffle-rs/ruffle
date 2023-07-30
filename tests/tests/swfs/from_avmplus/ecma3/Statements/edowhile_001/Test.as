/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "dowhile-002";
//     var VERSION = "ECMA_2";
//     var TITLE   = "do...while with a labeled continue statement";


    var testcases = getTestCases();

    
function getTestCases() {
    var array = new Array();
    var item = 0;
    
    LabeledContinue( 0, 1 );
    LabeledContinue( 1, 1 );
    LabeledContinue( -1, 1 );
    LabeledContinue( 5, 5 );
    
    function LabeledContinue( limit, expect ) {
        i = 0;
        woohoo:
            do {
                i++;
                continue woohoo;
            } while ( i < limit );
    
        array[item++] = Assert.expectEq(
            
            "do while ( " + i +" < " + limit +" )",
            expect,
            i );
    }
    
    return array;
}

