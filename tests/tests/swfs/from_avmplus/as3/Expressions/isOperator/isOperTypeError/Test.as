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
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "is Operator";       // Provide ECMA section title or a description



var myNumber:Number = 56.342;
var myString:String = "hello";

var result:String = "Type Error Not Thrown";
try {
    myNumber is notAValidType;
} catch (e:ReferenceError) {
    result = e;
} finally {
    Assert.expectEq("is Operator: invalid Type - notAValidType","ReferenceError: Error #1065", Utils.referenceError(result));
}
// The following tests are now compile-time errors in falcon - See CMP-2028 for details.
/*
result = "Type Error Not Thrown";
try {
    myNumber is 897;
} catch (e:TypeError) {
    result = e;
} finally {
    Assert.expectEq("is Operator: invalid Type - 897","TypeError: Error #1009", Utils.typeError(result));
}

result = "Type Error Not Thrown";
try {
    myNumber is myNumber;
} catch (e:TypeError) {
    result = e;
} finally {
    Assert.expectEq("is Operator: invalid Type - value variable on RHS","TypeError: Error #1009", Utils.typeError(result));
}

result = "Type Error Not Thrown";
try {
    myNumber is myString;
} catch (e:TypeError) {
    result = e;
} finally {
    Assert.expectEq("is Operator: invalid Type - different var on RHS","TypeError: Error #1009", Utils.typeError(result));
}
*/
result = "Value error not thrown";
try {
    nonDeclaredVar is Object;
} catch (e:ReferenceError) {
    result = e;
} finally {
    Assert.expectEq("is Operator: undeclared var on LHS","ReferenceError: Error #1065",Utils.referenceError(result));
}




////////////////////////////////////////////////////////////////

              // displays results.
