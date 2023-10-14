/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//     var SECTION = '15.2.4.5';
//     var VERSION = 'no version';
//     var TITLE = 'hasOwnProperty';


    var count = 0;
    var testcases = new Array();


    testcases[count++] = Assert.expectEq(  "String.prototype.hasOwnProperty(\"split\")", true,String.prototype.hasOwnProperty("split"));

    var str = new String("JScript");

    testcases[count++] = Assert.expectEq(  "str.hasOwnProperty(\"split\")", false,str.hasOwnProperty("split"));

    testcases[count++] = Assert.expectEq(  "Array.prototype.hasOwnProperty(\"pop\")", true,Array.prototype.hasOwnProperty("pop"));

    testcases[count++] = Assert.expectEq(  "Number.prototype.hasOwnProperty(\"toPrecision\")", true,Number.prototype.hasOwnProperty("toPrecision"));

    testcases[count++] = Assert.expectEq(  "Date.prototype.hasOwnProperty(\"getTime\")", true,Date.prototype.hasOwnProperty("getTime"));

    testcases[count++] = Assert.expectEq(  "RegExp.prototype.hasOwnProperty(\"exec\")", true,RegExp.prototype.hasOwnProperty("exec"));

    testcases[count++] = Assert.expectEq(  "String.prototype.hasOwnProperty(\"random\")", false,String.prototype.hasOwnProperty("random"));

    testcases[count++] = Assert.expectEq(  "Object.prototype.hasOwnProperty(\"constructor\")", true,Object.prototype.hasOwnProperty("constructor"));

    testcases[count++] = Assert.expectEq(  "Object.prototype.hasOwnProperty(\"getTime\")", false,Object.prototype.hasOwnProperty("getTime"));

    var myobj = new Object();

    testcases[count++] = Assert.expectEq(  "myobj.hasOwnProperty(\"constructor\")", false,myobj.hasOwnProperty("constructor"));


