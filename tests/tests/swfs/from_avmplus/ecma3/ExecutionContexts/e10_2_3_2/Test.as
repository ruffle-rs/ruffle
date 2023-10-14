/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "10.2.3-2";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Function and Anonymous Code";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var o = new MyObject("hello")

    array[item++] = Assert.expectEq( 
                                    "MyFunction(\"PASSED!\")",
                                    "PASSED!",
                                    MyFunction("PASSED!") );

    var o = MyFunction();

    array[item++] = Assert.expectEq( 
                                    "MyOtherFunction(true);",
                                    false,
                                    MyOtherFunction(true) );

    function MyFunction( value ) {
        var x = value;
        delete x;
        return x;
    }
    function MyOtherFunction(value) {
        var x = value;
        return delete x;
    }
    function MyObject( value ) {
     this.THIS = this;
    }

    return ( array );
}
