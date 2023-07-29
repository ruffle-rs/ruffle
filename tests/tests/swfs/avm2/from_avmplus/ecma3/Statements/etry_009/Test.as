/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-009";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement: try in a while block";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    var EXCEPTION_STRING = "Exception thrown: ";
    var NO_EXCEPTION_STRING = "No exception thrown: ";


    TryInWhile( new TryObject( "hello", ThrowException, true ) );
    TryInWhile( new TryObject( "aloha", NoException, false ));


    function TryObject( value, throwFunction, result ) {
        this.value = value;
        this.thrower = throwFunction;
        this.result = result;
    }
    function ThrowException() {
        throw EXCEPTION_STRING + this.value;
    }
    function NoException() {
        return NO_EXCEPTION_STRING + this.value;
    }
    function TryInWhile( object ) {
        result = null;
        while ( true ) {
            try {
                object.thrower();
                result = NO_EXCEPTION_STRING + object.value;
                break;
            } catch ( e ) {
                result = e;
                break;
            }
        }

        array[item++] = Assert.expectEq(
            
            "( "+ object  +".thrower() )",
            (object.result
            ? EXCEPTION_STRING + object.value :
            NO_EXCEPTION_STRING + object.value),
            result );
    }
    return array;
}
