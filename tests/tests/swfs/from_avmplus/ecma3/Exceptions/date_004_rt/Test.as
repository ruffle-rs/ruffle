/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;
//     var SECTION = "date-004";
//     var VERSION = "JS1_4";
//     var TITLE   = "Date.prototype.getTime";


    var testcases = getTestCases();
    
function getTestCases() {
    var array = new Array();
    var item = 0;

    if (true) {       // TODO: REVIEW AS4 CONVERSION ISSUE   
        var MYDATE = new MyDate();
        array[item++] = Assert.expectEq(
          //    SECTION,
            "MYDATE = new MyDate(); MYDATE.getTime()",
            NaN,
            MYDATE.getTime());
    } else {
        var result = "Failed";
        var exception = "No exception thrown";
        var expect = "Passed";
    
        try {
            var MYDATE = new MyDate();
            result = MYDATE.getTime();
        } catch ( e ) {
            result = expect;
            exception = e.toString();
        }finally{
    
        array[item++] = Assert.expectEq(
          //    SECTION,
            "MYDATE = new MyDate(); MYDATE.getTime()",
            Utils.TYPEERROR+1034,
            Utils.typeError( exception ) );
        }
    }
    
    return array;
}

function MyDate( value ) {
    this.value = value;
    this.getTime = Date.prototype.getTime;
}


