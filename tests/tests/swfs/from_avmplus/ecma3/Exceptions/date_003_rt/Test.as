/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "date-003";
//     var VERSION = "JS1_4";
//     var TITLE   = "Date.prototype.valueOf";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array;
    var item = 0;

    if (true) {       // TODO: REVIEW AS4 CONVERSION ISSUE   
        var OBJ = new MyObject( new Date(0) );
        array[item++] = Assert.expectEq(
      //    SECTION,
        "OBJ = new MyObject( new Date(0)); OBJ.valueOf()",
        NaN,
        OBJ.valueOf());
    } else {
        var result = "Failed";
        var exception = "No exception thrown";
        var expect = "Passed";
    
        try {
            var OBJ = new MyObject( new Date(0) );
            result = OBJ.valueOf();
        } catch ( e ) {
            result = expect;
            exception = e.toString();
        }finally{
    
        array[item++] = Assert.expectEq(
          //    SECTION,
            "OBJ = new MyObject( new Date(0)); OBJ.valueOf()",
            Utils.TYPEERROR+1034,
            Utils.typeError(exception) );
        }
    }
    return array;
}

function MyObject( value ) {
    this.value = value;
    this.valueOf = Date.prototype.valueOf;
//  The following line causes an infinte loop
//    this.toString = new Function( "return this+\"\";");
    return this;
}


