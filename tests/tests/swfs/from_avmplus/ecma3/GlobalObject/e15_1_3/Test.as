/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


//     var SECTION = "15-1.3";
//     var VERSION = "ECMA_3";
//     var TITLE   = "Unicode Surrogate pairs";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    try {
        decodeURI('\uD800');
    } catch (e ){
        trace( "Exception thrown" );
    }

    array[item++] = Assert.expectEq(  "escape( false )", "false",  escape( false ) );
    array[item++] = Assert.expectEq(  "encodeURI('\u007f')", "%7F",  encodeURI('\u007f') );

    return ( array );
}
