/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-007";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement:  for-in";



    var testcases = getTestCases();

    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    function throwException() {
        throw EXCEPTION_STRING +": " + this.valueOf();
    }
    var EXCEPTION_STRING = "Exception thrown:";

    function noException() {
        return this.valueOf();
    }

    TryForIn( new TryObject( "hello", throwException, true ));
    TryForIn( new TryObject( "hola",  noException, false ));



function TryObject( value, fun, exception ) {
    this.value = value;
    this.exception = exception;

    this.check = fun;
    this.valueOf = function () { return this.value; }
}


var result;

 function TryForIn( object ) {
    try {
        for ( p in object ) {
            if ( typeof object[p] == "function" ) {
                result = object[p]();
            }
        }
    } catch ( e ) {
        result = e;
    }

    array[item++] = Assert.expectEq(
        
        "TryForIn( " + object+ " )",
        (object.exception ? EXCEPTION_STRING +": " + object.value : object.value),
        result );

 }
    return array;
}
