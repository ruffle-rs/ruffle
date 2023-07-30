/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "12.6.1-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "The While statement";

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var MYVAR = 0; while( MYVAR++ < 100) { if ( MYVAR < 100 ) break; }
    array[item++] = Assert.expectEq( 
                                    "var MYVAR = 0; while( MYVAR++ < 100) { if ( MYVAR < 100 ) break; } MYVAR ",
                                    1,
                                    MYVAR);

    var MYVAR = 0; while( MYVAR++ < 100) { if ( MYVAR < 100 ) continue; else break; }
    array[item++] = Assert.expectEq( 
                                    "var MYVAR = 0; while( MYVAR++ < 100) { if ( MYVAR < 100 ) continue; else break; } MYVAR ",
                                    100,
                                    MYVAR);


    function MYFUN( arg1 )
    {
        while ( arg1++ < 100 )
        {
            if ( arg1 < 100 ) return arg1;
        }
    }
    array[item++] = Assert.expectEq( 
                                    "function MYFUN( arg1 ) { while ( arg1++ < 100 ) { if ( arg1 < 100 ) return arg1; } }; MYFUN(1)",
                                    2,
                                   (MYFUN(1)));
    return array;
}
