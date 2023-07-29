/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-008";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement: try in a constructor";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    function Integer( value, exception ) {
        try {
            this.value = checkValue( value );
        } catch ( e ) {
            this.value = e.toString();
        }

        array[item++] = Assert.expectEq(
            
            "Integer( " + value +" )",
            (exception ? INVALID_INTEGER_VALUE +": " + value : this.value),
            this.value );
    }

    var INVALID_INTEGER_VALUE = "Invalid value for java.lang.Integer constructor";

    function checkValue( value ) {
        if ( Math.floor(value) != value || isNaN(value) ) {
            throw ( INVALID_INTEGER_VALUE +": " + value );
        } else {
            return value;
        }
    }

    // add test cases

    new Integer( 3, false );
    new Integer( NaN, true );
    new Integer( 0, false );
    new Integer( Infinity, false );
    new Integer( -2.12, true );
    new Integer( Math.LN2, true );


    return array;
}
