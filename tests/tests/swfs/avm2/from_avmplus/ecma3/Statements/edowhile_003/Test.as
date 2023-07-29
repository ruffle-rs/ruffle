/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "dowhile-003";
//     var VERSION = "ECMA_2";
//     var TITLE   = "do...while with a labeled continue statement";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    DoWhile( new DoWhileObject( 1, 1, 0 ));
    DoWhile( new DoWhileObject( 1000, 1000, 0 ));
    DoWhile( new DoWhileObject( 1001, 1001, 0 ));
    DoWhile( new DoWhileObject( 1002, 1001, 1 ));
    DoWhile( new DoWhileObject( -1, 1001, -1002 ));



    function DoWhileObject( value, iterations, endvalue ) {
        this.value = value;
        this.iterations = iterations;
        this.endvalue = endvalue;
    }
    
    function DoWhile( object ) {
        var i = 0;
    
        do {
            object.value =  --object.value;
            i++;
            if ( i > 1000 )
                break;
       } while( object.value );
    
       array[item++] = Assert.expectEq(
            
            "loop iterations",
            object.iterations,
            i
        );
    
       array[item++] = Assert.expectEq(
            
            "object.value",
            object.endvalue,
            Number( object.value )
        );
    
    }

    return array;
}
