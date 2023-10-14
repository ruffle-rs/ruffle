/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = "10.2.3-1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Eval Code";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var o = new MyObject("hello")

    array[item++] = Assert.expectEq( 
                                    "var o = new MyObject('hello'); o.THIS == x",
                                    true,
                                    o.THIS == o );

    var o = MyFunction();

    array[item++] = Assert.expectEq( 
                                    "var o = MyFunction(); o == this",
                                    true,
                                    o == this );

    function MyFunction( value ) {
        return this;
    }
    function MyObject( value ) {
     this.THIS = this;
    }

    return ( array );
}
