/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "15.6.3.1-4";
//     var VERSION = "ECMA_1";
//     var TITLE   = "Boolean.prototype"

    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;

    var BOOL_PROTO = Boolean.prototype;

    array[item++] = Assert.expectEq( 
                                 "var BOOL_PROTO = Boolean.prototype; Boolean.prototype; Boolean.prototype == BOOL_PROTO",
                                 true,
                                 (BOOL_PROTO = Boolean.prototype, Boolean.prototype, Boolean.prototype == BOOL_PROTO ) );

    var thisError="no error";

    try{
        // save
        var origBoolProto = Boolean.prototype;

        Boolean.prototype=null;

        // restore
        Boolean.prototype = origBoolProto;
    }catch(e:ReferenceError){
        thisError=e.toString();
    }finally{
        array[item++]=Assert.expectEq("Trying to verify that Boolean.prototype is read-only","ReferenceError: Error #1074",Utils.referenceError(thisError));
    }

    return ( array );
}
