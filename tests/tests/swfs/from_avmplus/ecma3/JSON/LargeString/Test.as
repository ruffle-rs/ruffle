/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


    import flash.utils.*;
import com.adobe.test.Assert;

//     var SECTION = "15.2";
//     var VERSION = "ECMA_5";
//     var TITLE   = "JSON AS3 large String";





    var string100:String='0123456790abcdefghijklmnopqstuABCDEFGHIJKLMNOPQRSTU01234567890123456789abcdefghijklmnopqrstuvwxyzAB';
    var largeString='';
    // generate string of length 10,000
    for (i=0;i<200;i++) {
        largeString+=string100;
    }
    Assert.expectEq("stringify a largeString",true,'"'+largeString+'"'==JSON.stringify(largeString));


