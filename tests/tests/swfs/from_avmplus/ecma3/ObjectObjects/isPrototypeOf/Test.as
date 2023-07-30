/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = '15.2.4.6';
//     var VERSION = 'no version';
//     var TITLE = 'isPrototypeOf';


    var count = 0;
    var testcases = new Array();

    var re = new RegExp();


    testcases[count++] = Assert.expectEq(  "RegExp.prototype.isPrototypeOf(re))", true,RegExp.prototype.isPrototypeOf(re));

    var str = new String("JScript");

    testcases[count++] = Assert.expectEq(  "String.prototype.isPrototypeOf(str)", true,String.prototype.isPrototypeOf(str));

    testcases[count++] = Assert.expectEq(  "String.prototype.isPrototypeOf(re)", false,String.prototype.isPrototypeOf(re));

    testcases[count++] = Assert.expectEq(  "String.prototype.isPrototypeOf(new Number())", false,String.prototype.isPrototypeOf(new Number()));

    testcases[count++] = Assert.expectEq(  "Object.prototype.isPrototypeOf(str)", true,Object.prototype.isPrototypeOf(str));

    testcases[count++] = Assert.expectEq(  "Object.prototype.isPrototypeOf(re)", true,Object.prototype.isPrototypeOf(re));

    var myobj = new Object();


    testcases[count++] = Assert.expectEq(  "String.prototype.isPrototypeOf(myobj)", false,String.prototype.isPrototypeOf(myobj));

    var myobj2 = null;

    testcases[count++] = Assert.expectEq(  "Object.prototype.isPrototypeOf(myobj2)", false,Object.prototype.isPrototypeOf(myobj2));



