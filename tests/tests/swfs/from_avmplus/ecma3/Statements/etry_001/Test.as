/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement";


    var testcases = getTestCases();
    
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    var INVALID_JAVA_INTEGER_VALUE = "Invalid value for java.lang.Integer constructor";

    TryNewJavaInteger( "3.14159", INVALID_JAVA_INTEGER_VALUE );
    TryNewJavaInteger( NaN, INVALID_JAVA_INTEGER_VALUE );
    TryNewJavaInteger( 0,  0 );
    TryNewJavaInteger( -1, -1 );
    TryNewJavaInteger( 1,  1 );
    TryNewJavaInteger( Infinity, Infinity );




    function newJavaInteger( v ) {
        value = Number( v );
        if ( Math.floor(value) != value || isNaN(value) ) {
            throw ( INVALID_JAVA_INTEGER_VALUE );
        } else {
            return value;
        }
    }

    function TryNewJavaInteger( value, expect ) {
        var finalTest = false;

        try {
            result = newJavaInteger( value );
        } catch ( e ) {
            result = String( e );
        } finally {
            finalTest = true;
        }
            array[item++] = Assert.expectEq(
                
                "newJavaValue( " + value +" )",
                expect,
                result);

            array[item++] = Assert.expectEq(
                
                "newJavaValue( " + value +" ) hit finally block",
                true,
                finalTest);

    }
    return array;
}
