/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;


//     var SECTION = "15.5.4.3-3-n";
//     var VERSION = "ECMA_1";
//     var TITLE   = "String.prototype.valueOf";


    var testcases = getTestCases();

function getTestCases() {
    var array = new Array();
    var item = 0;
    var valof=String.prototype.valueOf;
    astring=new Number();
    
    var expectedError = 1056;
    if (true) {
        expectedError = 1037;
    }
    try{
        astring.valueOf = valof;
    }catch(e:Error){
        thisError=e.toString();
    }finally{
        array[item++] = Assert.expectEq( 
                "var valof=String.prototype.valueOf; astring=new Number(); astring.valueOf = valof; astring.valueOf()",
                Utils.REFERENCEERROR+expectedError,
                Utils.referenceError(thisError) )
    }

    

    return ( array );
}
