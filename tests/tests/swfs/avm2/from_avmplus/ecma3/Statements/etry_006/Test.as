/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "try-006";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement";



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

    TryWith( new TryObject( "hello", throwException, true ));
    TryWith( new TryObject( "hola",  noException, false ));



    function TryObject( value, fun, exception ) {
        this.value = value;
        this.exception = exception;

       // this.valueOf = new Function ( "return this.value" );
        this.valueOf = function(){return this.value;}
        this.check = fun;
    }

    function TryWith( object ) {
        try {
            with ( object ) {
                result = check();
            }
        } catch ( e ) {
            result = e;
        }
        array[item++] = Assert.expectEq(
            
            "TryWith( " + object.value +" )",
            (object.exception ? EXCEPTION_STRING +": " + object.valueOf() : object.valueOf()),
            result );
        with (Object)                   //Added this as bug #139065 is fixed
           {
             try{
                 undeclR = undecl;
            }catch (e1){
                 result = e1.toString();
             }
           }
           
        array[item++] = Assert.expectEq(
            
            "TryWith( " + object.value +" )",
            "ReferenceError: Error #1065",
            Utils.referenceError(result) );

        
    }
    
    return array;
}
