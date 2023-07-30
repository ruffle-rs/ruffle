/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
//     var SECTION = "try-005";
//     var VERSION = "ECMA_2";
//     var TITLE   = "The try statement";



    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;
        
    TryToCatch( "Math.PI", Math.PI );
    TryToCatch2( "Thrower(5)",   "Caught 5" );
    TryToCatch3( "Thrower(\"some random exception\")", "Caught some random exception" );


    function Thrower( v ) {
        throw "Caught " + v;
    }
    function Eval( v ) {
        return Math.PI;
    }
    function Eval2( v ) {
        return Thrower(5);
    }
    function Eval3( v ) {
        return Thrower("some random exception");
    }




    // Math.PI
    function TryToCatch( value, expect ) {
        try {
            result = Eval(value);
        } catch ( e ) {
            result = e;
        }

        array[item++] = Assert.expectEq(
            
            value,
            expect,
            result );
    }

    // Thrower(5);
    function TryToCatch2( value, expect ) {
        try {
            result = Eval2( value );
        } catch ( e ) {
            result = e;
        }

        array[item++] = Assert.expectEq(
            
            value,
            expect,
            result );
    }

    function TryToCatch3( value, expect ) {
        try {
            result = Eval3( value );
        } catch ( e ) {
            result = e;
        }

        array[item++] = Assert.expectEq(
            
            value,
            expect,
            result );
    }

    return array;
}
