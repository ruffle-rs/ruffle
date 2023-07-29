/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "15.4.5.2-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Array.length";


    var testcases = new Array();

    addCase( new Array(), 0, Math.pow(2,14), Math.pow(2,14) );

    addCase( new Array(), 0, 1, 1 );

    addCase( new Array(Math.pow(2,12)), Math.pow(2,12), 0, 0 );
    addCase( new Array(Math.pow(2,13)), Math.pow(2,13), Math.pow(2,12), Math.pow(2,12) );
    addCase( new Array(Math.pow(2,12)), Math.pow(2,12), Math.pow(2,12), Math.pow(2,12) );
    addCase( new Array(Math.pow(2,14)), Math.pow(2,14), Math.pow(2,12), Math.pow(2,12) )

    // some tests where array is not empty
    // array is populated with strings
    var length = Math.pow(2,12);
    var a = new Array(length);
    for (var i = 0; i < Math.pow(2,12); i++ ) {
        a[i] = i;
    }
    addCase( a, i, i, i );
    addCase( a, i, Math.pow(2,12)+i+1, Math.pow(2,12)+i+1, true );
    addCase( a, Math.pow(2,12)+5, 0, 0, true );


function addCase( object, old_len, set_len, new_len, ... rest) {
    var checkitems;
    if( rest.length == 1 ){
        checkitems = rest[0];
    }

    object.length = set_len;

    testcases[testcases.length] = Assert.expectEq( 
        "array = new Array("+ old_len+"); array.length = " + set_len +
        "; array.length",
        new_len,
        object.length );

    if ( checkitems ) {
    // verify that items between old and newlen are all undefined
    if ( new_len < old_len ) {
        var passed = true;
        for ( var i = new_len; i < old_len; i++ ) {
            if ( object[i] != void 0 ) {
                passed = false;
            }
        }
        testcases[testcases.length] = Assert.expectEq( 
            "verify that array items have been deleted",
            true,
            passed );
    }
    if ( new_len > old_len ) {
        var passed = true;
        for ( var i = old_len; i < new_len; i++ ) {
            if ( object[i] != void 0 ) {
                passed = false;
            }
        }
        testcases[testcases.length] = Assert.expectEq( 
            "verify that new items are undefined",
            true,
            passed );
    }
    }

}
