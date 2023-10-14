/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
var GLOBAL = "[object global]";
 
   // TODO: REVIEW AS4 CONVERSION ISSUE  
//     var SECTION = "10.2.1";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Global Code";


    var testcases = getTestCases();


function getTestCases() {
    var array = new Array();
    var item = 0;

    var THIS = this;

    array[item++] = Assert.expectEq( 
                                    "this +''",
                                    GLOBAL,
                                    THIS + "" );

    var GLOBAL_PROPERTIES = new Array();
    var i = 0;

    for ( p in this ) {
        GLOBAL_PROPERTIES[i++] = p;
    }

    for ( i = 0; i < GLOBAL_PROPERTIES.length; i++ ) {
        array[item++] = Assert.expectEq( 
                                        GLOBAL_PROPERTIES[i] +" == void 0",
                                        false,
                                        (GLOBAL_PROPERTIES[i] == void 0));
    }

    return ( array );
}
