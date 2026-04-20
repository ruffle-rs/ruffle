/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
import com.adobe.test.Utils;

// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS 3.0";        // Version of ECMAScript or ActionScript
// var TITLE   = "Filter operator on non-XML object - runtime error";       // Provide ECMA section title or a description
var BUGNUMBER = "144368";




var employees = [{fname:"John",age:20},{fname:"Sue",age:30}]
try {
    var names = employees.(fname=="John");
    result = "no exception";
} catch(e1) {
    result = Utils.typeError(e1.toString());

}

expected = "TypeError: Error #1123";

Assert.expectEq("Use wildcard operator on an array", expected, result);

var object = [{a:1, b:2, c:3}];

try {
    var names = object.(a==1);
    result = "no exception";
} catch(e2) {
    result = Utils.typeError(e2.toString());

}

expected = "TypeError: Error #1123";

Assert.expectEq("Use wildcard operator on an object", expected, result);

